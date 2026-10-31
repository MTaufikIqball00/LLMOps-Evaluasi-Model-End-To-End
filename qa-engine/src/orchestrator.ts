import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import type { 
  ExecutionMode, 
  QARunRecord, 
  TestCaseResult, 
  Finding, 
  ExecutionSummary 
} from './types.ts';
import { FEATURE_INVENTORY } from './inventory.ts';
import { RootCauseAnalyzer } from './root_cause_analyzer.ts';
import { AIRepairAdvisor } from './ai_repair_advisor.ts';
import { ReportGenerator } from './report_generator.ts';

export class QAOrchestrator {
  private projectRoot: string;
  private backendDir: string;
  private frontendDir: string;
  private dataDir: string;
  private reportsDir: string;
  private artifactDir: string;

  constructor(projectRoot: string, artifactDir: string) {
    this.projectRoot = projectRoot;
    this.backendDir = path.join(projectRoot, 'backend');
    this.frontendDir = path.join(projectRoot, 'frontend');
    this.dataDir = path.join(projectRoot, 'qa-engine', 'data');
    this.reportsDir = path.join(projectRoot, 'qa-engine', 'reports');
    this.artifactDir = artifactDir;

    if (!fs.existsSync(this.dataDir)) {
      fs.mkdirSync(this.dataDir, { recursive: true });
    }
    if (!fs.existsSync(this.reportsDir)) {
      fs.mkdirSync(this.reportsDir, { recursive: true });
    }
  }

  public async run(mode: ExecutionMode = 'full'): Promise<QARunRecord> {
    const startedAt = new Date().toISOString();
    const startTime = Date.now();
    const testCases: TestCaseResult[] = [];
    const findings: Finding[] = [
      {
        id: 'SEC-VULN-001',
        title: 'Secret Scrubber Regex Hyphen Leak (sk-proj-... / sk-ant-...)',
        severity: 'HIGH',
        category: 'Security / Token Sanitization',
        status: 'OPEN',
        file_path: 'backend/src/observability/mod.rs',
        line_number: 62,
        description: 'Regex `(?i)(sk-[a-zA-Z0-9]{20,})` does not match hyphens, leaving modern project keys unredacted.',
        evidence: 'security_audit_test.rs::test_modern_project_key_vulnerability_probe confirmed token persistence',
        impact: 'Exposure of modern OpenAI and Anthropic project credentials in logs and database.',
        cve_or_cwe: 'CWE-312: Cleartext Storage of Sensitive Information'
      },
      {
        id: 'EVAL-DIV-001',
        title: 'Hallucination Index Formula Divergence Between Spec and Code',
        severity: 'MEDIUM',
        category: 'LLM Evaluation & Ragas',
        status: 'OPEN',
        file_path: 'backend/src/integrations/ragas.rs',
        line_number: 107,
        description: 'Implementation uses `1.0 - Faithfulness` instead of weighted formula `1.0 - (0.40*F + 0.35*G + 0.25*CP)`.',
        evidence: 'evaluation_engine_test.rs::test_hallucination_index_formula_divergence_audit confirmed calculation gap',
        impact: 'Low groundedness answers do not trigger hallucination alerts if faithfulness is high.',
        cve_or_cwe: 'CWE-684: Incorrect Provision of Specified Functionality'
      },
      {
        id: 'ARCH-DOC-001',
        title: 'Documented API Route Prefixing Mismatch (/api/* vs /api/v1/*)',
        severity: 'LOW',
        category: 'API Architecture & Documentation',
        status: 'OPEN',
        file_path: 'backend/src/api/mod.rs',
        line_number: 25,
        description: 'Axum router prefixes all endpoints under `/api/v1/*` while docs state `/api/*`.',
        evidence: 'Direct HTTP GET on http://localhost:8210/api/models returned 404',
        impact: 'External integrations expecting unversioned /api/* will encounter 404 Not Found.',
        cve_or_cwe: 'Architecture Documentation Drift'
      }
    ];

    console.log(`[QA Orchestrator] Starting autonomous run in mode: '${mode}'`);

    // 1. Backend Rust Suites
    if (['backend', 'database', 'evaluation', 'security', 'performance', 'full', 'smoke'].includes(mode)) {
      this.executeRustTests(mode, testCases);
    }

    // 2. Frontend Playwright Suites
    if (['frontend', 'full'].includes(mode)) {
      this.executePlaywrightTests(testCases);
    }

    // 3. Smoke / Live Probes
    if (['smoke', 'full'].includes(mode)) {
      this.executeLiveProbes(testCases);
    }

    const completedAt = new Date().toISOString();
    const durationSeconds = Math.round((Date.now() - startTime) / 1000);

    const passed = testCases.filter(t => t.status === 'PASS').length;
    const failed = testCases.filter(t => t.status === 'FAIL').length;
    const blocked = testCases.filter(t => t.status === 'BLOCKED').length;
    const skipped = testCases.filter(t => t.status === 'SKIPPED').length;
    const notTested = testCases.filter(t => t.status === 'NOT TESTED').length;
    const notImplemented = testCases.filter(t => t.status === 'NOT IMPLEMENTED').length;
    const total = testCases.length;
    const passRate = total > 0 ? parseFloat(((passed / total) * 100).toFixed(2)) : 0;

    const summary: ExecutionSummary = {
      run_id: `QA-${Date.now()}`,
      started_at: startedAt,
      completed_at: completedAt,
      duration_seconds: durationSeconds,
      execution_mode: mode,
      total_tests: total,
      passed,
      failed,
      blocked,
      skipped,
      not_tested: notTested,
      not_implemented: notImplemented,
      pass_rate_percentage: passRate
    };

    // Analyze root causes and repair proposals
    const failedTests = testCases.filter(t => t.status === 'FAIL');
    const rootCauses = RootCauseAnalyzer.analyze(failedTests, findings);
    const repairProposals = AIRepairAdvisor.generateProposals(findings);

    const record: QARunRecord = {
      summary,
      test_cases: testCases,
      findings,
      root_cause_analyses: rootCauses,
      repair_proposals: repairProposals,
      feature_coverage: FEATURE_INVENTORY
    };

    // Persist records
    this.persistRun(record);

    // Generate Reports
    ReportGenerator.saveReports(record, this.reportsDir, this.artifactDir);

    console.log(`[QA Orchestrator] Run finished. Total: ${total}, Passed: ${passed}, Failed: ${failed}. Pass Rate: ${passRate}%`);
    return record;
  }

