# Local build helper (for this machine, which lacks a C linker).
#
# Produces a SELF-CONTAINED release exe at src-tauri/target/release/deskpet.exe
# with the Vue frontend embedded. It uses:
#   - Rust's bundled lld as linker
#   - locally fetched MSYS2 binutils in .tools/ (dlltool/as/ld/windres)
#   - a tiny gcc shim in .tools/ for windres RC preprocessing
#
# NOTE: `cargo build --release` alone does NOT embed the frontend (white screen).
#       You MUST use `tauri build` so build.frontendDist (dist/) is compiled in.
#       Add --no-bundle to skip MSI/NSIS installer generation.
#
# On a healthy machine with VS Build Tools, simply run: npm run tauri build
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-local.ps1
# Continue (not Stop): tauri CLI writes normal Info/Warn lines to stderr, which
# PowerShell 5.1 treats as NativeCommandError under Stop and aborts the build.
# Real failures are still caught by the explicit $LASTEXITCODE checks + throw.
$ErrorActionPreference = "Continue"

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

Push-Location $root
try {
  # --no-bundle: embed frontend, skip MSI/NSIS installer tooling
  npm run tauri -- build --no-bundle
  if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }
  Write-Host "Build OK. Self-contained exe: src-tauri/target/release/deskpet.exe"
} finally {
  Pop-Location
}
