import type { RepairPatchProposal, Finding } from './types.ts';

export class AIRepairAdvisor {
  /**
   * Generates minimal, reviewable patch proposals with rollback instructions.
   * STRICT POLICY: Does not modify production code automatically.
   */
  public static generateProposals(findings: Finding[]): RepairPatchProposal[] {
    const proposals: RepairPatchProposal[] = [];

    for (const finding of findings) {
      if (finding.id === 'SEC-VULN-001') {
        proposals.push({
          id: "PATCH-001",
          finding_id: "SEC-VULN-001",
          title: "Patch Secret Scrubber Regex to Cover Modern Project & Anthropic Keys",
          target_file: "backend/src/observability/mod.rs",
          target_lines: "62-63",
          rationale: "Updating the OpenAI regex from `(?i)(sk-[a-zA-Z0-9]{20,})` to include hyphens and underscores `(?i)(sk-[a-zA-Z0-9_-]{20,})` ensures modern project-scoped tokens (e.g., `sk-proj-...`) and Anthropic tokens (`sk-ant-...`) are completely redacted into `[REDACTED_API_KEY]` before logging or persistence.",
          diff: `--- a/backend/src/observability/mod.rs
+++ b/backend/src/observability/mod.rs
@@ -59,7 +59,7 @@ impl RequestTrace {
     /// Scrubs potential API keys, secrets, and auth tokens from payload strings
     pub fn scrub_secrets(&mut self) {
         // OpenAI API Key pattern: sk-...
-        let openai_regex = Regex::new(r"(?i)(sk-[a-zA-Z0-9]{20,})").unwrap();
+        let openai_regex = Regex::new(r"(?i)(sk-[a-zA-Z0-9_-]{20,})").unwrap();
         self.prompt_payload = openai_regex
             .replace_all(&self.prompt_payload, "[REDACTED_API_KEY]")
             .to_string();`,
          side_effects_assessment: "Low risk. Valid prompts containing arbitrary words will not be affected unless they match the `sk-` prefix with 20+ alphanumerics, hyphens, or underscores.",
          rollback_instructions: "Revert `backend/src/observability/mod.rs` to commit HEAD using `git checkout backend/src/observability/mod.rs`.",
          verification_command: "cargo test --test security_audit_test test_modern_project_key_vulnerability_probe"
        });
      }

      if (finding.id === 'EVAL-DIV-001') {
        proposals.push({
          id: "PATCH-002",
          finding_id: "EVAL-DIV-001",
          title: "Align Hallucination Index Formula with Documented Multi-Metric Weighted Architecture",
          target_file: "backend/src/integrations/ragas.rs",
          target_lines: "107-111",
          rationale: "Update `backend/src/integrations/ragas.rs` to compute the Hallucination Index using the documented weighted formula: `1.0 - (0.40 * Faithfulness + 0.35 * Groundedness + 0.25 * Context Precision)`. This ensures low groundedness or irrelevant context triggers hallucination alerts even if faithfulness is falsely high.",
          diff: `--- a/backend/src/integrations/ragas.rs
+++ b/backend/src/integrations/ragas.rs
@@ -104,7 +104,11 @@ impl RagasEvaluator {
 
         // Calculate composite hallucination index
         // Hallucination Index = 1.0 - (0.40 * Faithfulness + 0.35 * Groundedness + 0.25 * Context Precision)
-        let hallucination_index = (1.0 - record.faithfulness).max(0.0);
+        let composite_groundedness = (0.40 * record.faithfulness)
+            + (0.35 * record.groundedness)
+            + (0.25 * record.context_precision);
+        let hallucination_index = (1.0 - composite_groundedness).clamp(0.0, 1.0);
+        
         let is_hallucinated = hallucination_index > 0.35 || record.groundedness < 0.75;`,
          side_effects_assessment: "Moderate. Existing benchmarks with low groundedness (< 0.75) will now correctly transition `is_hallucinated` to `true`, triggering alert thresholds as designed in the system architecture.",
          rollback_instructions: "Restore the single-metric calculation: `let hallucination_index = (1.0 - record.faithfulness).max(0.0);`.",
          verification_command: "cargo test --test evaluation_engine_test test_hallucination_index_formula_divergence_audit"
        });
      }

      if (finding.id === 'ARCH-DOC-001') {
        proposals.push({
          id: "PATCH-003",
          finding_id: "ARCH-DOC-001",
          title: "Add Route Aliases in Axum Router for Root /api/* Endpoints",
          target_file: "backend/src/api/mod.rs",
          target_lines: "30-36",
          rationale: "Nests the router at both `/v1` and the root `/` under `/api` to maintain backwards compatibility with documentation and legacy clients expecting `/api/models` while preserving `/api/v1/models` versioning.",
          diff: `--- a/backend/src/api/mod.rs
+++ b/backend/src/api/mod.rs
@@ -32,7 +32,8 @@ pub fn create_router(pool: SqlitePool) -> Router {
 
     // Nest under /api/v1 and provide alias for unversioned /api
     Router::new()
-        .nest("/api/v1", v1_routes)
+        .nest("/api/v1", v1_routes.clone())
+        .nest("/api", v1_routes)
         .route("/health", get(health_check))`,
          side_effects_assessment: "Negligible. Extends route matching so both `/api/*` and `/api/v1/*` resolve successfully without modifying handler logic or database interactions.",
          rollback_instructions: "Remove `.nest(\"/api\", v1_routes)` from `backend/src/api/mod.rs`.",
          verification_command: "cargo test --test api_endpoints_test"
        });
      }
    }

    return proposals;
  }
}