  private executeRustTests(mode: ExecutionMode, results: TestCaseResult[]): void {
    console.log(`[QA Orchestrator] Executing Rust test suites...`);

    const testSuitesToRun = [];
    if (mode === 'backend' || mode === 'full') {
      testSuitesToRun.push('api_endpoints_test', 'database_integrity_test', 'evaluation_engine_test', 'security_audit_test', 'performance_test', 'integration_tests');
    } else if (mode === 'database') {
      testSuitesToRun.push('database_integrity_test');
    } else if (mode === 'evaluation') {
      testSuitesToRun.push('evaluation_engine_test');
    } else if (mode === 'security') {
      testSuitesToRun.push('security_audit_test');
    } else if (mode === 'performance') {
      testSuitesToRun.push('performance_test');
    } else if (mode === 'smoke') {
      testSuitesToRun.push('integration_tests');
    }

    for (const testName of testSuitesToRun) {
      const suiteStart = Date.now();
      try {
        const out = execSync(`cargo test --test ${testName} -- --nocapture`, {
          cwd: this.backendDir,
          encoding: 'utf-8',
          stdio: ['pipe', 'pipe', 'pipe']
        });
        const duration = Date.now() - suiteStart;
        results.push({
          id: `RUST-${testName}`,
          name: `Rust Suite: ${testName}`,
          suite: testName,
          category: testName.includes('security') ? 'security' : testName.includes('performance') ? 'performance' : testName.includes('database') ? 'database' : testName.includes('evaluation') ? 'evaluation' : 'backend',
          status: 'PASS',
          duration_ms: duration,
          assertion: 'cargo test exit code 0',
          evidence: out.split('\n').filter(l => l.includes('test result: ok')).join('; ')
        });
      } catch (err: any) {
        const duration = Date.now() - suiteStart;
        results.push({
          id: `RUST-${testName}`,
          name: `Rust Suite: ${testName}`,
          suite: testName,
          category: 'backend',
          status: 'FAIL',
          duration_ms: duration,
          assertion: 'cargo test exit code 0',
          error_message: err.message,
          stack_trace: err.stderr || err.stdout
        });
      }
    }
  }

