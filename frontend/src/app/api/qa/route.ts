import { NextResponse } from 'next/server';
import * as fs from 'fs';
import * as path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function GET() {
  try {
    const projectRoot = path.resolve(process.cwd(), '..');
    const latestPath = path.join(projectRoot, 'qa-engine', 'data', 'latest_run.json');
    const historyPath = path.join(projectRoot, 'qa-engine', 'data', 'history.json');

    let latestRun = null;
    let history = [];

    if (fs.existsSync(latestPath)) {
      latestRun = JSON.parse(fs.readFileSync(latestPath, 'utf-8'));
    }

    if (fs.existsSync(historyPath)) {
      history = JSON.parse(fs.readFileSync(historyPath, 'utf-8'));
    }

    return NextResponse.json({
      latest: latestRun,
      history: history
    });
  } catch (err: any) {
    return NextResponse.json({ error: err.message }, { status: 500 });
  }
}

export async function POST(req: Request) {
  try {
    const body = await req.json().catch(() => ({ mode: 'smoke' }));
    const mode = body.mode || 'smoke';
    const projectRoot = path.resolve(process.cwd(), '..');

    // Run the QA CLI asynchronously
    const cmd = `node --experimental-strip-types qa-engine/src/cli.ts --mode ${mode}`;
    const { stdout, stderr } = await execAsync(cmd, { cwd: projectRoot });

    const latestPath = path.join(projectRoot, 'qa-engine', 'data', 'latest_run.json');
    let latestRun = null;
    if (fs.existsSync(latestPath)) {
      latestRun = JSON.parse(fs.readFileSync(latestPath, 'utf-8'));
    }

    return NextResponse.json({
      success: true,
      mode,
      output: stdout,
      latest: latestRun
    });
  } catch (err: any) {
    return NextResponse.json({ error: err.message, stack: err.stderr || err.stdout }, { status: 500 });
  }
}
