/**
 * Autonomous AI QA Engine & Audit System Types
 * Sole Source of Truth: LLMOps Control Center & Pipeline Evaluasi Model End-to-End
 */

export type ExecutionMode =
  | 'smoke'
  | 'frontend'
  | 'backend'
  | 'database'
  | 'evaluation'
  | 'integration'
  | 'security'
  | 'performance'
  | 'full';

export type TestStatus =
  | 'PASS'
  | 'FAIL'
  | 'BLOCKED'
  | 'SKIPPED'
  | 'NOT TESTED'
  | 'NOT IMPLEMENTED';

export type FindingSeverity =
  | 'CRITICAL'
  | 'HIGH'
  | 'MEDIUM'
  | 'LOW'
  | 'INFO';

export type RootCauseCategory =
  | 'Backend logic bug'
  | 'Frontend UI/state bug'
  | 'API schema mismatch'
  | 'DB constraint violation'
  | 'Environment configuration issue'
  | 'External service unavailability'
  | 'Flaky or timing-dependent test'
  | 'Outdated test expectation'
  | 'INSUFFICIENT EVIDENCE — ROOT CAUSE NOT CONFIRMED';

export type ConfidenceLevel =
  | 'CONFIRMED'
  | 'PROBABLE'
  | 'INSUFFICIENT_EVIDENCE';

export interface TestCaseResult {
  id: string;
  name: string;
  suite: string;
  category: 'frontend' | 'backend' | 'database' | 'evaluation' | 'security' | 'performance' | 'integration' | 'smoke';
  status: TestStatus;
  duration_ms: number;
  assertion?: string;
  evidence?: string;
  error_message?: string;
  stack_trace?: string;
}

export interface Finding {
  id: string;
  title: string;
  severity: FindingSeverity;
  category: string;
  status: 'OPEN' | 'RESOLVED' | 'WONT_FIX';
  file_path?: string;
  line_number?: number;
  description: string;
  evidence: string;
  impact: string;
  cve_or_cwe?: string;
}

export interface RootCauseAnalysis {
  failure_id: string;
  title: string;
  root_cause_category: RootCauseCategory;
  confidence_level: ConfidenceLevel;
  file_path?: string;
  line_number?: number;
  description: string;
  evidence_trace: string[];
  reproduction_steps: string[];
}

export interface RepairPatchProposal {
  id: string;
  finding_id: string;
  title: string;
  target_file: string;
  target_lines: string;
  rationale: string;
  diff: string;
  side_effects_assessment: string;
  rollback_instructions: string;
  verification_command: string;
}

export interface FeatureCoverageItem {
  feature_id: string;
  feature_name: string;
  frontend_route: string;
  backend_endpoint: string;
  database_entities: string[];
  test_suites: string[];
  verification_status: TestStatus;
  notes: string;
}

export interface ExecutionSummary {
  run_id: string;
  started_at: string;
  completed_at: string;
  duration_seconds: number;
  execution_mode: ExecutionMode;
  total_tests: number;
  passed: number;
  failed: number;
  blocked: number;
  skipped: number;
  not_tested: number;
  not_implemented: number;
  pass_rate_percentage: number;
}

export interface QARunRecord {
  summary: ExecutionSummary;
  test_cases: TestCaseResult[];
  findings: Finding[];
  root_cause_analyses: RootCauseAnalysis[];
  repair_proposals: RepairPatchProposal[];
  feature_coverage: FeatureCoverageItem[];
}
