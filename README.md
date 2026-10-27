# LLMOps Control Center

An enterprise-grade, end-to-end LLMOps and Model Evaluation Platform built from scratch with a high-performance **Rust (Axum + Tokio + SQLx)** backend, an enterprise **Next.js + TypeScript + Tailwind CSS** frontend, zero-dependency embedded persistence, and native quality and hallucination evaluation engines.

---

## 1. Executive Summary & Capabilities

**LLMOps Control Center** provides a unified operational control plane for large language model operations:
- **Experiment & Run Tracking**: Discover runs, hyperparameters, and loss/perplexity metrics across MLflow, Weights & Biases, or native runners with side-by-side run diffing.
- **Model Registry & Staging Lifecycle**: Manage model architectures (70B, 32B, 16B), weights, S3/blob artifact lineages, and promote stages from *Development* to *Staging*, *Production*, and *Archived*.
- **LLM Evaluation & Hallucination Gates**: Algorithmic metrics for *Faithfulness*, *Answer Relevance*, *Context Precision*, *Context Recall*, and *TruLens Groundedness*, with automated flagged ungrounded assertion alerts.
- **Real-Time Production Observability**: Prometheus metrics exposition (`/api/v1/metrics`), latency percentile tracking (p50, p95, p99), throughput (RPM), and Grafana dashboard linkages.
- **Request Traces & Debugger**: Waterfall span inspection, prompt/completion payload viewer with automated secret scrubbing (`[REDACTED_SECRET]`), token breakdown, and error analysis.
- **Token Usage & Cost Tracking**: Provider-specific pricing catalogs, input vs output token consumption, monthly budget limits, and automated spend threshold alerts.
- **Kubernetes Serving Workloads**: Monitor inference pods (vLLM / TGI), replica auto-scaling, CPU/VRAM hardware utilization, and trigger rolling restarts.
- **CI/CD Pipelines**: Inspect GitHub Actions automated benchmark regression runs, container scans, and dispatch manual test runs.
- **Integrations Control Hub**: Active probe testing for MLflow, W&B, Prometheus, Grafana, Ragas, TruLens, Kubernetes, and GitHub Actions with transparent status reporting.
- **System Health & SRE Diagnostics**: Process uptime, SQLite database pool status, dedicated port validation, and security posture enforcement.

---

## 2. Port Isolation Matrix

All application ports are explicitly isolated and verified free to prevent collisions with existing local projects:

| Service / Component | Dedicated Port | Protocol | Binding | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Next.js Frontend UI** | `3210` | HTTP | `localhost:3210` | Enterprise Control Center Dashboard |
| **Rust Backend API** | `8210` | HTTP | `0.0.0.0:8210` | Axum REST API, Scrapers & Evaluation Runner |
| **Prometheus Target** | `9190` | HTTP | `0.0.0.0:9190` | Optional external metrics scraping server |
| **Grafana Target** | `3310` | HTTP | `0.0.0.0:3310` | Optional external dashboard suite |
| **MLflow Target** | `5500` | HTTP | `0.0.0.0:5500` | Optional external tracking server |
| **PostgreSQL (Optional)** | `55432` | TCP | `localhost:55432` | Containerized database (SQLite is default) |

---

## 3. Technology Stack

### Backend
- **Language**: Stable Rust (Rust 1.80+)
- **HTTP Framework**: Axum 0.7 with Tokio async runtime
- **Persistence**: SQLx 0.8 with embedded ACID SQLite (`data/llmops.db`) — runs with zero external container requirements
- **HTTP Client**: Reqwest 0.12 with Rustls TLS
- **Logging & Tracing**: Tracing & Tracing-Subscriber with JSON/ANSI formatters
- **Error Handling**: Thiserror 2.0 with strongly-typed `AppError` and structured JSON responses
- **Serialization**: Serde & Serde JSON

### Frontend
- **Framework**: Next.js 14 with TypeScript
- **Styling**: Tailwind CSS v3 with enterprise dark mode palette
- **Icons**: Lucide-React
- **Visualization**: Zero-dependency responsive SVG timeseries & sparkline charts
- **Design System**: Restrained enterprise cards, semantic status badges, and accessible data hierarchy

---

## 4. Quick Start & Launcher Scripts

### Prerequisites
- Rust toolchain (`cargo` and `rustc`): Installed
- Node.js (v18+) and npm: Installed

### First-Time Setup
```bash
# 1. Clone or navigate to the repository
cd "Pipeline LLMOps & Evaluasi Model End-to-End"

# 2. Copy environment configuration
cp .env.example .env

# 3. Build backend binary
cd backend
cargo build
cd ..

# 4. Install frontend dependencies & compile production build
cd frontend
npm install
npm run build
cd ..
```

### Running on Windows
To start both the Rust backend and Next.js frontend with port verification and background logging:
```cmd
start.bat
```
*(Or in PowerShell: `powershell -ExecutionPolicy Bypass -File .\scripts\start.ps1`)*

To safely stop only the services started:
```cmd
stop.bat
```
*(Or in PowerShell: `powershell -ExecutionPolicy Bypass -File .\scripts\stop.ps1`)*

### Running on Linux / macOS / WSL2
```bash
# Make scripts executable
chmod +x scripts/*.sh

# Start services
./scripts/start.sh

# Stop services
./scripts/stop.sh
```

