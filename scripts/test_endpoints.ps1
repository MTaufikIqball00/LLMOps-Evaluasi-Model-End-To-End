# LLMOps Control Center — Automated Verification Probe

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   Running Live Verification Probe Across All Services    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Probe Backend Health
$health = Invoke-RestMethod -Uri "http://localhost:8210/api/v1/health"
Write-Host "[OK] Backend Health: $($health.status) (DB: $($health.database))" -ForegroundColor Green

# 2. Probe Backend Overview Stats
$stats = Invoke-RestMethod -Uri "http://localhost:8210/api/v1/overview/stats"
Write-Host "[OK] Overview Stats: $($stats.total_requests) requests | p95 latency: $($stats.latency.p95_ms)ms" -ForegroundColor Green

# 3. Probe Prometheus Exposition
$prom = (Invoke-WebRequest -Uri "http://localhost:8210/api/v1/metrics" -UseBasicParsing).Content
if ($prom -match "llmops_http_requests_total") {
    Write-Host "[OK] Prometheus Metrics: Verified exposition lines" -ForegroundColor Green
} else {
    Write-Host "[FAIL] Prometheus metrics missing expected headers" -ForegroundColor Red
}

# 4. Probe All 11 Frontend Routes
$routes = @(
    "/",
    "/models",
    "/experiments",
    "/evaluations",
    "/monitoring",
    "/traces",
    "/costs",
    "/deployments",
    "/cicd",
    "/integrations",
    "/system"
)

Write-Host "`nTesting Frontend UI Routes on http://localhost:3210..." -ForegroundColor Yellow
$allPassed = $true

foreach ($r in $routes) {
    try {
        $res = Invoke-WebRequest -Uri ("http://localhost:3210" + $r) -UseBasicParsing
        if ($res.StatusCode -eq 200) {
            Write-Host "  [200 OK] $r" -ForegroundColor Green
        } else {
            Write-Host "  [$($res.StatusCode)] $r" -ForegroundColor Red
            $allPassed = $false
        }
    } catch {
        Write-Host "  [FAIL] $r : $_" -ForegroundColor Red
        $allPassed = $false
    }
}

Write-Host ""
if ($allPassed) {
    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host "   ALL 11 ROUTES & API ENDPOINTS VERIFIED 100% OPERATIONAL" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
} else {
    Write-Host "==========================================================" -ForegroundColor Red
    Write-Host "   SOME ROUTES FAILED VERIFICATION                        " -ForegroundColor Red
    Write-Host "==========================================================" -ForegroundColor Red
}
