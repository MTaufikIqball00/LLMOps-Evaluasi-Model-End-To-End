# 🚀 LLMOps Control Center — Enterprise Platform & Model Evaluation Pipeline

[![Platform: Rust](https://img.shields.io/badge/Backend-Rust%201.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Framework: Axum](https://img.shields.io/badge/HTTP%20Framework-Axum%200.7-blue.svg?style=flat-square)](https://github.com/tokio-rs/axum)
[![Frontend: Next.js 14](https://img.shields.io/badge/Frontend-Next.js%2014%20App%20Router-black.svg?style=flat-square&logo=next.js)](https://nextjs.org/)
[![Styling: Tailwind CSS](https://img.shields.io/badge/Styling-Tailwind%20CSS%203.4-38bdf8.svg?style=flat-square&logo=tailwind-css)](https://tailwindcss.com/)
[![Database: SQLite (Zero-Setup)](https://img.shields.io/badge/Persistence-SQLite%20(SQLx)-003B57.svg?style=flat-square&logo=sqlite)](https://sqlite.org/)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square)](LICENSE)

An enterprise-grade, end-to-end LLMOps and Model Evaluation Platform engineered from scratch with a high-performance **Rust (Axum + Tokio + SQLx)** backend, a modern **Next.js 14 + TypeScript + Tailwind CSS** frontend, zero-dependency embedded persistence, and native algorithmic quality and hallucination evaluation engines.

Repository: **[https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End](https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End)**

---

## 📑 Daftar Isi / Table of Contents
1. [Ringkasan Eksekutif (Indonesian Summary)](#1-ringkasan-eksekutif-indonesian-summary)
2. [Executive Overview & Capabilities](#2-executive-overview--capabilities)
3. [Arsitektur Sistem (System Architecture)](#3-arsitektur-sistem-system-architecture)
4. [Matriks Isolasi Port (Port Isolation Matrix)](#4-matriks-isolasi-port-port-isolation-matrix)
5. [Teknologi Stack (Technology Stack)](#5-teknologi-stack-technology-stack)
6. [Panduan Instalasi & Menjalankan (Quick Start Guide)](#6-panduan-instalasi--menjalankan-quick-start-guide)
7. [Struktur Modul & Navigasi Antarmuka (Navigation & Features)](#7-struktur-modul--navigasi-antarmuka-navigation--features)
8. [Engine Evaluasi & Deteksi Halusinasi (Ragas & TruLens Scoring)](#8-engine-evaluasi--deteksi-halusinasi-ragas--trulens-scoring)
9. [Keamanan, Redaksi Rahasia & Observabilitas (Security & SRE)](#9-keamanan-redaksi-rahasia--observabilitas-security--sre)
10. [REST API Reference & OpenAPI Endpoints](#10-rest-api-reference--openapi-endpoints)
11. [Troubleshooting & Solusi Kendala](#11-troubleshooting--solusi-kendala)

---

## 1. Ringkasan Eksekutif (Indonesian Summary)

**LLMOps Control Center** adalah platform enterprise terpadu untuk mengelola seluruh siklus hidup model bahasa besar (*Large Language Models*):
- **Pelacakan Eksperimen**: Menampilkan runs, hyperparameter, kurva loss, dan perplexity dari MLflow, W&B, maupun native runner dengan fitur komparasi *diff* berdampingan.
- **Model Registry & Staging**: Mengelola versi model (70B, 32B, 16B), checksum bobot, artefak S3, serta mempromosikan stage (*Development* → *Staging* → *Production* → *Archived*).
- **Evaluasi Kualitas & Deteksi Halusinasi**: Engine berbasis Ragas & TruLens (*Faithfulness*, *Answer Relevance*, *Context Precision*, *Context Recall*, dan *Hallucination Index*).
- **Observabilitas Produksi Real-Time**: Eksposisi metrik Prometheus format open metrics (`/api/v1/metrics`), tracking persentil latensi (p50, p95, p99), dan throughput (RPM).
- **Inspeksi Distributed Traces**: Waterfall spans, viewer prompt/completion dengan proteksi sensor token rahasia otomatis (*Secret Scrubbing*).
- **Analisis Biaya & Token**: Kalkulator biaya multi-provider (OpenAI, Anthropic, DeepSeek, Local vLLM), kuota anggaran bulanan, dan sistem peringatan *budget threshold*.
- **Manajemen Serving Kubernetes**: Monitoring pod vLLM / TGI, utilisasi CPU/VRAM GPU, auto-scaling, dan aksi *rolling restart*.
- **CI/CD Quality Gates**: Eksekusi benchmark regresi otomatis pada pipeline GitHub Actions sebelum model di-deploy ke production.

---

## 2. Executive Overview & Capabilities

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   LLMOps Control Center                                         │
├─────────────────────┬──────────────────────┬──────────────────────┬─────────────────────────────┤
│   Model Registry    │   Experiment Runs    │   LLM Evaluations    │   Production Observability  │
│  - Version staging  │  - MLflow/W&B sync   │  - Faithfulness      │  - Prometheus metrics       │
│  - Lineage tracking │  - Side-by-side diff │  - Hallucination idx │  - Latency p50/p95/p99      │
│  - Artifact storage │  - Loss & Perplexity │  - Ragas / TruLens   │  - Grafana integration      │
├─────────────────────┼──────────────────────┼──────────────────────┼─────────────────────────────┤
│   Distributed Trace │   Token & Budgets    │   Workload Serving   │      CI/CD Automation       │
│  - Waterfall spans  │  - Provider catalogs │  - Kubernetes pods   │  - GitHub Actions probes    │
│  - Secret scrubbing │  - Monthly ceilings  │  - VRAM utilization  │  - Automated eval gates     │
│  - Error breakdown  │  - Projected costs   │  - Rolling restarts  │  - Automated regression     │
└─────────────────────┴──────────────────────┴──────────────────────┴─────────────────────────────┘
```

---

## 3. Arsitektur Sistem (System Architecture)

Platform ini mengusung arsitektur monorepo modular:
- **Rust Axum**: Berperan sebagai primary backend untuk orkestrasi bisnis, validasi tipe yang ketat (*compile-time safety*), background worker Tokio, dan eksposisi metrik.
- **Next.js 14 (App Router)**: Berperan sebagai UI enterprise modern dengan styling Tailwind CSS dan zero-dependency dynamic SVG visualizer.
- **Embedded SQLite / PostgreSQL (SQLx AnyPool)**: Mendukung SQLite zero-setup (mode WAL & Foreign Keys otomatis) sekaligus **PostgreSQL** via `DATABASE_URL` untuk deployment produksi terdistribusi.

```
                                ┌─────────────────────────────────────────┐
                                │       Next.js 14 Frontend UI            │
                                │       Dedicated Port: 3210              │
                                └────────────────────┬────────────────────┘
                                                     │ HTTP / CORS Restricted
                                ┌────────────────────▼────────────────────┐
                                │        Rust Axum Backend API            │
                                │        Dedicated Port: 8210             │
                                └───┬────────────────┬─────────────────┬──┘
                                    │                │                 │
         ┌──────────────────────────┴─────┐          │                 └──────────────────────────┐
         │ Persistent SQLite Database     │          │ Integration Probe Adapters               │ Prometheus Metrics Exporter
         │ data/llmops.db (Zero-Setup)    │          ├─ MLflow Tracking (:5500)                 │ /api/v1/metrics
         │ - Models & Experiments         │          ├─ Weights & Biases API                    └──────────────────────────
         │ - Eval Suites & Traces         │          ├─ Prometheus Server (:9190)
         │ - Pricing & Deployments        │          ├─ Grafana Dashboards (:3310)
         └────────────────────────────────┘          ├─ Native Ragas & TruLens Engine
                                                     ├─ Kubernetes Cluster API
                                                     └─ GitHub Actions API
```

---

## 4. Matriks Isolasi Port (Port Isolation Matrix)

Seluruh port telah dikonfigurasi terisolasi dan divalidasi ketersediaannya untuk menghindari bentrok dengan servis bawaan developer lain:

| Komponen / Servis | Port Khusus | Protokol | Binding Host | Deskripsi / Fungsi |
| :--- | :---: | :---: | :---: | :--- |
| **Frontend Web UI** | `3210` | HTTP | `localhost:3210` | Antarmuka dashboard kontrol Next.js |
| **Backend REST API** | `8210` | HTTP | `0.0.0.0:8210` | Axum HTTP Server & Tokio Background Workers |
| **Prometheus Exporter** | `8210` | HTTP | `0.0.0.0:8210/metrics` | Endpoint scraping metrik OpenMetrics format |
| **Prometheus Target (Ops)** | `9190` | HTTP | `0.0.0.0:9190` | Server Prometheus lokal opsional |
| **Grafana Target (Ops)** | `3310` | HTTP | `0.0.0.0:3310` | Server Grafana lokal opsional |
| **MLflow Target (Ops)** | `5500` | HTTP | `0.0.0.0:5500` | Tracking server MLflow opsional |
| **PostgreSQL (Opsional)** | `55432` | TCP | `localhost:55432` | Database container (default memakai SQLite) |

---

## 5. Teknologi Stack (Technology Stack)

### Backend (Rust)
- **Rust Toolchain**: Rust 1.80+ (Stable Edition)
- **Web Framework**: Axum 0.7 + Tower-HTTP 0.6 (CORS Multi-Origin & `*.vercel.app`, `TimeoutLayer`, Sliding-Window Rate Limiting, Optional `ADMIN_API_KEY` Auth)
- **Asynchronous Engine**: Tokio 1.40 (Multi-threaded runtime & scheduled evaluation/alert workers)
- **Persistence Layer**: SQLx 0.8 `AnyPool` (SQLite WAL/FK otomatis + PostgreSQL native support)
- **HTTP Client**: Reqwest 0.12 dengan Rustls TLS (AI Gateway, PromQL, GitHub Actions dispatch, Slack/Discord webhooks, LLM-as-a-Judge)
- **Observability & OpenAPI**: Tracing, Secret Scrubber Regex Engine, OpenMetrics, OpenAPI 3.1.0 & Swagger UI (`/api/v1/docs`)
- **Centralized Errors**: Thiserror 2.0 dengan pemetaan status HTTP (`400`, `401`, `404`, `429`, `502`, `500`) dan payload JSON seragam

### Frontend (TypeScript & Tailwind)
- **Framework**: Next.js 14.2.15 (React 18.3.1 + App Router)
- **Styling**: Tailwind CSS 3.4.14 dengan custom enterprise tokens & dark mode
- **Ikonografi**: Lucide-React
- **Visualisasi**: Zero-dependency SVG Timeseries & Sparklines (bebas dari library bloat)
- **Komunikasi Data**: Resilient typed fetch API client dengan error boundary

---

## 6. Panduan Instalasi & Menjalankan (Quick Start Guide)

### Prasyarat
- **Rust & Cargo**: Terinstal (`cargo --version` >= 1.78)
- **Node.js**: Terinstal (`node --version` >= 18.0.0)

### 1. Kloning Repositori
```bash
git clone https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End.git
cd LLMOps-Evaluasi-Model-End-To-End
```

### 2. Salin Konfigurasi Environment
```bash
cp .env.example .env
```

### 3. Kompilasi & Build (Sekali Saja Saat Setup Pertama)
```bash
# Build binary backend Rust
cd backend
cargo build
cd ..

# Install dependencies dan build frontend Next.js
cd frontend
npm install
npm run build
cd ..
```

### 4. Menjalankan Aplikasi

#### 🖥️ Pengguna Windows (Paling Praktis)
Cukup jalankan script batch otomatis:
```cmd
start.bat
```
Untuk menghentikan:
```cmd
stop.bat
```

*(Atau via PowerShell)*:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start.ps1
# Untuk stop:
powershell -ExecutionPolicy Bypass -File .\scripts\stop.ps1
```

#### 🐧 Pengguna Linux / macOS / WSL2
```bash
chmod +x scripts/*.sh
./scripts/start.sh

# Untuk stop:
./scripts/stop.sh
```

### 5. Akses Antarmuka Web
Setelah servis aktif:
- **Control Center Dashboard**: [http://localhost:3210](http://localhost:3210)
- **Prompt Registry & A/B Diff**: [http://localhost:3210/prompts](http://localhost:3210/prompts)
- **Alerting & Webhooks**: [http://localhost:3210/alerts](http://localhost:3210/alerts)
- **Autonomous QA Engine UI**: [http://localhost:3210/qa](http://localhost:3210/qa)
- **Rust Backend API**: [http://localhost:8210/api/v1](http://localhost:8210/api/v1)
- **Interactive Swagger Docs (OpenAPI)**: [http://localhost:8210/api/v1/docs](http://localhost:8210/api/v1/docs)
- **Health Check Endpoint**: [http://localhost:8210/api/v1/health](http://localhost:8210/api/v1/health)
- **Prometheus Metrics**: [http://localhost:8210/api/v1/metrics](http://localhost:8210/api/v1/metrics)

---

## 7. Struktur Modul & Navigasi Antarmuka (Navigation & Features)

Aplikasi memiliki 14 halaman navigasi utama yang aktif:

| # | Rute | Nama Fitur | Deskripsi Fungsionalitas |
| :-: | :--- | :--- | :--- |
| 1 | `/` | **Overview Dashboard** | KPI throughput, real p50/p95/p99 latency percentiles dari database, distribusi request model, dan status pipeline. |
| 2 | `/models` | **Model Registry** | Inventaris model, parameter (70B, 32B), checksum bobot, dan workflow promosi stage. |
| 3 | `/prompts` | **Prompt Registry** | Manajemen versi template prompt, deteksi variabel `{{var}}`, stage promosi, dan perbandingan A/B diff. |
| 4 | `/experiments` | **Experiments** | Tracking hyperparameter, perbandingan loss/perplexity, dan visualisasi side-by-side run diff. |
| 5 | `/evaluations` | **LLM Evaluations** | Suite evaluasi Ragas/TruLens (dan LLM-as-a-Judge opsional): Faithfulness, Answer Relevance, Context Precision, dan Hallucination Index. |
| 6 | `/monitoring` | **Production Monitoring** | Latensi percentiles riil (PromQL live atau bucket `request_traces`), throughput (RPM), histogram latensi, dan Grafana URL terkonfigurasi. |
| 7 | `/traces` | **Traces & Requests** | Waterfall spans, token & biaya dinamis dari `pricing_catalog`, payload viewer dengan secret scrubbing menyeluruh. |
| 8 | `/costs` | **Token Usage & Costs** | Biaya per model/provider, akumulasi spend bulanan atomik, batas anggaran, dan alert spend limit. |
| 9 | `/alerts` | **Alerting & Webhooks** | Guardrail SLA (Hallucination, p95 latency, Budget, Error Rate) dengan webhook Slack/Discord/JSON & log insiden. |
| 10 | `/deployments` | **Deployments** | Monitoring pod Kubernetes inference (vLLM/TGI), VRAM GPU, dan pemicu rolling restart. |
| 11 | `/cicd` | **CI/CD Pipelines** | Status pipeline GitHub Actions, commit SHA, durasi eksekusi, dan real `workflow_dispatch` saat `GITHUB_TOKEN` tersedia. |
| 12 | `/integrations` | **Integrations Hub** | Uji probe konektivitas langsung untuk MLflow, W&B, Prometheus, Grafana, Ragas, K8s, dan GitHub Actions. |
| 13 | `/system` | **System Health** | Status uptime proses, health pool database SQLite WAL / PostgreSQL, alokasi port, dan postur keamanan SRE. |
| 14 | `/qa` | **QA & Audit Engine** | Dashboard pengujian otonom, matriks cakupan fitur, root-cause analyzer, dan ekspor laporan (JSON/CSV). |

---

## 8. Engine Evaluasi & Deteksi Halusinasi (Ragas & TruLens Scoring)

Engine evaluasi lokal Rust mengimplementasikan algoritma penilaian kualitas respons secara terukur:

$$ \text{Hallucination Index} = 1.0 - \left( 0.40 \cdot \text{Faithfulness} + 0.35 \cdot \text{Groundedness} + 0.25 \cdot \text{Context Precision} \right) $$

- **Faithfulness (Skor 0.0 - 1.0)**: Mengukur kesesuaian klaim faktual dalam respons terhadap konteks yang diberikan.
- **Answer Relevance (Skor 0.0 - 1.0)**: Mengukur seberapa tepat respons menjawab esensi pertanyaan prompt.
- **Context Precision (Skor 0.0 - 1.0)**: Mengukur densitas informasi yang relevan di dalam chunk retrieval.
- **Context Recall (Skor 0.0 - 1.0)**: Mengukur persentase fakta kunci konteks yang berhasil dikutip.
- **TruLens Groundedness**: Verifikasi pernyataan tanpa dasar (*ungrounded assertions*). Jika di bawah ambang batas (default: `0.75`), sistem menandai evaluasi sebagai `Flagged for Hallucination`.

---

## 9. Keamanan, Redaksi Rahasia & Observabilitas (Security & SRE)

- **Automated Secret Scrubbing**: Engine regex di [`src/observability/mod.rs`](backend/src/observability/mod.rs) mendeteksi dan menyensor token modern ber-hyphen OpenAI (`sk-proj-...`), Anthropic (`sk-ant-...`), GitHub (`ghp_...` / `github_pat_...`), Slack (`xoxb-...`), Hugging Face (`hf_...`), AWS (`AKIA...`), header `Bearer ...`, serta inline password/api_key di seluruh payload (`prompt`, `completion`, `error_message`, dan `metadata`) menjadi `[REDACTED_SECRET]`.
- **Optional Admin API Key Guard, Rate Limiting & Timeouts**: Mutasi (`POST`/`PATCH`/`PUT`/`DELETE`) dapat dilindungi via `ADMIN_API_KEY` (`X-API-Key` atau `Authorization: Bearer`), dibatasi per-menit via `RATE_LIMIT_RPM`, dan dilindungi oleh global `TimeoutLayer` (`REQUEST_TIMEOUT_SECS`).
- **Parameterized Queries & SQLite WAL/FK**: Seluruh query dinamis memanggil `adapt_sql` dan binding aman untuk mencegah SQL injection di SQLite maupun PostgreSQL; setiap koneksi SQLite otomatis mengaktifkan `PRAGMA foreign_keys = ON;` dan `PRAGMA journal_mode = WAL;`.
- **Dynamic CORS Protection**: Mendukung daftar domain dipisah koma melalui `FRONTEND_ORIGIN` termasuk wildcard subdomain `https://*.vercel.app`.
- **Graceful Shutdown**: Menangkap sinyal OS `SIGINT` / `SIGTERM` / `Ctrl+C` untuk menyelesaikan transaksi database dan menghentikan worker secara bersih.

---

## 10. REST API Reference & OpenAPI Endpoints

Semua endpoint bisnis berada di bawah prefix `/api/v1/*`. Dokumentasi interaktif Swagger UI tersedia di `GET /api/v1/docs` dan skema OpenAPI 3.1.0 JSON di `GET /api/v1/openapi.json`.

| Method | Endpoint | Deskripsi | Query Parameters / Body |
| :---: | :--- | :--- | :--- |
| `GET` | `/api/v1/health` | Status kesehatan daemon & engine DB | - |
| `GET` | `/api/v1/metrics` | Prometheus OpenMetrics exporter | - |
| `GET` | `/api/v1/openapi.json` | Spesifikasi OpenAPI 3.1.0 | - |
| `GET` | `/api/v1/docs` | Antarmuka Swagger UI interaktif | - |
| `GET` | `/api/v1/overview/stats` | Ringkasan KPI dashboard & persentil riil | - |
| `GET` | `/api/v1/models` | Daftar model di registry | `?stage=Production&framework=vLLM` |
| `POST` | `/api/v1/models` | Daftarkan model baru | Payload JSON model |
| `PATCH` | `/api/v1/models/:id/stage` | Promosikan stage model | `{"stage": "Production"}` |
| `GET` | `/api/v1/prompts` | Daftar template prompt | `?stage=Production&slug=...` |
| `POST` | `/api/v1/prompts` | Simpan versi template prompt baru | Payload JSON prompt |
| `PATCH` / `DELETE` | `/api/v1/prompts/:id` | Ubah stage/template atau hapus versi | `{"stage": "Production"}` |
| `GET` | `/api/v1/prompts/compare` | Perbandingan A/B & line diff prompt | `?a=prm-rag-v1&b=prm-rag-v2` |
| `GET` | `/api/v1/experiments` | Daftar eksperimen & runs | `?model_id=...&framework=...` |
| `POST` | `/api/v1/experiments` | Catat eksperimen baru | Payload JSON eksperimen |
| `GET` | `/api/v1/experiments/compare` | Perbandingan eksperimen berdampingan | `?runs=id1,id2` |
| `GET` | `/api/v1/evaluations` | Daftar evaluasi & benchmark | - |
| `POST` | `/api/v1/evaluations` | Jalankan evaluasi (Native / LLM Judge) | `{"name": "...", "use_llm_judge": true}` |
| `GET` | `/api/v1/evaluations/:id` | Detail evaluasi & hasil test case | - |
| `GET` | `/api/v1/monitoring/metrics` | Deret waktu latensi (PromQL / SQL Traces) | - |
| `GET` | `/api/v1/monitoring/latency-distribution` | Bucket histogram latensi | - |
| `GET` / `POST` | `/api/v1/traces` | Riwayat / ingest trace + kalkulasi biaya atomik | `?model=...&provider=...&status=200` |
| `POST` | `/api/v1/gateway/chat/completions` | AI Gateway OpenAI-compatible + auto trace/cost | `{"model": "gpt-4o", "messages": [...]}` |
| `GET` | `/api/v1/costs/summary` | Ringkasan biaya & proyeksi bulanan | - |
| `GET` | `/api/v1/costs/comparison` | Perbandingan efisiensi token Sebelum (Baseline) vs Sesudah (LLMOps Control Guard) | `?source=auto\|database\|transcripts` |
| `GET` | `/api/v1/costs/pricing` | Katalog tarif token per model/provider | - |
| `PATCH` | `/api/v1/costs/budget` | Perbarui batas anggaran & threshold alert | `{"monthly_budget_usd": 3000, "alert_threshold_pct": 80}` |
| `GET` / `POST` | `/api/v1/alerts/rules` | Daftar & buat rule alerting SLA | `metric_type`, `operator`, `threshold` |
| `PATCH` / `DELETE` | `/api/v1/alerts/rules/:id` | Aktifkan/jeda/hapus rule alerting | `{"is_enabled": true}` |
| `POST` | `/api/v1/alerts/rules/:id/test` | Kirim webhook uji coba Slack/Discord/JSON | - |
| `POST` | `/api/v1/alerts/evaluate` | Evaluasi semua rule SLA sekarang | - |
| `GET` | `/api/v1/alerts/events` | Riwayat insiden & status pengiriman webhook | - |
| `GET` | `/api/v1/deployments` | Status pod & serving Kubernetes | - |
| `POST` | `/api/v1/deployments/:id/restart` | Rolling restart pod serving | - |
| `GET` | `/api/v1/cicd/workflows` | Riwayat pipeline CI/CD | - |
| `POST` | `/api/v1/cicd/workflows/trigger` | Trigger GitHub `workflow_dispatch` atau lokal | `{"workflow_name": "...", "branch": "main"}` |
| `GET` | `/api/v1/integrations` | Status probe semua 8 integrasi | - |

---

## 11. Troubleshooting & Solusi Kendala

### Port Sudah Digunakan (Port Conflict)
Jika port `8210` atau `3210` telah dipakai oleh aplikasi lain:
1. Buka file `.env`.
2. Ubah `SERVER_PORT=8210` dan `FRONTEND_PORT=3210` ke nomor port lain yang bebas.
3. Jalankan kembali `start.bat`.

### Reset Database Lokal SQLite
Untuk mengembalikan database ke kondisi awal bersih (*fresh seed data*):
```powershell
# Hapus file database SQLite
Remove-Item backend/data/llmops.db -Force
# Jalankan start.bat; migrasi awal dan data seed akan otomatis dibuat kembali.
```

### Menguji Endpoint Otomatis
Jalankan skrip tes endpoint untuk memeriksa apakah seluruh 11 rute backend dan frontend merespons dengan HTTP 200:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\test_endpoints.ps1
```

---

## 📄 Lisensi
Proyek ini didistribusikan di bawah lisensi [Apache 2.0](LICENSE). Terbuka untuk pengembangan dan riset enterprise LLMOps.
