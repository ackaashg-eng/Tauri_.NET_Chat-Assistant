<#
.SYNOPSIS
  One-time setup of the SHARED SERVER (Windows): finds Microsoft SQL Server, builds the C# MCP
  server and creates the database, builds the Rust backend (with the MCP client inside) and the
  React screens, and writes config\server.env. Safe to run again at any time.
  Existing accounts and settings from the old Python backend (backend\.env, backend\data\users.db)
  are moved over automatically.
  The desktop app is built separately: scripts\build-desktop.ps1

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts\setup.ps1
  powershell -ExecutionPolicy Bypass -File scripts\setup.ps1 -SqlServer "MYSERVER\SQLEXPRESS" -Database LumenSearch
#>
[CmdletBinding()]
param(
    [string]$SqlServer,                 # e.g. localhost\SQLEXPRESS  (found automatically if omitted)
    [string]$Database = "LumenSearch",
    [string]$ConnectionString,          # full connection string instead of -SqlServer/-Database (e.g. SQL login)
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Step($text) { Write-Host "`n==> $text" -ForegroundColor Cyan }
function Fail($text) { Write-Host "`nERROR: $text" -ForegroundColor Red; exit 1 }
function Need($cmd, $hint) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) { Fail "'$cmd' was not found. $hint" }
}

# ---------------------------------------------------------------------------
Step "Checking prerequisites"
Need "dotnet" "Install the .NET 10 SDK: https://dotnet.microsoft.com/download"
Need "cargo"  "Install Rust (choose the MSVC toolchain): https://rustup.rs  - it also needs 'Visual Studio Build Tools' with the C++ workload."
Need "node"   "Install Node.js 20 or newer: https://nodejs.org"
$sdks = @(dotnet --list-sdks)
if (-not ($sdks -match '^1[0-9]\.')) { Fail ".NET 10 SDK is required. Installed SDKs:`n$($sdks -join "`n")" }
Write-Host "dotnet, cargo and node found."

# ---------------------------------------------------------------------------
Step "Finding Microsoft SQL Server"
if (-not $ConnectionString) {
    if (-not $SqlServer) {
        $services = @(Get-Service -ErrorAction SilentlyContinue | Where-Object { $_.Name -eq 'MSSQLSERVER' -or $_.Name -like 'MSSQL$*' })
        if ($services.Count -eq 0) {
            Fail ("No SQL Server was found on this PC.`n" +
                  "  Install the free SQL Server Express: https://www.microsoft.com/sql-server/sql-server-downloads`n" +
                  "  (click 'Download now' under Express, then choose 'Basic'), then run this script again.`n" +
                  "  To use a SQL Server on another computer:  scripts\setup.ps1 -SqlServer 'SERVERNAME\INSTANCE'")
        }
        $pick = $services | Sort-Object { $_.Status -ne 'Running' } | Select-Object -First 1
        $SqlServer = if ($pick.Name -eq 'MSSQLSERVER') { 'localhost' } else { 'localhost\' + $pick.Name.Substring(6) }
        if ($pick.Status -ne 'Running') {
            Write-Host "SQL Server service '$($pick.Name)' is stopped - starting it..."
            try { Start-Service $pick.Name } catch {
                Fail "SQL Server is installed but not running. Open the 'Services' app, find '$($pick.DisplayName)' and click Start (or run PowerShell as Administrator: Start-Service '$($pick.Name)')."
            }
        }
    }
    # Trusted_Connection = sign in with your Windows account (no SQL password needed).
    $ConnectionString = "Server=$SqlServer;Database=$Database;Trusted_Connection=True;TrustServerCertificate=True;Encrypt=True"
}
Write-Host "Connection: $ConnectionString"

