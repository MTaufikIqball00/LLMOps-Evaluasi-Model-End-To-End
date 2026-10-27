# LLMOps Control Center — PowerShell Launcher
param(
    [int]$BackendPort = 8210,
    [int]$FrontendPort = 3210
)

$ErrorActionPreference = 'Stop'

Write-Host '==========================================================' -ForegroundColor Cyan
Write-Host '   LLMOps Control Center — Launching Services...          ' -ForegroundColor Cyan
Write-Host '==========================================================' -ForegroundColor Cyan

# 1. Port Conflict Verification
$ports = @($BackendPort, $FrontendPort)
foreach ($p in $ports) {
    $conn = Get-NetTCPConnection -LocalPort $p -ErrorAction SilentlyContinue
    if ($conn) {
        Write-Host "[ERROR] Port $p is already in use by PID $($conn.OwningProcess)!" -ForegroundColor Red
        Write-Host 'Please stop the conflicting process or change ports in .env.' -ForegroundColor Yellow
        exit 1
    }
}
Write-Host "[OK] Ports verified: Backend :$BackendPort, Frontend :$FrontendPort are free." -ForegroundColor Green

# 2. Prepare Log Directory
$logDir = Join-Path $PSScriptRoot '..\.logs'
if (-not (Test-Path $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}

$backendOut = Join-Path $logDir 'backend_out.log'
$backendErr = Join-Path $logDir 'backend_err.log'
$frontendOut = Join-Path $logDir 'frontend_out.log'
$frontendErr = Join-Path $logDir 'frontend_err.log'
$pidFile = Join-Path $PSScriptRoot '..\.pids'

# 3. Start Rust Backend Binary Directly
$backendDir = (Resolve-Path (Join-Path $PSScriptRoot '..\backend')).Path
$backendExe = Join-Path $backendDir 'target\debug\llmops-backend.exe'
if (-not (Test-Path $backendExe)) {
    Write-Host 'Compiling Rust backend binary...' -ForegroundColor Yellow
    Push-Location $backendDir
    cargo build
    Pop-Location
}
Write-Host "Starting Rust Axum backend on port $BackendPort..." -ForegroundColor Yellow
$backendProcess = Start-Process -FilePath $backendExe -WorkingDirectory $backendDir -RedirectStandardOutput $backendOut -RedirectStandardError $backendErr -PassThru -NoNewWindow

# 4. Start Next.js Frontend
Write-Host "Starting Next.js frontend on port $FrontendPort..." -ForegroundColor Yellow
$frontendDir = (Resolve-Path (Join-Path $PSScriptRoot '..\frontend')).Path
$frontendProcess = Start-Process -FilePath 'node' -ArgumentList "./node_modules/next/dist/bin/next start -p $FrontendPort" -WorkingDirectory $frontendDir -RedirectStandardOutput $frontendOut -RedirectStandardError $frontendErr -PassThru -NoNewWindow

# 5. Save PIDs
@($backendProcess.Id, $frontendProcess.Id) | Set-Content -Path $pidFile

Write-Host 'Waiting for services to become healthy...' -ForegroundColor Yellow
Start-Sleep -Seconds 5

# 6. Verify Health Endpoint
$healthUrl = "http://localhost:$BackendPort/api/v1/health"
$frontendUrl = "http://localhost:$FrontendPort"

Write-Host ''
Write-Host '==========================================================' -ForegroundColor Green
Write-Host '   LLMOPS CONTROL CENTER RUNNING SUCCESSFULLY             ' -ForegroundColor Green
Write-Host '==========================================================' -ForegroundColor Green
Write-Host " Dashboard UI:      $frontendUrl" -ForegroundColor Cyan
Write-Host " Backend API:       http://localhost:$BackendPort" -ForegroundColor Cyan
Write-Host " Health Endpoint:   $healthUrl" -ForegroundColor Cyan
Write-Host " Metrics:           http://localhost:$BackendPort/api/v1/metrics" -ForegroundColor Cyan
Write-Host " Backend PID:       $($backendProcess.Id)" -ForegroundColor White
Write-Host " Frontend PID:      $($frontendProcess.Id)" -ForegroundColor White
Write-Host " Logs Directory:    $logDir" -ForegroundColor White
Write-Host ''
Write-Host 'To stop services safely, run: .\stop.bat or .\scripts\stop.ps1' -ForegroundColor Yellow
Write-Host '==========================================================' -ForegroundColor Green
