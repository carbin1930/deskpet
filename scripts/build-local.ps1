# Local build helper.
#
# Two paths:
#   1. Healthy machine (VS Build Tools / MinGW installed): just run
#        npm run tauri build
#      You do NOT need this script.
#   2. This dev machine lacks a C linker. This script wires up Rust's bundled
#      lld as the linker and uses the locally fetched MSYS2 binutils in .tools/
#      (dlltool / as / ld / windres + a tiny gcc shim for RC preprocessing).
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-local.ps1
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$tools = Join-Path $root ".tools"

# Prepend local toolchain (binutils + gcc shim) so windres/dlltool resolve.
if (Test-Path $tools) {
  $env:PATH = "$tools;$env:PATH"
  Write-Host "Prepended local toolchain: $tools"
}

$sysroot = & rustc --print sysroot
if ($LASTEXITCODE -ne 0) { throw "rustc not found" }

$lld = Join-Path $sysroot "lib/rustlib/x86_64-pc-windows-gnu/bin/rust-lld.exe"
if (-not (Test-Path $lld)) {
  throw "rust-lld.exe not found at: $lld"
}

Write-Host "Using linker: $lld"
$env:RUSTFLAGS = "-C linker=$lld"

Push-Location (Join-Path $root "src-tauri")
try {
  cargo build -j 1
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
  Write-Host "Build OK. Binary: src-tauri/target/debug/deskpet.exe"
} finally {
  Pop-Location
}
