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
#
# Keep this file pure ASCII: PowerShell 5.1 reads UTF-8-without-BOM as GBK and
# non-ASCII quotes break parsing.

# Continue (not Stop): tauri CLI writes normal Info/Warn lines to stderr, which
# PowerShell 5.1 treats as NativeCommandError under Stop and aborts the build.
# Real failures are caught by explicit $LASTEXITCODE checks + exit codes.
$ErrorActionPreference = "Continue"

$root = Split-Path -Parent $PSScriptRoot
$tools = Join-Path $root ".tools"

# Single-threaded cargo: concurrent rustc instances otherwise fight over the
# same temporary *.dll_imports.lib produced by dlltool on windows-gnu.
$env:CARGO_BUILD_JOBS = "1"

if (Test-Path $tools) {
  $env:PATH = "$tools;$env:PATH"
  Write-Host "Prepended local toolchain: $tools"
}

$sysroot = & rustc --print sysroot
if ($LASTEXITCODE -ne 0) { Write-Error "rustc not found"; exit 1 }

$lld = Join-Path $sysroot "lib/rustlib/x86_64-pc-windows-gnu/bin/rust-lld.exe"
if (-not (Test-Path $lld)) {
  Write-Error "rust-lld.exe not found at: $lld"
  exit 1
}

Write-Host "Using linker: $lld"
$env:RUSTFLAGS = "-C linker=$lld"

# Clear the frontend output dir ourselves before building. Vite's emptyDir
# would otherwise try to remove it from inside node; on some setups that
# delete is blocked/hooked and the build fails before Rust even starts.
$dist = Join-Path $root "dist"
if (Test-Path $dist) {
  Remove-Item -Recurse -Force $dist -Confirm:$false
  Write-Host "Cleared prior dist/ output."
}

Push-Location $root
try {
  # --no-bundle: embed frontend, skip MSI/NSIS installer tooling.
  # Retry a few times: Defender can transiently lock a freshly generated
  # build-script exe (os error 5). Each retry reuses the cargo cache.
  $ok = $false
  for ($i = 1; $i -le 10; $i++) {
    Write-Host "=== Build attempt $i / 10 ==="
    npm run tauri -- build --no-bundle
    if ($LASTEXITCODE -eq 0) { $ok = $true; break }
    Write-Host "Build attempt $i failed (exit $LASTEXITCODE); retrying in 2s..."
    Start-Sleep -Seconds 2
  }
  if (-not $ok) {
    Write-Error "tauri build failed after 10 attempts"
    exit 1
  }
  Write-Host "Build OK. Self-contained exe: src-tauri/target/release/deskpet.exe"
} finally {
  Pop-Location
}
exit 0