# ---------------------------------------------------------------------------
Step "Building the C# MCP server (.NET, STDIO)"
$ServerOut = Join-Path $Root "dist\server"
dotnet publish "server-dotnet\SemanticSearch.McpServer\SemanticSearch.McpServer.csproj" `
    -c Release -r win-x64 --self-contained false -o $ServerOut
if ($LASTEXITCODE -ne 0) { Fail "dotnet publish failed" }
$ServerExe = Join-Path $ServerOut "SemanticSearch.McpServer.exe"
# The rules are read from the source folder, so edits there apply after "Reload rules and data".
$RulesFile = Join-Path $Root "server-dotnet\SemanticSearch.McpServer\Rules\search-rules.json"

Step "Creating the database and loading the search rules"
& $ServerExe --connection $ConnectionString --rules $RulesFile --index-only
if ($LASTEXITCODE -ne 0) { Fail "The MCP server could not use SQL Server (see the messages above)." }

# ---------------------------------------------------------------------------
Step "Building the Rust backend (web server + MCP client) and the mcp-cli test tool"
cargo build --release --manifest-path "client-rust\Cargo.toml" -p spil-backend -p mcp-client
if ($LASTEXITCODE -ne 0) { Fail "cargo build failed. Make sure Rust uses the MSVC toolchain and VS Build Tools (C++) are installed." }
New-Item -ItemType Directory -Force (Join-Path $Root "dist") | Out-Null
Copy-Item "client-rust\target\release\spil-backend.exe" (Join-Path $Root "dist\spil-backend.exe") -Force
Copy-Item "client-rust\target\release\mcp-cli.exe" (Join-Path $Root "dist\mcp-cli.exe") -Force

# ---------------------------------------------------------------------------
Step "Settings (config\server.env) and accounts (data\users.db)"
$ConfigDir = Join-Path $Root "config"
$EnvFile   = Join-Path $ConfigDir "server.env"
$OldEnv    = Join-Path $Root "backend\.env"
$UsersDb   = Join-Path $Root "data\users.db"
$OldUsers  = Join-Path $Root "backend\data\users.db"
New-Item -ItemType Directory -Force $ConfigDir, (Join-Path $Root "data") | Out-Null

# Move the existing accounts from the old Python backend (same file format - nothing to convert).
if ((Test-Path $OldUsers) -and -not (Test-Path $UsersDb)) {
    Copy-Item $OldUsers $UsersDb
    Write-Host "Existing accounts copied: backend\data\users.db -> data\users.db" -ForegroundColor Green
}
# Move the existing settings (same keys; the JWT secret is kept so nobody is signed out).
if ((Test-Path $OldEnv) -and -not (Test-Path $EnvFile)) {
    Copy-Item $OldEnv $EnvFile
    Write-Host "Existing settings copied: backend\.env -> config\server.env" -ForegroundColor Green
}

$argsJson = ConvertTo-Json -Compress @("--connection", $ConnectionString, "--rules", ($RulesFile -replace '\\', '/'))
if (Test-Path $EnvFile) {
    # Keep secrets and settings; only point the backend at the (re)built server, SQL Server and the account file.
    $lines = @(Get-Content $EnvFile | Where-Object { $_ -notmatch '^(MCP_SERVER_(COMMAND|ARGS)|USERS_DB_PATH|FRONTEND_DIST)=' })
    $lines += "MCP_SERVER_COMMAND=$ServerExe"
    $lines += "MCP_SERVER_ARGS=$argsJson"
    $lines += "USERS_DB_PATH=data/users.db"
    $lines += "FRONTEND_DIST=frontend/dist"
    $lines | Set-Content -Encoding UTF8 $EnvFile
    Write-Host "Updated config\server.env (secrets and other settings unchanged)."
} else {
    $bytes = New-Object byte[] 48
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $secret = [Convert]::ToBase64String($bytes)

    $adminLines = ""
    if (-not (Test-Path $UsersDb)) {
        $adminUser = Read-Host "Choose the administrator username [admin]"
        if (-not $adminUser) { $adminUser = "admin" }
        do {
            $secure = Read-Host "Choose the administrator password (10+ characters, letters and numbers)" -AsSecureString
            $plain = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure))
        } while ($plain.Length -lt 10)
        $adminLines = "ADMIN_USERNAME=$adminUser`nADMIN_PASSWORD=$plain"
    }

    @"
MCP_SERVER_COMMAND=$ServerExe
MCP_SERVER_ARGS=$argsJson
MCP_REQUEST_TIMEOUT=30
MCP_STARTUP_TIMEOUT=180
JWT_SECRET=$secret
JWT_EXPIRE_MINUTES=480
$adminLines
USERS_DB_PATH=data/users.db
FRONTEND_DIST=frontend/dist
LOGIN_MAX_ATTEMPTS=5
LOGIN_WINDOW_SECONDS=300
CORS_ORIGINS=http://localhost:5173
"@ | Set-Content -Encoding UTF8 $EnvFile
    Write-Host "Created config\server.env."
    if ($adminLines) { Write-Host "After your first sign-in, delete the ADMIN_PASSWORD line from config\server.env." -ForegroundColor Yellow }
}

# ---------------------------------------------------------------------------
if (-not $SkipFrontend) {
    Step "Building the React app"
    Push-Location frontend
    npm ci
    if ($LASTEXITCODE -ne 0) { npm install }
    npm run build
    $ok = $LASTEXITCODE -eq 0
    Pop-Location
    if (-not $ok) { Fail "Frontend build failed" }
}

Step "Quick check: a test search through the Rust CLI -> STDIO -> C# server -> SQL Server"
& (Join-Path $Root "dist\mcp-cli.exe") --server $ServerExe --server-arg --connection --server-arg $ConnectionString `
    --server-arg --rules --server-arg $RulesFile --quiet search "my parcel never arrived" --top-k 3

foreach ($old in @("models", "backend")) {
    if (Test-Path (Join-Path $Root $old)) {
        Write-Host "Note: the '$old' folder is from an earlier version and is no longer used. Your accounts and settings were copied - you can delete it." -ForegroundColor Yellow
    }
}
Write-Host "`nSetup complete. Start the server with:  powershell -ExecutionPolicy Bypass -File scripts\run.ps1 -Lan" -ForegroundColor Green
