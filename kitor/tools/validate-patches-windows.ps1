# Requires Windows, Rust/Cargo, and Python 3. Reads owned games; never patches them.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Config,
    [string]$Report = (Join-Path (Get-Location) ("kitor-windows-validation-" + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.json')),
    [switch]$IncludeCandidates
)
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Run this entry point on Windows.' }
$Config = (Resolve-Path -LiteralPath $Config).Path
$Report = [System.IO.Path]::GetFullPath($Report)
if (Test-Path -LiteralPath $Report) { throw 'Report exists. Choose a new -Report filename.' }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Install Rust/Cargo, then reopen PowerShell.' }
if (Get-Command py -ErrorAction SilentlyContinue) {
    $PythonPath = & py -3 -c 'import sys; print(sys.executable)'
} elseif (Get-Command python -ErrorAction SilentlyContinue) {
    $PythonPath = & python -c 'import sys; assert sys.version_info.major == 3; print(sys.executable)'
} else { throw 'Install Python 3 (including its Windows launcher), then reopen PowerShell.' }
if ($LASTEXITCODE -ne 0) { throw 'A working Python 3 installation is required.' }
$env:KITOR_VALIDATION_PYTHON = ($PythonPath | Select-Object -Last 1).Trim()
$BuildDirectory = Join-Path ([System.IO.Path]::GetTempPath()) 'kitor-windows-validation-build'
$Preflight = @'
import json, pathlib, sys
config = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8-sig"))
if not isinstance(config, dict) or not all(isinstance(p, str) for p in config.values()):
    raise SystemExit("Config must map game IDs to directory strings")
locations = [pathlib.Path(p).resolve() for p in sys.argv[2:]]
for configured in config.values():
    if not configured.strip():
        continue
    game = pathlib.Path(configured).resolve()
    for location in locations:
        if location == game or game in location.parents:
            raise SystemExit("Tool directory, build directory, Cargo cache, and report must be outside configured game folders: " + str(location))
'@
$CargoCache = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
& $env:KITOR_VALIDATION_PYTHON -c $Preflight $Config $PSScriptRoot $Report $BuildDirectory $CargoCache
if ($LASTEXITCODE -ne 0) { throw 'Read-only game safety preflight failed before Cargo was invoked.' }
$Manifest = Join-Path $PSScriptRoot 'lifecycle-harness/Cargo.toml'
Write-Host 'Building the native disposable validation tool. Your games remain read-only sources.'
& cargo build --manifest-path $Manifest --release --locked --bin validate_owned_games --target-dir $BuildDirectory
if ($LASTEXITCODE -ne 0) { throw 'Rust validator build failed.' }
$Executable = Join-Path $BuildDirectory 'release/validate_owned_games.exe'
$ValidatorArgs = @($Config, $Report)
if ($IncludeCandidates) { $ValidatorArgs += '--include-candidates' }
& $Executable @ValidatorArgs
$ValidationExitCode = $LASTEXITCODE
if (Test-Path -LiteralPath $Report) {
    Write-Host "Report: $Report"
    Write-Host 'This validates install/uninstall on disposable file subsets. Gameplay has not been tested.'
}
exit $ValidationExitCode
