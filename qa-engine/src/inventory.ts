import type { FeatureCoverageItem } from './types.ts';

export const FEATURE_INVENTORY: FeatureCoverageItem[] = [
  {
    feature_id: "FEAT-001",
    feature_name: "Overview Dashboard & Aggregated KPIs",
    frontend_route: "/",
    backend_endpoint: "/api/v1/overview/stats",
    database_entities: ["models", "experiments", "evaluations", "request_traces", "deployments"],
    test_suites: ["frontend::01_overview", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Computes real p50/p95/p99 latency percentiles directly from persisted request_traces and displays live spend/token telemetry."
  },
  {
    feature_id: "FEAT-002",
    feature_name: "Model Registry & Version Lifecycle",
    frontend_route: "/models",
    backend_endpoint: "/api/v1/models",
    database_entities: ["models"],
    test_suites: ["frontend::02_models", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Supports model registration, parameter validation, artifact URI storage, stage filtering, and lifecycle promotion."
  },
  {
    feature_id: "FEAT-003",
    feature_name: "Experiment Tracking & Hyperparameter Diffing",
    frontend_route: "/experiments",
    backend_endpoint: "/api/v1/experiments",
    database_entities: ["experiments"],
    test_suites: ["frontend::03_experiments", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Tracks runs, hyperparameters, metrics, and renders side-by-side run diff comparison table."
  },
  {
    feature_id: "FEAT-004",
    feature_name: "LLM Evaluations & Hallucination Analysis",
    frontend_route: "/evaluations",
    backend_endpoint: "/api/v1/evaluations",
    database_entities: ["evaluations", "evaluation_test_cases"],
    test_suites: ["frontend::04_evaluations", "backend::evaluation_engine_test"],
    verification_status: "PASS",
    notes: "Uses composite weighted Hallucination Index formula (0.40*F + 0.35*G + 0.25*CP) with optional LLM-as-a-Judge."
  },
  {
    feature_id: "FEAT-005",
    feature_name: "Production Monitoring & Prometheus Scraper",
    frontend_route: "/monitoring",
    backend_endpoint: "/api/v1/monitoring/metrics",
    database_entities: ["request_traces"],
    test_suites: ["frontend::05_monitoring", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Queries live Prometheus PromQL when reachable or builds real trace-bucketed rolling series; Grafana URL configurable via env."
  },
  {
    feature_id: "FEAT-006",
    feature_name: "Request Traces & Secret Scrubbing Debugger",
    frontend_route: "/traces",
    backend_endpoint: "/api/v1/traces",
    database_entities: ["request_traces"],
    test_suites: ["frontend::06_traces", "backend::security_audit_test"],
    verification_status: "PASS",
    notes: "Redacts modern hyphenated keys (sk-proj-*, sk-ant-*), AWS, HF, Slack, GitHub PATs across prompt, completion, error_message, and metadata."
  },
  {
    feature_id: "FEAT-007",
    feature_name: "Token Usage & Cost Budget Tracking",
    frontend_route: "/costs",
    backend_endpoint: "/api/v1/costs/summary",
    database_entities: ["pricing_catalog", "budget_settings"],
    test_suites: ["frontend::07_costs", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Looks up dynamic pricing_catalog rates on trace ingestion and atomically increments budget_settings.current_spend_usd."
  },
  {
    feature_id: "FEAT-008",
    feature_name: "Kubernetes Serving & Workload Deployments",
    frontend_route: "/deployments",
    backend_endpoint: "/api/v1/deployments",
    database_entities: ["deployments"],
    test_suites: ["frontend::08_deployments", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Monitors vLLM & TGI replica states, GPU memory telemetry, container status, and rolling restart dispatch."
  },
  {
    feature_id: "FEAT-009",
    feature_name: "CI/CD Pipeline Automation & Quality Gates",
    frontend_route: "/cicd",
    backend_endpoint: "/api/v1/cicd/workflows",
    database_entities: ["cicd_workflows"],
    test_suites: ["frontend::09_cicd", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Integrates live GitHub Actions workflow_dispatch API when GITHUB_TOKEN is set with local fallback."
  },
  {
    feature_id: "FEAT-010",
    feature_name: "Integrations Hub & Health Probes",
    frontend_route: "/integrations",
    backend_endpoint: "/api/v1/integrations",
    database_entities: ["integrations_config"],
    test_suites: ["frontend::10_integrations", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Manages 8 external adapters (Prometheus, Grafana, MLflow, W&B, Kubernetes, Ragas, TruLens, GitHub Actions)."
  },
  {
    feature_id: "FEAT-011",
    feature_name: "System Health & SRE Diagnostic Hub",
    frontend_route: "/system",
    backend_endpoint: "/api/v1/health",
    database_entities: ["sqlite_master"],
    test_suites: ["frontend::11_system", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Reports SQLite WAL / PostgreSQL connectivity, uptime, port registry, API key auth, rate limiting, and timeout posture."
  },
  {
    feature_id: "FEAT-012",
    feature_name: "Prompt Registry & A/B Template Diffing",
    frontend_route: "/prompts",
    backend_endpoint: "/api/v1/prompts",
    database_entities: ["prompts"],
    test_suites: ["backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Versioned templates, automatic {{variable}} detection, stage lifecycle, and A/B line diffing."
  },
  {
    feature_id: "FEAT-013",
    feature_name: "Alerting Rules & Webhook Notifier",
    frontend_route: "/alerts",
    backend_endpoint: "/api/v1/alerts/rules",
    database_entities: ["alert_rules", "alert_events"],
    test_suites: ["backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Automated SLA guards for hallucination_score, latency_p95_ms, budget_used_pct, error_rate_pct with Slack/Discord webhooks."
  },
  {
    feature_id: "FEAT-014",
    feature_name: "OpenAI-Compatible AI Gateway",
    frontend_route: "/traces",
    backend_endpoint: "/api/v1/gateway/chat/completions",
    database_entities: ["request_traces", "pricing_catalog", "budget_settings"],
    test_suites: ["backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Proxies OpenAI-compatible completions or local fallback with automatic scrubbing, trace persistence, and budget accounting."
  },
  {
    feature_id: "FEAT-015",
    feature_name: "OpenAPI 3.1 Spec & Swagger UI",
    frontend_route: "/qa",
    backend_endpoint: "/api/v1/docs",
    database_entities: [],
    test_suites: ["backend::api_endpoints_test", "frontend::13_qa_dashboard"],
    verification_status: "PASS",
    notes: "Serves interactive Swagger UI at /api/v1/docs and OpenAPI 3.1.0 schema at /api/v1/openapi.json."
  }
];
