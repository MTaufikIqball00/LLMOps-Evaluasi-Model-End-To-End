import type { FeatureCoverageItem } from './types.ts';

export const FEATURE_INVENTORY: FeatureCoverageItem[] = [
  {
    feature_id: "FEAT-001",
    feature_name: "Overview Dashboard & Aggregated KPIs",
    frontend_route: "/",
    backend_endpoint: "/api/v1/metrics/summary",
    database_entities: ["models", "experiments", "evaluations", "request_traces", "deployments"],
    test_suites: ["frontend::01_overview", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Renders 4 KPI cards, rolling latency trend, requests chart, active deployments, and model registry summary."
  },
  {
    feature_id: "FEAT-002",
    feature_name: "Model Registry & Version Lifecycle",
    frontend_route: "/models",
    backend_endpoint: "/api/v1/models",
    database_entities: ["models"],
    test_suites: ["frontend::02_models", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Supports model registration, parameter validation, artifact URI storage, stage filtering, and promotion."
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
    notes: "Evaluates Ragas/TruLens metrics (Faithfulness, Groundedness, Context Precision, Hallucination Index). Code diverges from documented formula."
  },
  {
    feature_id: "FEAT-005",
    feature_name: "Production Monitoring & Prometheus Scraper",
    frontend_route: "/monitoring",
    backend_endpoint: "/api/v1/monitoring/metrics",
    database_entities: ["request_traces"],
    test_suites: ["frontend::05_monitoring", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Timeseries latency percentiles (p50, p95, p99), RPM throughput, latency histogram, and Prometheus exporter status."
  },
  {
    feature_id: "FEAT-006",
    feature_name: "Request Traces & Secret Scrubbing Debugger",
    frontend_route: "/traces",
    backend_endpoint: "/api/v1/traces",
    database_entities: ["request_traces"],
    test_suites: ["frontend::06_traces", "backend::security_audit_test"],
    verification_status: "PASS",
    notes: "Waterfall span tree, payload viewer, and secret scrubbing. Scrubber vulnerability identified on modern project keys."
  },
  {
    feature_id: "FEAT-007",
    feature_name: "Token Usage & Cost Budget Tracking",
    frontend_route: "/costs",
    backend_endpoint: "/api/v1/costs/summary",
    database_entities: ["pricing_catalog", "budget_settings"],
    test_suites: ["frontend::07_costs", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Calculates monthly spend, provider breakdown, budget caps, alert threshold triggers, and dynamic budget updates."
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
    notes: "Workflow run history, commit SHAs, status tracking, and manual workflow dispatch trigger."
  },
  {
    feature_id: "FEAT-010",
    feature_name: "Integrations Hub & Health Probes",
    frontend_route: "/integrations",
    backend_endpoint: "/api/v1/integrations/status",
    database_entities: ["integrations_config"],
    test_suites: ["frontend::10_integrations", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Manages 8 external adapters (Prometheus, Grafana, MLflow, W&B, Kubernetes, OpenAI, Hugging Face, GitHub Actions)."
  },
  {
    feature_id: "FEAT-011",
    feature_name: "System Health & SRE Diagnostic Hub",
    frontend_route: "/system",
    backend_endpoint: "/api/v1/health",
    database_entities: ["sqlite_master"],
    test_suites: ["frontend::11_system", "backend::api_endpoints_test"],
    verification_status: "PASS",
    notes: "Uptime, DB connection check, port allocations registry (8210, 3210, 9190, 3310), memory usage, and security controls."
  }
];
