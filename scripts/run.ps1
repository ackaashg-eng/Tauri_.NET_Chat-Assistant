<#
.SYNOPSIS
  Starts the SPIL Labs server: the Rust backend serves the screens and /api, and starts the
  C# MCP server itself (through its built-in Rust MCP client, over STDIO).
  People connect with a web browser or the SPIL Labs desktop app.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts\run.ps1 -Lan      # shared server: other PCs can connect
  powershell -ExecutionPolicy Bypass -File scripts\run.ps1            # only this PC (http://localhost:8000)
#>
param(
    [int]$Port = 8000,
    [switch]$Lan
)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Exe = Join-Path $Root "dist\spil-backend.exe"
if (-not (Test-Path $Exe)) { Write-Host "Run scripts\setup.ps1 first." -ForegroundColor Red; exit 1 }
if (-not (Test-Path (Join-Path $Root "config\server.env"))) { Write-Host "config\server.env is missing - run scripts\setup.ps1." -ForegroundColor Red; exit 1 }

$extra = @("--port", $Port)
if ($Lan) {
    $extra += "--lan"
    $ips = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike "127.*" -and $_.IPAddress -notlike "169.254.*" }).IPAddress
    Write-Host "Server address for the desktop app / browser:  $($ips | ForEach-Object { "http://$($_):$Port" })" -ForegroundColor Green
    Write-Host "If other PCs can't connect, allow the port once (PowerShell as Administrator):"
    Write-Host "  New-NetFirewallRule -DisplayName 'SPIL Labs Assistant' -Direction Inbound -Protocol TCP -LocalPort $Port -Action Allow"
} else {
    Write-Host "Open http://localhost:$Port" -ForegroundColor Green
}

# Run from the project folder: paths in config\server.env are relative to it.
Set-Location $Root
& $Exe --config "config\server.env" @extra
