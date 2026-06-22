param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$LlmeterArgs
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$manifestPath = Join-Path $repoRoot "Cargo.toml"
$defaultTargetDir = Join-Path $repoRoot "target"
$defaultBinaryPath = Join-Path $defaultTargetDir "release\llmeter.exe"
$fallbackTargetDir = Join-Path ([System.IO.Path]::GetTempPath()) "llmeter-build"
$fallbackBinaryPath = Join-Path $fallbackTargetDir "release\llmeter.exe"

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

    foreach ($path in $inputPaths) {
        $files = Get-ChildItem -LiteralPath $path -Recurse -File
        foreach ($file in $files) {
            if ($file.LastWriteTimeUtc -gt $binaryTimestamp) {
                return $true
            }
        }
    }

    return $false
}

function Invoke-CargoBuild {
    param(
        [string]$TargetDir
    )

    $buildArgs = @("build", "--release")
    if ($TargetDir) {
        $buildArgs += @("--target-dir", $TargetDir)
    }

    & cargo @buildArgs
    return $LASTEXITCODE
}

Push-Location $repoRoot
try {
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        throw "Could not find Cargo.toml in $repoRoot."
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

    & $selectedBinaryPath @LlmeterArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
