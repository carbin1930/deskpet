# One-click BUILD + RUN for this machine (Windows, no VS Build Tools).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\dev-local.ps1
#
# What it does:
#   1. Calls build-local.ps1, which sets up the local GNU toolchain (.tools/ on
#      PATH + rust-lld as linker) and builds a self-contained release exe with
#      built-in retries for Defender file locks. This avoids the
#      "dlltool.exe: program not found" error from plain cargo/tauri commands.
#   2. Launches the exe so you can test the pet (right-click menu, edge docking,
#      layout fix) on your desktop.
#
# To stop: use the tray icon -> Quit, or close the pet window.
#
# NOTE: keep this file pure ASCII -- PowerShell 5.1 reads UTF-8-without-BOM as
#       GBK and non-ASCII quotes break parsing.

$ErrorActionPreference = "Continue"

$root = Split-Path -Parent $PSScriptRoot
$buildScript = Join-Path $PSScriptRoot "build-local.ps1"
$exe = Join-Path $root "src-tauri/target/release/deskpet.exe"

& $buildScript
if ($LASTEXITCODE -ne 0) {
  Write-Error "Build failed; not launching."
  exit 1
}
if (-not (Test-Path $exe)) {
  Write-Error "exe not found after build: $exe"
  exit 1
}

Write-Host "=== Launching $exe ==="
& $exe
