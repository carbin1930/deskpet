# Local dev runner for this machine (Windows, no VS Build Tools).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\dev-local.ps1
#
# What it does:
#   1. Prepend the repo-local .tools/ (dlltool / as / ld / windres / gcc shim) to PATH
#      Otherwise crates like windows-version fail with "dlltool.exe: program not found".
#   2. Use Rust's bundled rust-lld as linker (no MSVC link.exe on this box).
#   3. Warm up cargo check in a retry loop (Defender transiently locks fresh exe -> os error 5),
#      then launch `tauri dev` (frontend hot reload + native transparent window).
#
# Press Ctrl+C to stop.

# Keep "Continue": cargo writes normal progress (Compiling ...) to stderr,
# and PowerShell 5.1 would treat that as a NativeCommandError under "Stop".
# We check $LASTEXITCODE explicitly and still use throw for real failures.
$ErrorActionPreference = "Continue"

$root = Split-Path -Parent $PSScriptRoot
$tools = Join-Path $root ".tools"

if (-not (Test-Path (Join-Path $tools "dlltool.exe"))) {
  throw "dlltool.exe not found in $tools ; .tools/ is missing."
}
$env:PATH = "$tools;$env:PATH"
Write-Host "[1/3] Local toolchain added to PATH: $tools"

$sysroot = & rustc --print sysroot
if ($LASTEXITCODE -ne 0) { throw "rustc not found" }
$lld = Join-Path $sysroot "lib/rustlib/x86_64-pc-windows-gnu/bin/rust-lld.exe"
if (-not (Test-Path $lld)) { throw "rust-lld.exe not found at: $lld" }
$env:RUSTFLAGS = "-C linker=$lld"
Write-Host "[2/3] Using linker: $lld"

Push-Location $root
try {
  Write-Host "[3/3] Warming up Rust build scripts..."
  $ok = $false
  for ($i = 1; $i -le 8; $i++) {
    & cargo check --manifest-path src-tauri/Cargo.toml
    if ($LASTEXITCODE -eq 0) {
      $ok = $true
      Write-Host "Rust check OK (attempt $i)"
      break
    }
    Write-Host "cargo check attempt $i failed, retrying in 1s..."
    Start-Sleep -Seconds 1
  }
  if (-not $ok) { throw "cargo check failed after 8 attempts" }

  Write-Host "Launching tauri dev ... (Ctrl+C to stop)"
  & npm run tauri -- dev
  if ($LASTEXITCODE -ne 0) { throw "tauri dev exited with code $LASTEXITCODE" }
} finally {
  Pop-Location
}
