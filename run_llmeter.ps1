param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$LlmeterArgs
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$manifestPath = Join-Path $repoRoot "Cargo.toml"
$binaryPath = Join-Path $repoRoot "target\release\llmeter.exe"

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

Push-Location $repoRoot
try {
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        throw "Could not find Cargo.toml in $repoRoot."
    }

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "cargo is not on PATH. Install the stable Rust toolchain, then rerun this script."
    }

    if (Test-NeedsBuild -RepoRoot $repoRoot -BinaryPath $binaryPath -ManifestPath $manifestPath) {
        Write-Host "Building llmeter (release)..." -ForegroundColor Cyan
        & cargo build --release
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
    else {
        Write-Host "Using existing release build." -ForegroundColor DarkGray
    }

    if (-not (Test-Path -LiteralPath $binaryPath)) {
        throw "Build completed, but $binaryPath was not found."
    }

    & $binaryPath @LlmeterArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
