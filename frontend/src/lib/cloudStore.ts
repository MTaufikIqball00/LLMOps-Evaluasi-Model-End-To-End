// Stateful Cloud & Edge Fallback Engine for Vercel / Serverless Deployments
// Mirrors the exact schema, baseline seed records, and business logic of the Rust Axum + SQLx Backend

const STORAGE_KEY = "llmops_control_center_cloud_state_v1";

export interface CloudState {
  models: any[];
  experiments: any[];
  pricing: any[];
  budget: any;
  evaluations: any[];
  evaluationTestCases: Record<string, any[]>;
  traces: any[];
  deployments: any[];
  workflows: any[];
  integrations: any[];
  startedAt: number;
}

const nowIso = () => new Date().toISOString();

export function getInitialCloudState(): CloudState {
  const ts = nowIso();
  return {
    startedAt: Date.now() - 1842 * 1000,
    models: [
      {
        id: "m-llama31-70b",
        name: "Llama-3.1-70B-Instruct",
        framework: "vLLM / PyTorch",
        task: "Text Generation / RAG",
        stage: "Production",
        version: "v2.1.0",
        parameters_count: "70B",
        artifact_uri: "s3://ml-artifacts/llama-3.1-70b-instruct/v2.1.0",
        description: "Meta primary enterprise reasoning and customer support agent.",
        tags: '["production", "reasoning", "fp8"]',
        created_at: ts,
        updated_at: ts,
      },
      {
        id: "m-mistral-large",
        name: "Mistral-Large-2",
        framework: "TGI / HF",
        task: "Multi-turn Dialogue",
        stage: "Staging",
        version: "v1.4.0",
        parameters_count: "123B",
        artifact_uri: "s3://ml-artifacts/mistral-large-2/v1.4.0",
        description: "High precision multilingual instruction-following model in staging candidate testing.",
        tags: '["staging", "multilingual", "candidate"]',
        created_at: ts,
        updated_at: ts,
      },
      {
        id: "m-qwen25-coder",
        name: "Qwen-2.5-Coder-32B",
        framework: "vLLM",
        task: "Code Generation",
        stage: "Development",
        version: "v0.9.3",
        parameters_count: "32.5B",
        artifact_uri: "s3://ml-artifacts/qwen-coder/v0.9.3",
        description: "Fine-tuned developer code completion engine.",
        tags: '["dev", "code", "lora"]',
        created_at: ts,
        updated_at: ts,
      },
      {
        id: "m-deepseek-v2",
        name: "DeepSeek-Coder-V2-Lite",
        framework: "vLLM",
        task: "Code Generation",
        stage: "Archived",
        version: "v1.0.0",
        parameters_count: "16B",
        artifact_uri: "s3://ml-artifacts/deepseek-v2-lite/v1.0.0",
        description: "Superseded by Qwen-2.5-Coder.",
        tags: '["archived", "legacy"]',
        created_at: ts,
        updated_at: ts,
      },
    ],
    experiments: [
      {
        id: "exp-lora-run-84",
        name: "llama31-customer-support-lora-r16",
        model_id: "m-llama31-70b",
        framework: "MLflow",
        status: "Completed",
        parameters:
          '{"learning_rate": 0.0002, "batch_size": 16, "lora_r": 16, "lora_alpha": 32, "max_seq_len": 4096, "optimizer": "adamw_8bit"}',
        metrics:
          '{"train_loss": 0.842, "eval_loss": 0.912, "perplexity": 2.48, "rouge_1": 0.642, "bleu": 42.8}',
        tags: '["fine-tuning", "customer-support", "production-candidate"]',
        duration_seconds: 4320,
        created_at: ts,
      },
      {
        id: "exp-dpo-run-12",
        name: "mistral-large-dpo-alignment-v3",
        model_id: "m-mistral-large",
        framework: "Weights & Biases",
        status: "Completed",
        parameters:
          '{"beta": 0.1, "learning_rate": 0.00005, "batch_size": 8, "loss_type": "sigmoid", "max_prompt_length": 2048}',
        metrics:
          '{"chosen_reward": 1.84, "rejected_reward": -0.62, "accuracy": 0.884, "margin": 2.46}',
        tags: '["alignment", "dpo", "safety"]',
        duration_seconds: 7200,
        created_at: ts,
      },
      {
        id: "exp-code-qlora-05",
        name: "qwen-coder-python-specialized",
        model_id: "m-qwen25-coder",
        framework: "Native",
        status: "Running",
        parameters:
          '{"learning_rate": 0.0001, "batch_size": 32, "lora_r": 32, "target_modules": "q_proj,v_proj,k_proj"}',
        metrics: '{"step": 1250, "loss": 1.140, "tokens_per_sec": 4820}',
        tags: '["code", "python", "active-job"]',
        duration_seconds: 2800,
        created_at: ts,
      },
    ],
    pricing: [
      {
        id: "p-gpt4o",
        provider: "OpenAI",
        model_pattern: "gpt-4o",
        prompt_cost_per_1k: 0.0025,
        completion_cost_per_1k: 0.01,
        last_updated: ts,
      },
      {
        id: "p-gpt4o-mini",
        provider: "OpenAI",
        model_pattern: "gpt-4o-mini",
        prompt_cost_per_1k: 0.00015,
        completion_cost_per_1k: 0.0006,
        last_updated: ts,
      },
      {
        id: "p-claude-sonnet",
        provider: "Anthropic",
        model_pattern: "claude-3-5-sonnet",
        prompt_cost_per_1k: 0.003,
        completion_cost_per_1k: 0.015,
        last_updated: ts,
      },
      {
        id: "p-llama-70b",
        provider: "Meta / Self-Hosted",
        model_pattern: "Llama-3.1-70B-Instruct",
        prompt_cost_per_1k: 0.0005,
        completion_cost_per_1k: 0.0009,
        last_updated: ts,
      },
      {
        id: "p-gemini-pro",
        provider: "Google",
        model_pattern: "gemini-1.5-pro",
        prompt_cost_per_1k: 0.00125,
        completion_cost_per_1k: 0.005,
        last_updated: ts,
      },
      {
        id: "p-mistral-large",
        provider: "Mistral AI",
        model_pattern: "mistral-large-2",
        prompt_cost_per_1k: 0.002,
        completion_cost_per_1k: 0.006,
        last_updated: ts,
      },
    ],
    budget: {
      id: "default-budget",
      monthly_budget_usd: 2500.0,
      alert_threshold_pct: 80.0,
      current_spend_usd: 1428.5,
      updated_at: ts,
    },
    evaluations: [
      {
        id: "eval-golden-rag-suite-01",
        name: "Q3 Enterprise Policy RAG Benchmark",
        model_name: "Llama-3.1-70B-Instruct",
        dataset_name: "enterprise-kb-v3-eval",
        evaluator_framework: "Ragas & TruLens Suite",
        status: "Completed",
        faithfulness: 0.942,
        answer_relevance: 0.918,
        context_precision: 0.885,
        context_recall: 0.902,
        groundedness: 0.954,
        hallucination_score: 0.046,
        avg_latency_ms: 312.4,
        total_tokens: 48200,
        test_cases_count: 25,
        created_at: ts,
      },
      {
        id: "eval-mistral-safety-02",
        name: "Multilingual Financial Compliance Audit",
        model_name: "Mistral-Large-2",
        dataset_name: "fin-compliance-100",
        evaluator_framework: "TruLens Triad",
        status: "Completed",
        faithfulness: 0.961,
        answer_relevance: 0.892,
        context_precision: 0.92,
        context_recall: 0.884,
        groundedness: 0.948,
        hallucination_score: 0.052,
        avg_latency_ms: 420.8,
        total_tokens: 62100,
        test_cases_count: 30,
        created_at: ts,
      },
      {
        id: "eval-code-test-suite-03",
        name: "HumanEval + Python Enterprise Libs",
        model_name: "Qwen-2.5-Coder-32B",
        dataset_name: "humaneval-enterprise",
        evaluator_framework: "Native Suite",
        status: "Completed",
        faithfulness: 0.889,
        answer_relevance: 0.945,
        context_precision: 0.871,
        context_recall: 0.865,
        groundedness: 0.912,
        hallucination_score: 0.088,
        avg_latency_ms: 198.5,
        total_tokens: 31400,
        test_cases_count: 40,
        created_at: ts,
      },
    ],
    evaluationTestCases: {
      "eval-golden-rag-suite-01": [
        {
          id: "tc-01",
          evaluation_id: "eval-golden-rag-suite-01",
          question: "What is the maximum reimbursement for remote workstation ergonomics per fiscal year?",
          ground_truth: "Employees may claim up to $750 per fiscal year with receipts submitted within 30 days.",
          generated_answer:
            "Under the remote work policy section 4.2, employees are eligible for up to $750 per fiscal year for ergonomic workstation accessories.",
          retrieved_context:
            "Section 4.2 Workstation Equipment: Eligible employees may claim up to $750 per fiscal year. Invoices must be submitted within 30 days of purchase.",
          faithfulness_score: 0.98,
          relevance_score: 0.96,
          groundedness_score: 0.99,
          hallucination_detected: false,
          latency_ms: 245.0,
          tokens_used: 340,
          created_at: ts,
        },
        {
          id: "tc-02",
          evaluation_id: "eval-golden-rag-suite-01",
          question: "Can temporary contractors access SOC-2 Type II customer audit logs directly?",
          ground_truth:
            "No, external contractors require VP of Security written exemption and temporary jump-host bastions.",
          generated_answer:
            "Contractors are strictly prohibited from direct access to SOC-2 logs unless an approved VP exception ticket is filed.",
          retrieved_context:
            "Security Policy Rule 18: No third-party or contractor may directly query customer data or SOC-2 evidence without an explicit VP Security exemption.",
          faithfulness_score: 0.95,
          relevance_score: 0.94,
          groundedness_score: 0.96,
          hallucination_detected: false,
          latency_ms: 310.0,
          tokens_used: 412,
          created_at: ts,
        },
        {
          id: "tc-03",
          evaluation_id: "eval-golden-rag-suite-01",
          question: "What is the data retention window for unindexed vector embeddings in the staging cluster?",
          ground_truth: "Raw chunk embeddings are retained for 90 days before cold storage archiving.",
          generated_answer: "Vector embeddings are deleted immediately after 14 days without archiving.",
          retrieved_context:
            "Data Governance Chapter 9: Embedding vectors in staging environments are retained for 90 days before automated lifecycle migration to cold S3 tier.",
          faithfulness_score: 0.42,
          relevance_score: 0.65,
          groundedness_score: 0.38,
          hallucination_detected: true,
          latency_ms: 290.0,
          tokens_used: 380,
          created_at: ts,
        },
      ],
    },
    traces: [
      {
        id: "tr-8821a",
        trace_id: "tr-20261027-001",
        span_name: "rag_orchestrator::execute",
        model_name: "Llama-3.1-70B-Instruct",
        provider: "vllm_local",
        status_code: 200,
        latency_ms: 268.4,
        prompt_tokens: 842,
        completion_tokens: 185,
        total_tokens: 1027,
        estimated_cost: 0.00058,
        redacted_prompt: "User query: Summarize security policy for [REDACTED_SECRET] rotation interval.",
        redacted_completion: "According to Policy 9.1, service credentials and API keys must be rotated every 90 days.",
        metadata: '{"env": "production", "user_tier": "enterprise", "cache_hit": false}',
        error_message: null,
        created_at: ts,
      },
      {
        id: "tr-8822b",
        trace_id: "tr-20261027-002",
        span_name: "agent_chat::stream",
        model_name: "Llama-3.1-70B-Instruct",
        provider: "vllm_local",
        status_code: 200,
        latency_ms: 312.1,
        prompt_tokens: 1250,
        completion_tokens: 320,
        total_tokens: 1570,
        estimated_cost: 0.00091,
        redacted_prompt: "Synthesize incident response timeline for ticket #49122 on database failover.",
        redacted_completion: "Primary database failover initiated at 04:12 UTC. Secondary promoted in 42 seconds.",
        metadata: '{"env": "production", "user_tier": "internal_sre"}',
        error_message: null,
        created_at: ts,
      },
      {
        id: "tr-8823c",
        trace_id: "tr-20261027-003",
        span_name: "code_assistant::complete",
        model_name: "Qwen-2.5-Coder-32B",
        provider: "vllm_local",
        status_code: 200,
        latency_ms: 145.8,
        prompt_tokens: 620,
        completion_tokens: 140,
        total_tokens: 760,
        estimated_cost: 0.00038,
        redacted_prompt: "Generate rust axum handler with timeout middleware and sqlx query.",
        redacted_completion: "pub async fn handler(State(pool): State<Pool>) -> Result<Json<Data>, AppError> { ... }",
        metadata: '{"env": "development", "ide": "vscode"}',
        error_message: null,
        created_at: ts,
      },
      {
        id: "tr-8824d",
        trace_id: "tr-20261027-004",
        span_name: "doc_analyzer::extract",
        model_name: "Mistral-Large-2",
        provider: "tgi_cluster",
        status_code: 504,
        latency_ms: 3000.0,
        prompt_tokens: 3400,
        completion_tokens: 0,
        total_tokens: 3400,
        estimated_cost: 0.0068,
        redacted_prompt: "Extract tables from 80-page financial disclosure document.",
        redacted_completion: null,
        metadata: '{"env": "staging"}',
        error_message: "Gateway Timeout: vLLM inference node exceeded 3000ms threshold",
        created_at: ts,
      },
    ],
    deployments: [
      {
        id: "dep-llama-prod",
        name: "llama-3.1-70b-vllm-serving",
        model_name: "Llama-3.1-70B-Instruct",
        version: "v2.1.0",
        environment: "Production",
        cluster: "k8s-prod-us-east-1",
        replicas_desired: 4,
        replicas_ready: 4,
        status: "Healthy",
        image_tag: "vllm/vllm-openai:v0.6.2-cuda12.4",
        cpu_usage: "62%",
        memory_usage: "48.2 GB / 64 GB",
        updated_at: ts,
      },
      {
        id: "dep-mistral-stage",
        name: "mistral-large-candidate-serving",
        model_name: "Mistral-Large-2",
        version: "v1.4.0",
        environment: "Staging",
        cluster: "k8s-staging-us-east-1",
        replicas_desired: 2,
        replicas_ready: 2,
        status: "Healthy",
        image_tag: "ghcr.io/huggingface/text-generation-inference:2.4.0",
        cpu_usage: "35%",
        memory_usage: "38.4 GB / 48 GB",
        updated_at: ts,
      },
      {
        id: "dep-qwen-dev",
        name: "qwen-coder-dev-instance",
        model_name: "Qwen-2.5-Coder-32B",
        version: "v0.9.3",
        environment: "Development",
        cluster: "k8s-dev-cluster",
        replicas_desired: 1,
        replicas_ready: 1,
        status: "Healthy",
        image_tag: "vllm/vllm-openai:v0.6.2",
        cpu_usage: "18%",
        memory_usage: "16.1 GB / 24 GB",
        updated_at: ts,
      },
    ],
    workflows: [
      {
        id: "run-1092",
        workflow_name: "Model Evaluation & Benchmark Gates",
        run_id: "gh-9281921",
        branch: "main",
        commit_sha: "8b4f12a",
        commit_message: "feat: update prompt template for RAG synthesis",
        status: "Success",
        trigger_event: "push",
        duration_seconds: 342,
        url: "https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End/actions",
        started_at: ts,
      },
      {
        id: "run-1091",
        workflow_name: "vLLM Serving Container Build & Security Scan",
        run_id: "gh-9281890",
        branch: "release/v2.1.0",
        commit_sha: "3c19e88",
        commit_message: "chore: bump base CUDA image to 12.4.1",
        status: "Success",
        trigger_event: "pull_request",
        duration_seconds: 512,
        url: "https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End/actions",
        started_at: ts,
      },
      {
        id: "run-1090",
        workflow_name: "Canary Rollout to Staging Kubernetes",
        run_id: "gh-9281745",
        branch: "staging",
        commit_sha: "1e4a902",
        commit_message: "deploy: rollout mistral-large candidate v1.4.0",
        status: "Success",
        trigger_event: "workflow_dispatch",
        duration_seconds: 184,
        url: "https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End/actions",
        started_at: ts,
      },
    ],
    integrations: [
      {
        service_name: "mlflow",
        display_name: "MLflow Tracking Server",
        category: "Experiment Tracking",
        status: "Connected (Local / Cloud Sync)",
        endpoint_url: "http://localhost:5500",
        latency_ms: 14.2,
        message: "Tracking adapter initialized. Serving experiment telemetry and run diffs.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "wandb",
        display_name: "Weights & Biases",
        category: "Experiment Tracking",
        status: "Configured (API Ready)",
        endpoint_url: "https://api.wandb.ai",
        latency_ms: 42.5,
        message: "W&B adapter ready. Synchronizing DPO and LoRA alignment runs.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "prometheus",
        display_name: "Prometheus Metrics Server",
        category: "Observability",
        status: "Connected (Exporter Active)",
        endpoint_url: "http://localhost:9190",
        latency_ms: 8.4,
        message: "OpenMetrics exposition endpoint active at /api/v1/metrics.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "grafana",
        display_name: "Grafana Dashboards",
        category: "Observability",
        status: "Configured (Dashboards Linked)",
        endpoint_url: "http://localhost:3310",
        latency_ms: 19.1,
        message: "Grafana provisioning templates linked for p50/p95/p99 latency panels.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "ragas",
        display_name: "Ragas Evaluation Engine",
        category: "LLM Evaluation",
        status: "Connected (Native Suite Active)",
        endpoint_url: "native://ragas-rust-evaluator",
        latency_ms: 4.1,
        message: "Native algorithmic evaluator active for Faithfulness, Answer Relevance, and Context Precision.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "trulens",
        display_name: "TruLens RAG Triad",
        category: "LLM Evaluation",
        status: "Connected (Native Triad Active)",
        endpoint_url: "native://trulens-triad-engine",
        latency_ms: 3.8,
        message: "Groundedness and hallucination assertion verifier operational.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "kubernetes",
        display_name: "Kubernetes Serving Cluster",
        category: "Infrastructure & Serving",
        status: "Connected (Cluster Manifests Ready)",
        endpoint_url: "https://kubernetes.default.svc",
        latency_ms: 18.6,
        message: "Monitoring vLLM and TGI inference pod replicas and GPU memory allocations.",
        is_configured: true,
        last_checked: ts,
      },
      {
        service_name: "github_actions",
        display_name: "GitHub Actions CI/CD",
        category: "CI/CD Automation",
        status: "Connected (Pipeline Active)",
        endpoint_url: "https://api.github.com/repos/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End",
        latency_ms: 64.0,
        message: "Connected to repository workflows for automated evaluation gates.",
        is_configured: true,
        last_checked: ts,
      },
    ],
  };
}

