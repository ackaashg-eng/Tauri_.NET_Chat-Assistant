<#
.SYNOPSIS
  Development mode: the Rust backend on :8000 (cargo run) and the Vite dev server on :5173
  with instant reload of the React screens (open http://localhost:5173). Each runs in its own window.
#>
$Root = Split-Path -Parent $PSScriptRoot
Start-Process powershell -ArgumentList "-NoExit", "-Command", "Set-Location '$Root'; cargo run --manifest-path client-rust\Cargo.toml -p spil-backend -- --config config\server.env"
Start-Process powershell -ArgumentList "-NoExit", "-Command", "Set-Location '$Root\frontend'; npm run dev"
Write-Host "Backend: http://localhost:8000/api/health   Frontend: http://localhost:5173"