  private executePlaywrightTests(results: TestCaseResult[]): void {
    console.log(`[QA Orchestrator] Executing Playwright E2E suites...`);
    const cliPath = path.join(this.frontendDir, 'node_modules', '@playwright', 'test', 'cli.js');

    const specs = [
      { name: "01_overview.spec.ts", label: "Overview Dashboard & Metrics" },
      { name: "02_models.spec.ts", label: "Model Registry & Lifecycle" },
      { name: "03_experiments.spec.ts", label: "Experiment Tracking & Diff" },
      { name: "04_evaluations.spec.ts", label: "LLM Evaluations & Hallucination" },
      { name: "05_monitoring.spec.ts", label: "Production Observability & Metrics" },
      { name: "06_traces.spec.ts", label: "Request Traces & Secret Scrubbing" },
      { name: "07_costs.spec.ts", label: "Token Usage & Cost Budgeting" },
      { name: "08_deployments.spec.ts", label: "Kubernetes Serving Deployments" },
      { name: "09_cicd.spec.ts", label: "CI/CD Pipeline Quality Gates" },
      { name: "10_integrations.spec.ts", label: "Integrations & Service Adapters" },
      { name: "11_system.spec.ts", label: "System Health & SRE Diagnostics" },
      { name: "12_navigation_resilience.spec.ts", label: "Sidebar Navigation & Layout" },
      { name: "13_qa_dashboard.spec.ts", label: "QA Dashboard & Audit Engine" },
    ];

    for (const spec of specs) {
      const specStart = Date.now();
      try {
        const out = execSync(`node "${cliPath}" test "e2e/${spec.name}"`, {
          cwd: this.frontendDir,
          encoding: 'utf-8',
          stdio: ['pipe', 'pipe', 'pipe']
        });
        const duration = Date.now() - specStart;
        results.push({
          id: `E2E-${spec.name}`,
          name: spec.label,
          suite: spec.name,
          category: 'frontend',
          status: 'PASS',
          duration_ms: duration,
          assertion: 'Playwright DOM expectations and zero console error',
          evidence: out.split('\n').filter(l => l.includes('passed')).join('; ')
        });
      } catch (err: any) {
        const duration = Date.now() - specStart;
        results.push({
          id: `E2E-${spec.name}`,
          name: spec.label,
          suite: spec.name,
          category: 'frontend',
          status: 'FAIL',
          duration_ms: duration,
          assertion: 'Playwright DOM expectations',
          error_message: err.message,
          stack_trace: err.stdout || err.stderr
        });
      }
    }
  }

  private executeLiveProbes(results: TestCaseResult[]): void {
    console.log(`[QA Orchestrator] Executing Live HTTP Service Probes...`);

    // Probe 1: Backend Health
    try {
      const start = Date.now();
      const res = execSync('curl.exe -s http://localhost:8210/api/v1/health', { encoding: 'utf-8' });
      const duration = Date.now() - start;
      const isHealthy = res.includes('Healthy');
      results.push({
        id: 'PROBE-BACKEND-HEALTH',
        name: 'Backend API Live Health Probe (:8210)',
        suite: 'Live Service Probes',
        category: 'smoke',
        status: isHealthy ? 'PASS' : 'FAIL',
        duration_ms: duration,
        assertion: 'Status is Healthy',
        evidence: res
      });
    } catch (err: any) {
      results.push({
        id: 'PROBE-BACKEND-HEALTH',
        name: 'Backend API Live Health Probe (:8210)',
        suite: 'Live Service Probes',
        category: 'smoke',
        status: 'FAIL',
        duration_ms: 0,
        error_message: err.message
      });
    }

    // Probe 2: Frontend Root
    try {
      const start = Date.now();
      const res = execSync('curl.exe -s -o NUL -w "%{http_code}" http://localhost:3210', { encoding: 'utf-8' });
      const duration = Date.now() - start;
      const code = res.trim();
      results.push({
        id: 'PROBE-FRONTEND-STATUS',
        name: 'Next.js Frontend HTTP Probe (:3210)',
        suite: 'Live Service Probes',
        category: 'smoke',
        status: code === '200' ? 'PASS' : 'FAIL',
        duration_ms: duration,
        assertion: 'HTTP Status 200 OK',
        evidence: `HTTP Status Code: ${code}`
      });
    } catch (err: any) {
      results.push({
        id: 'PROBE-FRONTEND-STATUS',
        name: 'Next.js Frontend HTTP Probe (:3210)',
        suite: 'Live Service Probes',
        category: 'smoke',
        status: 'FAIL',
        duration_ms: 0,
        error_message: err.message
      });
    }
  }

  private persistRun(record: QARunRecord): void {
    const latestFile = path.join(this.dataDir, 'latest_run.json');
    const historyFile = path.join(this.dataDir, 'history.json');

    fs.writeFileSync(latestFile, JSON.stringify(record, null, 2), 'utf-8');

    let history: ExecutionSummary[] = [];
    if (fs.existsSync(historyFile)) {
      try {
        history = JSON.parse(fs.readFileSync(historyFile, 'utf-8'));
      } catch (e) {
        history = [];
      }
    }
    history.unshift(record.summary);
    if (history.length > 50) history = history.slice(0, 50);
    fs.writeFileSync(historyFile, JSON.stringify(history, null, 2), 'utf-8');
  }
}
