param(
    [ValidateSet('Run', 'Clean', 'RemoveAllData', 'Uninstall')]
    [string]$Action = 'Run',
    [switch]$WhatIf,
    [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
    [string[]]$LlmeterArgs
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$manifestPath = Join-Path $repoRoot "Cargo.toml"
$defaultTargetDir = Join-Path $repoRoot "target"
$defaultBinaryPath = Join-Path $defaultTargetDir "release\llmeter.exe"
$fallbackTargetDir = Join-Path ([System.IO.Path]::GetTempPath()) "llmeter-build"
$fallbackBinaryPath = Join-Path $fallbackTargetDir "release\llmeter.exe"
$legacyCachePaths = @(
    (Join-Path $repoRoot '.uv-cache')
)
$script:NextProgressId = 1
$script:ActiveProgressActivities = [Collections.Generic.Dictionary[int, string]]::new()
$script:LauncherInteractive = -not [Console]::IsInputRedirected -and -not [Console]::IsOutputRedirected

function Start-LauncherProgress {
    param([Parameter(Mandatory)][string]$Activity, [Parameter(Mandatory)][string]$Status)
    $id = $script:NextProgressId++
    $script:ActiveProgressActivities[$id] = $Activity
    if ($script:LauncherInteractive) { Write-Progress -Id $id -Activity $Activity -Status $Status }
    return $id
}

function Update-LauncherProgress {
    param(
        [Parameter(Mandatory)][int]$Id,
        [Parameter(Mandatory)][string]$Activity,
        [Parameter(Mandatory)][string]$Status,
        [Nullable[int]]$PercentComplete
    )
    if (-not $script:ActiveProgressActivities.ContainsKey($Id)) { return }
    $activity = $script:ActiveProgressActivities[$Id]
    $progress = @{ Id = $Id; Activity = $activity; Status = $Status }
    if ($null -ne $PercentComplete) { $progress.PercentComplete = $PercentComplete }
    if ($script:LauncherInteractive) { Write-Progress @progress }
}

function Complete-LauncherProgress([int]$Id) {
    if ($script:ActiveProgressActivities.ContainsKey($Id)) {
        $activity = $script:ActiveProgressActivities[$Id]
        try {
            if ($script:LauncherInteractive) {
                try { Write-Progress -Id $Id -Activity $activity -Completed } catch { }
            }
        }
        finally {
            [void]$script:ActiveProgressActivities.Remove($Id)
        }
    }
}

function Clear-LauncherProgress {
    foreach ($id in @($script:ActiveProgressActivities.Keys)) {
        Complete-LauncherProgress -Id $id
    }
}

function Confirm-DestructiveAction([string]$Description) {
    if (-not $script:LauncherInteractive) {
        throw "The destructive action '$Description' requires an interactive console; no files were changed."
    }
    $confirmation = ([string](Read-Host "Continue to $($Description)? [y/N]")).Trim()
    if ($confirmation -notmatch '^(?i:y|yes)$') {
        Write-Host '[INFO] Operation cancelled. No changes were made.' -ForegroundColor DarkGray
        return $false
    }
    return $true
}

function Get-LlmeterHomePath {
    $configuredHome = if ([string]::IsNullOrWhiteSpace($env:LLMETER_HOME)) {
        Join-Path ([Environment]::GetFolderPath('UserProfile')) '.llmeter'
    } else {
        [Environment]::ExpandEnvironmentVariables($env:LLMETER_HOME.Trim())
    }
    if (-not [IO.Path]::IsPathRooted($configuredHome)) {
        $configuredHome = Join-Path $repoRoot $configuredHome
    }
    $home = [IO.Path]::GetFullPath($configuredHome).TrimEnd('\')
    $filesystemRoot = ([IO.Path]::GetPathRoot($home)).TrimEnd('\')
    $userProfile = [IO.Path]::GetFullPath([Environment]::GetFolderPath('UserProfile')).TrimEnd('\')
    $repository = [IO.Path]::GetFullPath($repoRoot).TrimEnd('\')
    if ($home -eq $filesystemRoot -or $home -eq $userProfile -or $home -eq $repository) {
        throw "Refusing to remove broad or protected LLMeter home '$home'. Set LLMETER_HOME to a dedicated directory."
    }
    return $home
}

function Remove-LauncherPath([string]$Path) {
    $fullPath = [IO.Path]::GetFullPath($Path)
    $filesystemRoot = ([IO.Path]::GetPathRoot($fullPath)).TrimEnd('\')
    $repository = [IO.Path]::GetFullPath($repoRoot).TrimEnd('\')
    if ($fullPath.TrimEnd('\') -eq $filesystemRoot -or $fullPath.TrimEnd('\') -eq $repository) {
        throw "Refusing to remove a filesystem or repository root: $fullPath"
    }
    if (-not (Test-Path -LiteralPath $fullPath)) { return }
    if ($WhatIf) {
        Write-Host "[WHATIF] Would remove $fullPath" -ForegroundColor DarkGray
        return
    }
    Remove-Item -LiteralPath $fullPath -Recurse -Force -Confirm:$false -ErrorAction Stop
}

function Get-LauncherCleanupTargets {
    param([switch]$IncludeHomeData)

    $targets = [Collections.Generic.List[string]]::new()
    $targets.Add($defaultTargetDir)
    $targets.Add($fallbackTargetDir)
    foreach ($legacyCachePath in $legacyCachePaths) {
        $targets.Add($legacyCachePath)
    }
    if ($IncludeHomeData) {
        $home = Get-LlmeterHomePath
        foreach ($relative in @('bin', 'config', 'benchmark_results')) {
            $targets.Add((Join-Path $home $relative))
        }
    }
    return @($targets | Select-Object -Unique)
}

function Invoke-LauncherCleanup {
    param([switch]$IncludeHomeData)

    foreach ($target in @(Get-LauncherCleanupTargets -IncludeHomeData:$IncludeHomeData)) {
        Remove-LauncherPath -Path $target
    }
    Write-Host '[DONE] Launcher-owned cleanup completed.' -ForegroundColor Green
}

function Test-NeedsBuild {
    param(
        [string]$RepoRoot,
        [string]$BinaryPath,
        [string]$ManifestPath
    )

    if (-not (Test-Path -LiteralPath $BinaryPath)) {
        return $true
    }

    $binaryTimestamp = (Get-Item -LiteralPath $BinaryPath).LastWriteTimeUtc
    $inputPaths = @($ManifestPath)

    $srcPath = Join-Path $RepoRoot "src"
    if (Test-Path -LiteralPath $srcPath) {
        $inputPaths += $srcPath
    }

    $files = @(foreach ($path in $inputPaths) {
        Get-ChildItem -LiteralPath $path -Recurse -File
    })
    $progressId = Start-LauncherProgress -Activity 'LLMeter: inspect build inputs' -Status "0 of $($files.Count) files"
    try {
        for ($index = 0; $index -lt $files.Count; $index++) {
            $file = $files[$index]
            $percent = if ($files.Count -eq 0) { 100 } else { [int](($index + 1) * 100 / $files.Count) }
            Update-LauncherProgress -Id $progressId -Activity 'LLMeter: inspect build inputs' -Status "$($index + 1) of $($files.Count): $($file.Name)" -PercentComplete $percent
            if ($file.LastWriteTimeUtc -gt $binaryTimestamp) {
                return $true
            }
        }
        return $false
    }
    finally {
        Complete-LauncherProgress $progressId
    }
}

function Invoke-CargoBuild {
    param(
        [string]$TargetDir
    )

    $buildArgs = @("build", "--release")
    if ($TargetDir) {
        $buildArgs += @("--target-dir", $TargetDir)
    }

    $activity = "LLMeter: cargo $([string]::Join(' ', $buildArgs))"
    Write-Host "[RUN] $activity" -ForegroundColor Cyan
    & cargo @buildArgs
    $exitCode = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
    if ($exitCode -eq 0) {
        Write-Host '[DONE] cargo build completed.' -ForegroundColor Green
    }
    else {
        Write-Host "[FAIL] cargo build exited with code $exitCode." -ForegroundColor Red
    }
    return $exitCode
}

Push-Location $repoRoot
try {
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        throw "Could not find Cargo.toml in $repoRoot."
    }

    if ($Action -eq 'Clean') {
        if (Confirm-DestructiveAction 'remove LLMeter build artifacts') {
            Invoke-LauncherCleanup
        }
        exit 0
    }

    if ($Action -eq 'RemoveAllData') {
        if (Confirm-DestructiveAction 'remove all LLMeter-owned home data and build artifacts') {
            Invoke-LauncherCleanup -IncludeHomeData
        }
        exit 0
    }

    if ($Action -eq 'Uninstall') {
        if (-not (Confirm-DestructiveAction 'remove the managed LLMeter installation while preserving home data')) {
            exit 0
        }
        $LlmeterArgs = @('uninstall')
    }

    $destructiveForwardedCommand = @($LlmeterArgs | Where-Object { $_ -ieq 'uninstall' -or $_ -ieq '--purge-home' })
    if ($Action -eq 'Run' -and $destructiveForwardedCommand.Count -gt 0) {
        if (-not (Confirm-DestructiveAction 'run the destructive LLMeter uninstall command')) {
            exit 0
        }
    }

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "cargo is not on PATH. Install the stable Rust toolchain, then rerun this script."
    }

    $selectedBinaryPath = $null
    $defaultNeedsBuild = Test-NeedsBuild -RepoRoot $repoRoot -BinaryPath $defaultBinaryPath -ManifestPath $manifestPath
    $fallbackNeedsBuild = Test-NeedsBuild -RepoRoot $repoRoot -BinaryPath $fallbackBinaryPath -ManifestPath $manifestPath

    if (-not $defaultNeedsBuild) {
        $selectedBinaryPath = $defaultBinaryPath
        Write-Host "Using existing release build." -ForegroundColor DarkGray
    }
    elseif (-not $fallbackNeedsBuild) {
        $selectedBinaryPath = $fallbackBinaryPath
        Write-Host "Using existing fallback release build from $fallbackTargetDir." -ForegroundColor DarkGray
    }
    else {
        Write-Host "Building llmeter (release)..." -ForegroundColor Cyan
        $defaultBuildExitCode = Invoke-CargoBuild -TargetDir $null
        if ($defaultBuildExitCode -eq 0 -and (Test-Path -LiteralPath $defaultBinaryPath)) {
            $selectedBinaryPath = $defaultBinaryPath
        }
        else {
            Write-Host "Default target build failed. Retrying with fallback target dir $fallbackTargetDir..." -ForegroundColor Yellow
            $fallbackBuildExitCode = Invoke-CargoBuild -TargetDir $fallbackTargetDir
            if ($fallbackBuildExitCode -ne 0) {
                exit $fallbackBuildExitCode
            }
            $selectedBinaryPath = $fallbackBinaryPath
        }
    }

    if (-not $selectedBinaryPath) {
        throw "No llmeter executable was selected."
    }

    if (-not (Test-Path -LiteralPath $selectedBinaryPath)) {
        throw "Build completed, but $selectedBinaryPath was not found."
    }

    $displayArgs = if ($LlmeterArgs.Count -gt 0) { $LlmeterArgs -join ' ' } else { '(no arguments)' }
    $activity = "LLMeter: run release binary ($displayArgs)"
    Write-Host "[RUN] $activity" -ForegroundColor Cyan
    & $selectedBinaryPath @LlmeterArgs
    $exitCode = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
    if ($exitCode -eq 0) {
        Write-Host '[DONE] llmeter completed.' -ForegroundColor Green
        if (@($LlmeterArgs | Where-Object { $_ -ieq 'uninstall' -or $_ -ieq '--purge-home' }).Count -gt 0) {
            Invoke-LauncherCleanup
        }
    }
    else {
        Write-Host "[FAIL] llmeter exited with code $exitCode." -ForegroundColor Red
    }
    exit $exitCode
}
finally {
    Pop-Location
    Clear-LauncherProgress
}
