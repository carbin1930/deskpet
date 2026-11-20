# Add the deskpet project + build output to Windows Defender exclusions.
#
# WHY: this machine builds with a hacked GNU toolchain. While compiling crates
#      like windows-sys / parking_lot_core, rustc asks dlltool to generate
#      temporary *.dll_imports.lib files under target/. Defender's real-time
#      scan can lock or delete them mid-build, producing:
#        os error 5  (access denied)   -> fresh build-script exe locked
#        os error 2  (file not found)  -> generated import lib removed
#      An exclusion stops all of that.
#
# USAGE:
#   powershell -ExecutionPolicy Bypass -File scripts\exclude-defender.ps1
#   A UAC prompt appears -> click Yes.
#
# Keep this file pure ASCII (PowerShell 5.1 reads no-BOM UTF-8 as GBK).

$ErrorActionPreference = "Continue"

# Re-launch as administrator if the current token is not elevated.
$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  Write-Host "Requesting admin rights... please click Yes on the UAC prompt."
  Start-Process -FilePath "powershell.exe" -ArgumentList @(
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "`"$PSCommandPath`""
  ) -Verb RunAs -Wait
  exit 0
}

$root = Split-Path -Parent $PSScriptRoot
$paths = @(
  $root,
  (Join-Path $root "src-tauri\target")
)

try {
  $existing = (Get-MpPreference).ExclusionPath
} catch {
  Write-Error "Could not query Defender settings. Is Microsoft Defender enabled?"
  exit 1
}

foreach ($p in $paths) {
  if ($existing -contains $p) {
    Write-Host "Already excluded: $p"
  } else {
    Add-MpPreference -ExclusionPath $p
    Write-Host "Added exclusion: $p"
  }
}

Write-Host ""
Write-Host "Done. Now re-run:"
Write-Host "  powershell -ExecutionPolicy Bypass -File scripts\dev-local.ps1"
