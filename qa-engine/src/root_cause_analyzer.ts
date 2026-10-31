import type { RootCauseAnalysis, TestCaseResult, Finding, ConfidenceLevel, RootCauseCategory } from './types.ts';

export class RootCauseAnalyzer {
  /**
   * Evaluates failed test cases and discovered findings to determine verified root causes.
   * Strictly adheres to the Evidence Rule:
   * "If root cause cannot be confirmed with concrete evidence, state clearly:
   *  INSUFFICIENT EVIDENCE — ROOT CAUSE NOT CONFIRMED"
   */
  public static analyze(
    failedTests: TestCaseResult[],
    findings: Finding[]
  ): RootCauseAnalysis[] {
    const analyses: RootCauseAnalysis[] = [];

    // Analyze confirmed findings from forensics & security/eval audits
    for (const finding of findings) {
      if (finding.id === 'SEC-VULN-001') {
        analyses.push({
          failure_id: finding.id,
          title: "Secret Scrubber Leaks Modern Hyphenated API Keys (sk-proj-... / sk-ant-...)",
          root_cause_category: "Backend logic bug",
          confidence_level: "CONFIRMED",
          file_path: "backend/src/observability/mod.rs",
          line_number: 62,
          description: "The secret redaction regex `(?i)(sk-[a-zA-Z0-9]{20,})` does not include hyphens `-`. OpenAI project-scoped keys (`sk-proj-...`) and Anthropic keys (`sk-ant-...`) contain hyphens, causing the regex to stop matching after the prefix, leaving the actual sensitive key secret exposed in trace payloads and persisted SQLite records.",
          evidence_trace: [
            "Source Code: backend/src/observability/mod.rs:62",
            "Pattern: Regex `(?i)(sk-[a-zA-Z0-9]{20,})` fails to match `sk-proj-9999-SECRET-TOKEN-EXPOSURE`",
            "Deterministic Test: backend/tests/security_audit_test.rs::test_modern_project_key_vulnerability_probe",
            "Verified Payload Output: Key remained intact in redacted trace text instead of being replaced with [REDACTED_API_KEY]"
          ],
          reproduction_steps: [
            "1. Instantiate RequestTrace with prompt containing `Authorization: Bearer sk-proj-12345-abcdef1234567890`",
            "2. Invoke `trace.scrub_secrets()`",
            "3. Inspect `trace.prompt_payload`",
            "4. Observe that `sk-proj-12345-abcdef1234567890` is NOT replaced with `[REDACTED_API_KEY]`"
          ]
        });
      }

      if (finding.id === 'EVAL-DIV-001') {
        analyses.push({
          failure_id: finding.id,
          title: "Hallucination Index Formula Divergence Between Spec and Implementation",
          root_cause_category: "Backend logic bug",
          confidence_level: "CONFIRMED",
          file_path: "backend/src/integrations/ragas.rs",
          line_number: 107,
          description: "Specification and architecture documents define Hallucination Index as `1.0 - (0.40 * Faithfulness + 0.35 * Groundedness + 0.25 * Context Precision)`. The actual implementation in `backend/src/integrations/ragas.rs:107` simplifies this to `(1.0 - record.faithfulness).max(0.0)`. This completely discards Groundedness and Context Precision weights from the composite score.",
          evidence_trace: [
            "Source Code: backend/src/integrations/ragas.rs lines 107-111",
            "Implemented: `let hallucination_index = (1.0 - record.faithfulness).max(0.0);`",
            "Documented Formula: `1.0 - (0.40 * F + 0.35 * G + 0.25 * CP)`",
            "Deterministic Test: backend/tests/evaluation_engine_test.rs::test_hallucination_index_formula_divergence_audit",
            "Verified Divergence: Groundedness & Context Precision delta of up to 0.40 unaccounted for"
          ],
          reproduction_steps: [
            "1. Provide evaluation test case with Faithfulness=0.90, Groundedness=0.30, Context Precision=0.40",
            "2. Documented formula expected score: 1.0 - (0.40*0.90 + 0.35*0.30 + 0.25*0.40) = 1.0 - (0.36 + 0.105 + 0.10) = 0.435 (High Hallucination alert triggered)",
            "3. Actual backend output: 1.0 - 0.90 = 0.10 (Alert NOT triggered)",
            "4. Severe hallucination and ungrounded output goes undetected"
          ]
        });
      }

      if (finding.id === 'ARCH-DOC-001') {
        analyses.push({
          failure_id: finding.id,
          title: "Documented API Route Prefixing Mismatch (/api/* vs /api/v1/*)",
          root_cause_category: "API schema mismatch",
          confidence_level: "CONFIRMED",
          file_path: "backend/src/api/mod.rs",
          line_number: 25,
          description: "Architecture diagrams and README specify `/api/models`, `/api/experiments`, etc. The Axum router nests all business endpoints under `/api/v1/*` while leaving `/health` at the root `/health`. Any external client connecting to `/api/models` receives a 404 Not Found.",
          evidence_trace: [
            "Source Code: backend/src/api/mod.rs:25-39",
            "Axum Router: `let api_routes = Router::new().nest(\"/v1\", v1_routes);`",
            "Live Probe: HTTP GET http://localhost:8210/api/models yields 404 Not Found",
            "Live Probe: HTTP GET http://localhost:8210/api/v1/models yields 200 OK"
          ],
          reproduction_steps: [
            "1. Send GET request to http://localhost:8210/api/models",
            "2. Note 404 response",
            "3. Send GET request to http://localhost:8210/api/v1/models",
            "4. Note 200 OK response with model records"
          ]
        });
      }
    }

    // Analyze dynamic test failures
    for (const test of failedTests) {
      if (analyses.some(a => a.failure_id === test.id)) continue;

      if (test.stack_trace && test.stack_trace.includes("expect(locator).toBeVisible()")) {
        analyses.push({
          failure_id: test.id,
          title: `UI Element Visibility Mismatch in ${test.name}`,
          root_cause_category: "Frontend UI/state bug",
          confidence_level: "PROBABLE",
          description: `Playwright assertion timed out waiting for DOM element: ${test.error_message}`,
          evidence_trace: [
            `Test Suite: ${test.suite}`,
            `Error: ${test.error_message}`,
            `Stack Trace: ${test.stack_trace.split('\n')[0]}`
          ],
          reproduction_steps: [
            `1. Run Playwright test: ${test.suite}`,
            `2. Observe locator failure`
          ]
        });
      } else {
        // Enforce the strict evidence rule
        analyses.push({
          failure_id: test.id,
          title: `Unresolved Failure: ${test.name}`,
          root_cause_category: "INSUFFICIENT EVIDENCE — ROOT CAUSE NOT CONFIRMED",
          confidence_level: "INSUFFICIENT_EVIDENCE",
          description: "INSUFFICIENT EVIDENCE — ROOT CAUSE NOT CONFIRMED. The failure output does not contain sufficient diagnostic traces, logs, or reproduction data to confirm an exact code defect.",
          evidence_trace: [
            `Suite: ${test.suite}`,
            `Error: ${test.error_message || 'No explicit error message captured'}`
          ],
          reproduction_steps: [
            `Rerun test with verbose diagnostic logging: ${test.suite}`
          ]
        });
      }
    }

    return analyses;
  }
}