let memoryState: CloudState | null = null;

function loadState(): CloudState {
  if (typeof window !== "undefined") {
    try {
      const raw = window.sessionStorage.getItem(STORAGE_KEY);
      if (raw) {
        return JSON.parse(raw);
      }
    } catch {
      // ignore storage errors
    }
  }
  if (!memoryState) {
    memoryState = getInitialCloudState();
  }
  return memoryState;
}

function saveState(state: CloudState) {
  memoryState = state;
  if (typeof window !== "undefined") {
    try {
      window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch {
      // ignore storage errors
    }
  }
}

export function handleCloudApiRequest(endpoint: string, options?: RequestInit): any {
  const state = loadState();
  const method = (options?.method || "GET").toUpperCase();
  const body = options?.body ? JSON.parse(String(options.body)) : {};

  // Parse path and query
  const cleanEndpoint = endpoint.replace(/^https?:\/\/[^/]+/, "").replace(/^\/api\/v1/, "");
  const [pathname, queryString] = cleanEndpoint.split("?");
  const params = new URLSearchParams(queryString || "");

  // 1. GET /health
  if (pathname === "/health" && method === "GET") {
    return {
      status: "Healthy",
      version: "0.1.0",
      database: "Connected & Healthy",
      uptime_seconds: Math.floor((Date.now() - state.startedAt) / 1000),
      allocated_ports: {
        backend_api: 8210,
        frontend_ui: 3210,
        prometheus_target: 9190,
        grafana_target: 3310,
        mlflow_target: 5500,
      },
      timestamp: nowIso(),
    };
  }

  // 2. GET /overview/stats
  if (pathname === "/overview/stats" && method === "GET") {
    const total = state.traces.length;
    const successful = state.traces.filter((t) => t.status_code === 200).length;
    const failed = total - successful;
    const successRate = total > 0 ? (successful / total) * 100.0 : 100.0;

    const promptTokens = state.traces.reduce((acc, t) => acc + (t.prompt_tokens || 0), 0);
    const completionTokens = state.traces.reduce((acc, t) => acc + (t.completion_tokens || 0), 0);
    const totalTokens = state.traces.reduce((acc, t) => acc + (t.total_tokens || 0), 0);

    const modelCounts: Record<string, number> = {};
    state.traces.forEach((t) => {
      modelCounts[t.model_name] = (modelCounts[t.model_name] || 0) + 1;
    });
    const modelDistribution = Object.entries(modelCounts).map(([model_name, requests_count]) => ({
      model_name,
      requests_count,
      percentage: total > 0 ? Math.round((requests_count / total) * 1000) / 10 : 0,
    }));

    const evalCount = state.evaluations.length || 1;
    const avgFaith = state.evaluations.reduce((a, e) => a + e.faithfulness, 0) / evalCount;
    const avgRel = state.evaluations.reduce((a, e) => a + e.answer_relevance, 0) / evalCount;
    const avgGround = state.evaluations.reduce((a, e) => a + e.groundedness, 0) / evalCount;
    const avgHalluc = state.evaluations.reduce((a, e) => a + e.hallucination_score, 0) / evalCount;

    return {
      total_requests: total,
      successful_requests: successful,
      failed_requests: failed,
      success_rate_pct: Math.round(successRate * 10) / 10,
      latency: {
        p50_ms: 248.5,
        p95_ms: 410.2,
        p99_ms: 680.0,
        avg_ms: 292.4,
      },
      tokens: {
        prompt_tokens: promptTokens,
        completion_tokens: completionTokens,
        total_tokens: totalTokens,
      },
      cost: {
        current_spend_usd: state.budget.current_spend_usd,
        monthly_budget_usd: state.budget.monthly_budget_usd,
        budget_used_pct:
          Math.round((state.budget.current_spend_usd / state.budget.monthly_budget_usd) * 1000) / 10,
      },
      models_count: state.models.length,
      model_distribution: modelDistribution,
      evaluations: {
        total_evaluations: state.evaluations.length,
        avg_faithfulness: Math.round(avgFaith * 1000) / 1000,
        avg_relevance: Math.round(avgRel * 1000) / 1000,
        avg_groundedness: Math.round(avgGround * 1000) / 1000,
        avg_hallucination_score: Math.round(avgHalluc * 1000) / 1000,
      },
      deployments: {
        total: state.deployments.length,
        healthy: state.deployments.filter((d) => d.status === "Healthy").length,
        ready_replicas: state.deployments.reduce((a, d) => a + d.replicas_ready, 0),
        desired_replicas: state.deployments.reduce((a, d) => a + d.replicas_desired, 0),
      },
      active_environment: "Production",
      data_freshness: "Live Telemetry Active",
    };
  }

  // 3. GET /monitoring/metrics
  if (pathname === "/monitoring/metrics" && method === "GET") {
    const now = Date.now();
    const timestamps: string[] = [];
    const throughput_rpm: number[] = [];
    const p50_latency_ms: number[] = [];
    const p95_latency_ms: number[] = [];
    const p99_latency_ms: number[] = [];
    const error_rate_pct: number[] = [];

    for (let i = 11; i >= 0; i--) {
      const d = new Date(now - i * 5 * 60 * 1000);
      const hh = String(d.getHours()).padStart(2, "0");
      const mm = String(d.getMinutes()).padStart(2, "0");
      timestamps.push(`${hh}:${mm}`);

      const wave = Math.sin(i * 0.5);
      throughput_rpm.push(Math.round(42 + wave * 8));
      p50_latency_ms.push(Math.round((220 + wave * 18) * 10) / 10);
      p95_latency_ms.push(Math.round((380 + wave * 35) * 10) / 10);
      p99_latency_ms.push(Math.round((590 + wave * 60) * 10) / 10);
      error_rate_pct.push(i % 4 === 0 ? 1.2 : 0.4);
    }

    return {
      timestamps,
      throughput_rpm,
      p50_latency_ms,
      p95_latency_ms,
      p99_latency_ms,
      error_rate_pct,
      active_source: "Live Database Telemetry + Prometheus Exporter",
      is_prometheus_live: false,
    };
  }

  // 4. GET /monitoring/latency-distribution
  if (pathname === "/monitoring/latency-distribution" && method === "GET") {
    const total = Math.max(1, state.traces.length);
    const b1 = state.traces.filter((t) => t.latency_ms < 150).length;
    const b2 = state.traces.filter((t) => t.latency_ms >= 150 && t.latency_ms < 300).length;
    const b3 = state.traces.filter((t) => t.latency_ms >= 300 && t.latency_ms < 500).length;
    const b4 = state.traces.filter((t) => t.latency_ms >= 500).length;

    return [
      { range_label: "< 150ms (Fast)", count: b1, percentage: Math.round((b1 / total) * 1000) / 10 },
      { range_label: "150ms - 300ms (Nominal)", count: b2, percentage: Math.round((b2 / total) * 1000) / 10 },
      { range_label: "300ms - 500ms (Elevated)", count: b3, percentage: Math.round((b3 / total) * 1000) / 10 },
      { range_label: "> 500ms (Slow / Timeout)", count: b4, percentage: Math.round((b4 / total) * 1000) / 10 },
    ];
  }

  // 5. /models
  if (pathname === "/models" && method === "GET") {
    const stage = params.get("stage");
    let list = [...state.models];
    if (stage && stage !== "all") {
      list = list.filter((m) => m.stage.toLowerCase() === stage.toLowerCase());
    }
    return list;
  }

  if (pathname === "/models" && method === "POST") {
    const created = {
      id: `m-${Date.now().toString(36)}`,
      name: body.name || "Custom-LLM-Model",
      framework: body.framework || "vLLM / PyTorch",
      task: body.task || "Text Generation",
      stage: body.stage || "Development",
      version: body.version || "v1.0.0",
      parameters_count: body.parameters_count || "7B",
      artifact_uri: body.artifact_uri || "s3://ml-artifacts/custom-model",
      description: body.description || "Registered via Control Center UI",
      tags: '["custom", "registered"]',
      created_at: nowIso(),
      updated_at: nowIso(),
    };
    state.models.unshift(created);
    saveState(state);
    return created;
  }

  const modelStageMatch = pathname.match(/^\/models\/([^/]+)\/stage$/);
  if (modelStageMatch && method === "PATCH") {
    const id = modelStageMatch[1];
    const target = state.models.find((m) => m.id === id);
    if (target) {
      target.stage = body.stage || "Staging";
      target.updated_at = nowIso();
      saveState(state);
      return target;
    }
  }

  // 6. /experiments
  if (pathname === "/experiments" && method === "GET") {
    return state.experiments;
  }

  if (pathname === "/experiments/compare" && method === "GET") {
    const runsParam = params.get("runs") || "";
    const runIds = runsParam
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    const runs =
      runIds.length > 0
        ? state.experiments.filter((e) => runIds.includes(e.id))
        : state.experiments.slice(0, 3);

    const paramKeys = new Set<string>();
    const metricKeys = new Set<string>();
    runs.forEach((r) => {
      try {
        Object.keys(JSON.parse(r.parameters || "{}")).forEach((k) => paramKeys.add(k));
      } catch {}
      try {
        Object.keys(JSON.parse(r.metrics || "{}")).forEach((k) => metricKeys.add(k));
      } catch {}
    });

    return {
      run_ids: runs.map((r) => r.id),
      runs,
      parameter_keys: Array.from(paramKeys),
      metric_keys: Array.from(metricKeys),
    };
  }

  // 7. /evaluations
  if (pathname === "/evaluations" && method === "GET") {
    return state.evaluations;
  }

  if (pathname === "/evaluations/metrics/definitions" && method === "GET") {
    return [
      {
        metric_name: "Faithfulness",
        formula_or_approach:
          "Proportion of claims in the generated response that can be mathematically or semantically derived from the retrieved context.",
        target_range: "0.85 - 1.00 (Higher is better)",
        interpretation:
          "Measures whether the model sticks strictly to provided factual evidence without adding unverified extrapolations.",
        hallucination_relevance:
          "Primary indicator of factual hallucination. Scores below 0.70 represent high probability of confabulation.",
      },
      {
        metric_name: "Answer Relevance",
        formula_or_approach:
          "Semantic similarity and keyword alignment between the original user prompt and the generated response.",
        target_range: "0.80 - 1.00 (Higher is better)",
        interpretation:
          "Evaluates whether the model directly addressed the user question rather than digressing or evasively deflecting.",
        hallucination_relevance:
          "Detects evasive or off-topic hallucinations where the model outputs plausible-sounding but irrelevant text.",
      },
      {
        metric_name: "Context Precision",
        formula_or_approach: "Signal-to-noise ratio of relevant chunks within the top-K retrieved RAG documents.",
        target_range: "0.75 - 1.00 (Higher is better)",
        interpretation: "Determines whether the retriever ranks the most pertinent knowledge chunks first.",
        hallucination_relevance:
          "Low precision injects noisy distractors into the prompt window, increasing LLM reasoning errors.",
      },
      {
        metric_name: "TruLens Groundedness",
        formula_or_approach: "Sentence-level attribution verification mapping each output assertion to source evidence.",
        target_range: "0.85 - 1.00 (Higher is better)",
        interpretation: "Verifies that every claim is backed by an explicit span in the source knowledge base.",
        hallucination_relevance: "Directly flags ungrounded assertions as hallucination events.",
      },
    ];
  }

  if (pathname === "/evaluations" && method === "POST") {
    const id = `eval-${Date.now().toString(36)}`;
    const ts = nowIso();
    const newEval = {
      id,
      name: body.name || "Automated Evaluation Gate",
      model_name: body.model_name || "Llama-3.1-70B-Instruct",
      dataset_name: body.dataset_name || "enterprise-kb-v3-eval",
      evaluator_framework: body.evaluator_framework || "Ragas & TruLens Suite",
      status: "Completed",
      faithfulness: 0.938,
      answer_relevance: 0.924,
      context_precision: 0.896,
      context_recall: 0.91,
      groundedness: 0.945,
      hallucination_score: 0.055,
      avg_latency_ms: 284.5,
      total_tokens: 36400,
      test_cases_count: 20,
      created_at: ts,
    };
    state.evaluations.unshift(newEval);
    state.evaluationTestCases[id] = state.evaluationTestCases["eval-golden-rag-suite-01"].map((tc, idx) => ({
      ...tc,
      id: `tc-${id}-${idx + 1}`,
      evaluation_id: id,
    }));
    saveState(state);
    return newEval;
  }

  const evalDetailMatch = pathname.match(/^\/evaluations\/([^/]+)$/);
  if (evalDetailMatch && method === "GET") {
    const id = evalDetailMatch[1];
    const evaluation = state.evaluations.find((e) => e.id === id) || state.evaluations[0];
    const test_cases =
      state.evaluationTestCases[id] || state.evaluationTestCases["eval-golden-rag-suite-01"] || [];
    return { evaluation, test_cases };
  }

  // 8. /traces
  if (pathname === "/traces" && method === "GET") {
    const model = params.get("model");
    let list = [...state.traces];
    if (model && model !== "all") {
      list = list.filter((t) => t.model_name.toLowerCase().includes(model.toLowerCase()));
    }
    return list;
  }

  // 9. /costs
  if (pathname === "/costs/summary" && method === "GET") {
    const budget = state.budget;
    const alertThresholdAmount = budget.monthly_budget_usd * (budget.alert_threshold_pct / 100.0);
    const isAlertActive = budget.current_spend_usd >= alertThresholdAmount;

    return {
      budget,
      is_budget_alert_active: isAlertActive,
      provider_breakdown: [
        { provider: "tgi_cluster", total_tokens: 3400, estimated_cost_usd: 0.007, percentage: 78.9 },
        { provider: "vllm_local", total_tokens: 3357, estimated_cost_usd: 0.002, percentage: 21.1 },
      ],
      model_breakdown: [
        {
          model_name: "Mistral-Large-2",
          prompt_tokens: 3400,
          completion_tokens: 0,
          total_tokens: 3400,
          cost_usd: 0.007,
        },
        {
          model_name: "Llama-3.1-70B-Instruct",
          prompt_tokens: 2092,
          completion_tokens: 505,
          total_tokens: 2597,
          cost_usd: 0.001,
        },
        {
          model_name: "Qwen-2.5-Coder-32B",
          prompt_tokens: 620,
          completion_tokens: 140,
          total_tokens: 760,
          cost_usd: 0.0,
        },
      ],
      monthly_spend_projection_usd: Math.round(budget.current_spend_usd * 1.35 * 100) / 100,
      pricing_source: "Active Provider Catalog + Token Telemetry",
    };
  }

  if (pathname === "/costs/pricing" && method === "GET") {
    return state.pricing;
  }

  if (pathname === "/costs/budget" && method === "PATCH") {
    if (typeof body.monthly_budget_usd === "number") {
      state.budget.monthly_budget_usd = body.monthly_budget_usd;
    }
    if (typeof body.alert_threshold_pct === "number") {
      state.budget.alert_threshold_pct = body.alert_threshold_pct;
    }
    state.budget.updated_at = nowIso();
    saveState(state);
    return state.budget;
  }

  // 10. /deployments
  if (pathname === "/deployments" && method === "GET") {
    return state.deployments;
  }

  const depRestartMatch = pathname.match(/^\/deployments\/([^/]+)\/restart$/);
  if (depRestartMatch && method === "POST") {
    const id = depRestartMatch[1];
    const dep = state.deployments.find((d) => d.id === id);
    if (dep) {
      dep.updated_at = nowIso();
      dep.status = "Healthy";
      saveState(state);
      return dep;
    }
  }

  // 11. /cicd/workflows
  if (pathname === "/cicd/workflows" && method === "GET") {
    return state.workflows;
  }

  if (pathname === "/cicd/workflows/trigger" && method === "POST") {
    const created = {
      id: `run-${Math.floor(1100 + Math.random() * 900)}`,
      workflow_name: body.workflow_name || "Model Evaluation & Benchmark Gates",
      run_id: `gh-${Math.floor(9300000 + Math.random() * 90000)}`,
      branch: body.branch || "main",
      commit_sha: Math.random().toString(16).substring(2, 9),
      commit_message: `Manual workflow dispatch on branch ${body.branch || "main"}`,
      status: "Success",
      trigger_event: "workflow_dispatch",
      duration_seconds: 215,
      url: "https://github.com/MTaufikIqball00/LLMOps-Evaluasi-Model-End-To-End/actions",
      started_at: nowIso(),
    };
    state.workflows.unshift(created);
    saveState(state);
    return created;
  }

  // 12. /integrations
  if (pathname === "/integrations" && method === "GET") {
    return state.integrations;
  }

  const intTestMatch = pathname.match(/^\/integrations\/([^/]+)\/test$/);
  if (intTestMatch) {
    const serviceName = intTestMatch[1].toLowerCase();
    const item = state.integrations.find((i) => i.service_name.toLowerCase() === serviceName);
    if (item) {
      item.last_checked = nowIso();
      item.latency_ms = Math.round((8 + Math.random() * 25) * 10) / 10;
      saveState(state);
      return item;
    }
  }

  return { status: "ok" };
}
