import * as path from 'path';
import * as fs from 'fs';
import { QAOrchestrator } from './orchestrator.ts';
import { QABackgroundService } from './service.ts';
import type { ExecutionMode } from './types.ts';

const PROJECT_ROOT = path.resolve(process.cwd());
const ARTIFACT_DIR = path.resolve('C:/Users/taufi/.gemini/antigravity/brain/ff35f78c-d870-4b7f-af03-74bf31e6bad0');

async function main() {
  const args = process.argv.slice(2);
  const orchestrator = new QAOrchestrator(PROJECT_ROOT, ARTIFACT_DIR);

  let mode: ExecutionMode = 'full';
  let isDaemon = false;
  let showStatus = false;
  let intervalMinutes = 30;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--mode' && args[i + 1]) {
      mode = args[i + 1] as ExecutionMode;
      i++;
    } else if (args[i] === '--daemon') {
      isDaemon = true;
    } else if (args[i] === '--interval' && args[i + 1]) {
      intervalMinutes = parseInt(args[i + 1], 10) || 30;
      i++;
    } else if (args[i] === '--status') {
      showStatus = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      printHelp();
      return;
    }
  }

  if (showStatus) {
    const latestPath = path.join(PROJECT_ROOT, 'qa-engine', 'data', 'latest_run.json');
    if (fs.existsSync(latestPath)) {
      const data = JSON.parse(fs.readFileSync(latestPath, 'utf-8'));
      console.log(`\n=== LATEST QA ENGINE AUDIT RUN ===`);
      console.log(`Run ID:        ${data.summary.run_id}`);
      console.log(`Timestamp:     ${data.summary.completed_at}`);
      console.log(`Mode:          ${data.summary.execution_mode}`);
      console.log(`Total Tests:   ${data.summary.total_tests}`);
      console.log(`Passed:        ${data.summary.passed}`);
      console.log(`Failed:        ${data.summary.failed}`);
      console.log(`Pass Rate:     ${data.summary.pass_rate_percentage}%`);
      console.log(`Findings:      ${data.findings.length} (${data.findings.filter((f: any) => f.severity === 'HIGH').length} High, ${data.findings.filter((f: any) => f.severity === 'MEDIUM').length} Medium)`);
      console.log(`Report:        qa-engine/reports/audit_report_latest.md\n`);
    } else {
      console.log(`No previous audit run recorded. Run 'node --experimental-strip-types qa-engine/src/cli.ts --mode full' first.`);
    }
    return;
  }

  if (isDaemon) {
    const service = new QABackgroundService(orchestrator, intervalMinutes, mode);
    service.start();
    // Keep process alive
    process.on('SIGINT', () => {
      service.stop();
      process.exit(0);
    });
  } else {
    console.log(`================================================================`);
    console.log(`     Autonomous AI QA Engine & Software Auditor Dispatcher      `);
    console.log(`================================================================`);
    console.log(`Target Workspace: ${PROJECT_ROOT}`);
    console.log(`Execution Mode:   ${mode}`);
    console.log(`Timestamp:        ${new Date().toISOString()}`);
    console.log(`----------------------------------------------------------------\n`);

    const record = await orchestrator.run(mode);

    console.log(`\n----------------------------------------------------------------`);
    console.log(`AUDIT COMPLETE:`);
    console.log(`- Total Tests:     ${record.summary.total_tests}`);
    console.log(`- Passed:          ${record.summary.passed}`);
    console.log(`- Failed:          ${record.summary.failed}`);
    console.log(`- Pass Rate:       ${record.summary.pass_rate_percentage}%`);
    console.log(`- Discovered Findings: ${record.findings.length}`);
    console.log(`- Reports Generated:`);
    console.log(`    Markdown: qa-engine/reports/audit_report_latest.md`);
    console.log(`    JSON:     qa-engine/reports/audit_report_latest.json`);
    console.log(`    Artifact: ${path.join(ARTIFACT_DIR, 'audit_report.md')}`);
    console.log(`================================================================\n`);
  }
}

function printHelp() {
  console.log(`
Autonomous AI QA Engine & Audit System CLI

Usage:
  node --experimental-strip-types qa-engine/src/cli.ts [options]

Options:
  --mode <mode>        Execution mode: smoke | frontend | backend | database |
                       evaluation | integration | security | performance | full
                       (default: full)
  --status             Display summary of the latest recorded audit run
  --daemon             Run in background scheduler mode with periodic executions
  --interval <mins>    Interval in minutes for daemon executions (default: 30)
  --help, -h           Show this help message
`);
}

main().catch(err => {
  console.error(`[QA Engine CLI Fatal Error]:`, err);
  process.exit(1);
});
