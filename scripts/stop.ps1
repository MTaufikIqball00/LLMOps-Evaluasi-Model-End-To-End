# LLMOps Control Center — PowerShell Shutdown Script
$pidFile = Join-Path $PSScriptRoot '..\.pids'

Write-Host '==========================================================' -ForegroundColor Cyan
Write-Host '   LLMOps Control Center — Stopping Services...           ' -ForegroundColor Cyan
Write-Host '==========================================================' -ForegroundColor Cyan

if (Test-Path $pidFile) {
    $pids = Get-Content $pidFile
    foreach ($p in $pids) {
        $p = $p.Trim()
        if ($p -match '^\d+$') {
            $proc = Get-Process -Id ([int]$p) -ErrorAction SilentlyContinue
            if ($proc) {
                Write-Host "Terminating process PID $p ($($proc.ProcessName))..." -ForegroundColor Yellow
                Stop-Process -Id ([int]$p) -Force -ErrorAction SilentlyContinue
            } else {
                Write-Host "Process PID $p is already stopped." -ForegroundColor DarkGray
            }
        }
    }
    Remove-Item $pidFile -Force -ErrorAction SilentlyContinue
    Write-Host '[OK] Saved processes terminated and .pids cleared.' -ForegroundColor Green
} else {
    Write-Host 'No active .pids file found. Checking ports 8210 and 3210...' -ForegroundColor Yellow
    foreach ($port in @(8210, 3210)) {
        $conn = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
        if ($conn) {
            $owningPid = $conn.OwningProcess
            Write-Host "Stopping process PID $owningPid on port $port..." -ForegroundColor Yellow
            Stop-Process -Id $owningPid -Force -ErrorAction SilentlyContinue
        }
    }
}

Write-Host '[OK] All LLMOps Control Center services stopped cleanly.' -ForegroundColor Green