### Automated Live Verification
Verify all 11 routes and API endpoints in a single automated command:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\test_endpoints.ps1
```

---

## 5. Application Navigation & Features

| # | Route | Feature Name | Description |
| :-: | :--- | :--- | :--- |
| 1 | `/` | **Overview** | Executive KPI cards, latency percentiles rolling chart, model request distribution, and quality summary. |
| 2 | `/models` | **Model Registry** | Model versions, parameter counts (70B, 32B), stages (*Development*, *Staging*, *Production*, *Archived*), and promotion workflows. |
| 3 | `/experiments` | **Experiments** | Discover MLflow/W&B runs and execute side-by-side hyperparameter and loss/perplexity diffing. |
| 4 | `/evaluations` | **LLM Evaluations** | Ragas & TruLens benchmarks: Faithfulness, Groundedness, Context Precision/Recall, Hallucination detection flags, and metric definitions. |
| 5 | `/monitoring` | **Production Monitoring** | Latency percentiles (p50/p95/p99), throughput (RPM), latency histograms, and Prometheus scraper validation. |
| 6 | `/traces` | **Traces & Requests** | Detailed request waterfall spans, token breakdown, estimated cost, and automated secret redaction (`[REDACTED_SECRET]`). |
| 7 | `/costs` | **Token Usage & Costs** | Spend attribution by provider/model, monthly budget thresholds, budget alerts, and active pricing catalog. |
| 8 | `/deployments` | **Deployments** | Kubernetes serving pods, ready vs desired replicas, CPU/VRAM usage, and rolling restart actions. |
| 9 | `/cicd` | **CI/CD Pipelines** | GitHub Actions workflow runs, commit SHAs, run duration, and manual benchmark gate dispatches. |
| 10 | `/integrations` | **Integrations** | Real-time connectivity probes for MLflow, W&B, Prometheus, Grafana, Ragas, TruLens, K8s, and GitHub Actions. |
| 11 | `/system` | **System Health** | Engine uptime, database pool status, dedicated port conflict validation, and SRE security posture. |

---

## 6. Integrations Architecture & Fallback Behavior

In strict compliance with the platform guidelines:
- **Every integration actively checks its target endpoint.**
- **If an external service is offline or unconfigured, the platform reports it honestly as `Disconnected / Offline` or `Unconfigured`** and falls back to persistent local database records with clear source labeling.
- **No operational results are fabricated.**

```
                               ┌────────────────────────────────┐
                               │   Next.js 14 Frontend UI       │
                               │   Port 3210                    │
                               └───────────────┬────────────────┘
                                               │ HTTP / CORS
                               ┌───────────────▼────────────────┐
                               │   Rust Axum Backend API        │
                               │   Port 8210                    │
                               └───┬──────────────┬───────────┬─┘
                                   │              │           │
         ┌─────────────────────────┴────┐         │           └──────────────────────────┐
         │ Persistent SQLite DB         │         │ Active Integration Adapters          │ Prometheus Metrics Exporter
         │ data/llmops.db (Zero-Setup)  │         ├─ MLflow Tracking (:5500)             │ /api/v1/metrics
         └──────────────────────────────┘         ├─ Weights & Biases API                └──────────────────────────
                                                  ├─ Prometheus Server (:9190)
                                                  ├─ Grafana Dashboards (:3310)
                                                  ├─ Native Ragas & TruLens Engine
                                                  ├─ Kubernetes Cluster API
                                                  └─ GitHub Actions API
```

---

## 7. Security & Reliability Controls

- **Automated Secret Scrubbing**: Ingested prompts and completions are scrubbed via regex before persistence; bearer tokens, GitHub tokens (`ghp_*`), and OpenAI keys (`sk-*`) are masked with `[REDACTED_SECRET]`.
- **CORS Isolation**: The Rust backend restricts CORS to `http://localhost:3210` and `http://127.0.0.1:3210`.
- **Bounded Concurrency & Async Runtime**: Tokio background workers process CPU/evaluation tasks asynchronously without blocking API request threads.
- **Graceful Shutdown**: Native OS signal handling (`Ctrl+C` / `SIGINT` and `SIGTERM`) drains active connections and flushes SQLite database writes.
- **Zero-Secret Commits**: All configuration templates (`.env.example`) contain placeholders only.

---

## 8. Verification & Testing Evidence

```
$ cargo test
running 3 tests
test test_prometheus_metrics_format ... ok
test test_ragas_evaluation_scoring ... ok
test test_secret_redaction ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

$ npm run build
✓ Compiled successfully
✓ Generating static pages (14/14)
✓ Finalizing page optimization
```

---

## 9. Troubleshooting

### Port Already in Use
If port `8210` or `3210` is occupied by another local project:
1. Edit `SERVER_PORT` and `FRONTEND_PORT` in `.env`.
2. Update `frontend/src/lib/api.ts` or pass `-BackendPort` and `-FrontendPort` to `scripts/start.ps1`.
3. Re-run `start.bat`.

### Database Reset
To reset the local SQLite database to its seeded state:
```powershell
Remove-Item backend/data/llmops.db -Force
# Restart backend; baseline migrations and seed data will run automatically.
```
