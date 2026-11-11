<#
.SYNOPSIS
  Builds the SPIL Labs desktop app (Tauri) and its Windows installers.
  Run this once on any Windows PC with Rust + VS Build Tools + Node.js; give the installer to users.
  The app itself has no data or passwords: on first start it asks for the server address.

.OUTPUTS
  desktop\src-tauri\target\release\bundle\msi\SPIL Labs Assistant_1.0.0_x64_en-US.msi
  desktop\src-tauri\target\release\bundle\nsis\SPIL Labs Assistant_1.0.0_x64-setup.exe
#>
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
function Fail($text) { Write-Host "`nERROR: $text" -ForegroundColor Red; exit 1 }
foreach ($c in @("cargo", "npm")) {
    if (-not (Get-Command $c -ErrorAction SilentlyContinue)) { Fail "'$c' was not found (see README: prerequisites)." }
}

Set-Location (Join-Path $Root "desktop")
Write-Host "`n==> Installing the Tauri tools" -ForegroundColor Cyan
npm install
if ($LASTEXITCODE -ne 0) { Fail "npm install failed" }

Write-Host "`n==> Building the desktop app and installers (the first build takes a few minutes)" -ForegroundColor Cyan
npx tauri build
if ($LASTEXITCODE -ne 0) { Fail "tauri build failed (see the messages above)" }

$bundle = Join-Path $Root "desktop\src-tauri\target\release\bundle"
Write-Host "`nDone. Installers:" -ForegroundColor Green
Get-ChildItem $bundle -Recurse -Include *.msi, *-setup.exe | ForEach-Object { Write-Host "  $($_.FullName)" }
