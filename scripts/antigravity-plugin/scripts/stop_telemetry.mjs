import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const WORKER_PATH = path.join(__dirname, "telemetry_worker.mjs");

function main() {
  let raw = "";
  try {
    raw = fs.readFileSync(0, "utf8");
  } catch {
    process.stdout.write("{}");
    return;
  }

  try {
    const child = spawn(process.execPath, [WORKER_PATH], {
      detached: true,
      stdio: ["pipe", "ignore", "ignore"],
      windowsHide: true,
      env: process.env,
    });
    child.on("error", () => {});
    process.stdout.write("{}");
    if (child.stdin) {
      child.stdin.end(raw, () => {
        child.unref();
        process.exit(0);
      });
    } else {
      child.unref();
      process.exit(0);
    }
  } catch {
    process.stdout.write("{}");
  }
}

main();
