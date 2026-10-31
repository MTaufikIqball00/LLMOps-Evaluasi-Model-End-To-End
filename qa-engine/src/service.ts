import { QAOrchestrator } from './orchestrator.ts';
import type { ExecutionMode } from './types.ts';

export class QABackgroundService {
  private orchestrator: QAOrchestrator;
  private intervalMinutes: number;
  private mode: ExecutionMode;
  private timer: NodeJS.Timeout | null = null;
  private isRunning: boolean = false;

  constructor(orchestrator: QAOrchestrator, intervalMinutes: number = 30, mode: ExecutionMode = 'smoke') {
    this.orchestrator = orchestrator;
    this.intervalMinutes = intervalMinutes;
    this.mode = mode;
  }

  public start(): void {
    if (this.isRunning) {
      console.log(`[QA Background Service] Already active.`);
      return;
    }
    this.isRunning = true;
    console.log(`[QA Background Service] Started. Interval: ${this.intervalMinutes}m, Default Mode: ${this.mode}`);

    // Trigger initial run immediately
    this.executeJob();

    // Schedule subsequent runs
    this.timer = setInterval(() => {
      this.executeJob();
    }, this.intervalMinutes * 60 * 1000);
  }

  public stop(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.isRunning = false;
    console.log(`[QA Background Service] Stopped.`);
  }

  private async executeJob(): Promise<void> {
    console.log(`[QA Background Service] [${new Date().toISOString()}] Dispatching scheduled audit job...`);
    try {
      const record = await this.orchestrator.run(this.mode);
      console.log(`[QA Background Service] Completed scheduled audit run ${record.summary.run_id}. Pass rate: ${record.summary.pass_rate_percentage}%`);
    } catch (err) {
      console.error(`[QA Background Service] Error during scheduled audit run:`, err);
    }
  }
}
