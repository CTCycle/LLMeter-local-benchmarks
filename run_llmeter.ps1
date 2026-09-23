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

function Test-LauncherInteractive {
    if (-not [Console]::IsInputRedirected -and -not [Console]::IsOutputRedirected) {
        return $true
    }

    # Windows ConPTY presents redirected standard handles to PowerShell even
    # though Read-Host remains interactive. The Rust binary uses this marker
    # for the same ConPTY boundary.
    return $env:LLMETER_CONPTY -eq '1'
}

$script:LauncherInteractive = Test-LauncherInteractive

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

function Read-ConptyConfirmationKey {
    if (-not ('LlmeterConptyInput' -as [type])) {
        Add-Type @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Explicit, Size = 20)]
public struct LlmeterInputRecord {
    [FieldOffset(0)] public ushort EventType;
    [FieldOffset(4)] public int KeyDown;
    [FieldOffset(8)] public ushort RepeatCount;
    [FieldOffset(10)] public ushort VirtualKeyCode;
    [FieldOffset(12)] public ushort VirtualScanCode;
    [FieldOffset(14)] public ushort UnicodeChar;
    [FieldOffset(16)] public uint ControlKeyState;
}

public static class LlmeterConptyInput {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr CreateFile(string name, uint access, uint share, IntPtr security, uint creation, uint flags, IntPtr template);

    [DllImport("kernel32.dll", EntryPoint = "ReadConsoleInputW", SetLastError = true)]
    private static extern bool ReadConsoleInput(IntPtr input, [Out] LlmeterInputRecord[] records, uint length, out uint read);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool GetConsoleMode(IntPtr input, out uint mode);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetConsoleMode(IntPtr input, uint mode);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool CloseHandle(IntPtr handle);

    public static string ReadKey() {
        IntPtr input = CreateFile("CONIN$", 0x80000000u | 0x40000000u, 0x1u | 0x2u, IntPtr.Zero, 3u, 0u, IntPtr.Zero);
        if (input == new IntPtr(-1))
            throw new Win32Exception(Marshal.GetLastWin32Error());

        uint originalMode = 0;
        bool restoreMode = GetConsoleMode(input, out originalMode);
        try {
            if (restoreMode) {
                uint rawMode = (originalMode & ~(1u | 2u | 4u | 16u)) | 32u | 64u | 128u | 512u;
                if (!SetConsoleMode(input, rawMode))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
            }

            var records = new LlmeterInputRecord[1];
            while (true) {
                uint read;
                if (!ReadConsoleInput(input, records, 1, out read))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                if (read == 1 && records[0].EventType == 1 && records[0].KeyDown != 0)
                    return ((char)records[0].UnicodeChar).ToString();
            }
        }
        finally {
            if (restoreMode)
                SetConsoleMode(input, originalMode);
            CloseHandle(input);
        }
    }
}
'@
    }
    return [LlmeterConptyInput]::ReadKey()
}

function Confirm-DestructiveAction([string]$Description) {
    if (-not $script:LauncherInteractive) {
        throw "The destructive action '$Description' requires an interactive console; no files were changed."
    }
    $prompt = "Continue to $($Description)? [y/N]"
    if ($env:LLMETER_CONPTY -eq '1') {
        Write-Host $prompt -NoNewline
        $confirmation = Read-ConptyConfirmationKey
        Write-Host ''
    }
    else {
        $confirmation = Read-Host $prompt
    }
    $confirmation = ([string]$confirmation).Trim()
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
    $llmeterHome = [IO.Path]::GetFullPath($configuredHome).TrimEnd('\')
    $filesystemRoot = ([IO.Path]::GetPathRoot($llmeterHome)).TrimEnd('\')
    $userProfilePath = [Environment]::GetFolderPath('UserProfile')
    $userProfile = if ([string]::IsNullOrWhiteSpace($userProfilePath)) {
        ''
    } else {
        [IO.Path]::GetFullPath($userProfilePath).TrimEnd('\')
    }
    $repository = [IO.Path]::GetFullPath($repoRoot).TrimEnd('\')
    if ($llmeterHome -eq $filesystemRoot -or $llmeterHome -eq $userProfile -or $llmeterHome -eq $repository) {
        throw "Refusing to remove broad or protected LLMeter home '$llmeterHome'. Set LLMETER_HOME to a dedicated directory."
    }
    return $llmeterHome
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
    [void]$targets.Add($defaultTargetDir)
    [void]$targets.Add($fallbackTargetDir)
    foreach ($legacyCachePath in $legacyCachePaths) {
        [void]$targets.Add($legacyCachePath)
    }
    if ($IncludeHomeData) {
        $llmeterHome = Get-LlmeterHomePath
        foreach ($relative in @('bin', 'config', 'benchmark_results')) {
            [void]$targets.Add((Join-Path $llmeterHome $relative))
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

function Format-LlmeterArgsForDisplay {
    param([string[]]$Arguments)

    if ($null -eq $Arguments -or $Arguments.Count -eq 0) { return '(no arguments)' }
    $displayArguments = [Collections.Generic.List[string]]::new()
    for ($index = 0; $index -lt $Arguments.Count; $index++) {
        $argument = $Arguments[$index]
        if ($argument -ieq '--base-url') {
            [void]$displayArguments.Add($argument)
            if ($index + 1 -lt $Arguments.Count) {
                [void]$displayArguments.Add('[redacted]')
                $index++
            }
            continue
        }
        if ($argument.StartsWith('--base-url=', [StringComparison]::OrdinalIgnoreCase)) {
            [void]$displayArguments.Add('--base-url=[redacted]')
            continue
        }
        [void]$displayArguments.Add($argument)
    }
    return [string]::Join(' ', $displayArguments)
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

    $displayArgs = Format-LlmeterArgsForDisplay -Arguments $LlmeterArgs
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
