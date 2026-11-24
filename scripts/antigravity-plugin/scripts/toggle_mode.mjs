import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PLUGIN_DIR = path.resolve(__dirname, "..");
const HOOKS_PATH = path.join(PLUGIN_DIR, "hooks.json");
const CONFIG_PATH = path.resolve(PLUGIN_DIR, "..", "..", "config.json");
const PLUGIN_ID = "llmops-control-guard";

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + "\n", "utf8");
}

function getStatus() {
  const hooks = readJson(HOOKS_PATH);
  const config = fs.existsSync(CONFIG_PATH) ? readJson(CONFIG_PATH) : {};
  const pluginEnabled = config?.plugins?.[PLUGIN_ID]?.enabled !== false;
  const preGuardEnabled = hooks?.["llmops-pre-tool-guard"]?.enabled !== false;
  const stopTelemetryEnabled = hooks?.["llmops-stop-telemetry"]?.enabled !== false;

  let activeMode = "off";
  if (pluginEnabled && preGuardEnabled && stopTelemetryEnabled) {
    activeMode = "mode1";
  } else if (pluginEnabled && !preGuardEnabled && stopTelemetryEnabled) {
    activeMode = "mode2";
  } else if (pluginEnabled) {
    activeMode = "custom";
  }

  return {
    activeMode,
    pluginEnabled,
    preGuardEnabled,
    stopTelemetryEnabled,
    nodePath: process.execPath,
    backendUrl: process.env.LLMOPS_API_URL || "http://127.0.0.1:8210/api/v1",
  };
}

function applyMode(mode) {
  const hooks = readJson(HOOKS_PATH);
  const config = fs.existsSync(CONFIG_PATH) ? readJson(CONFIG_PATH) : {};
  if (!config.plugins || typeof config.plugins !== "object") {
    config.plugins = {};
  }
  const existingEntry = config.plugins[PLUGIN_ID] && typeof config.plugins[PLUGIN_ID] === "object"
    ? config.plugins[PLUGIN_ID]
    : {};

  if (mode === "mode1") {
    hooks["llmops-pre-tool-guard"].enabled = true;
    hooks["llmops-stop-telemetry"].enabled = true;
    config.plugins[PLUGIN_ID] = { ...existingEntry, enabled: true };
  } else if (mode === "mode2") {
    hooks["llmops-pre-tool-guard"].enabled = false;
    hooks["llmops-stop-telemetry"].enabled = true;
    config.plugins[PLUGIN_ID] = { ...existingEntry, enabled: true };
  } else if (mode === "off") {
    hooks["llmops-pre-tool-guard"].enabled = false;
    hooks["llmops-stop-telemetry"].enabled = false;
    config.plugins[PLUGIN_ID] = { ...existingEntry, enabled: false };
  } else {
    console.error("Usage: node toggle_mode.mjs <mode1|mode2|off|status>");
    process.exit(1);
  }

  writeJson(HOOKS_PATH, hooks);
  writeJson(CONFIG_PATH, config);
  console.log(JSON.stringify(getStatus(), null, 2));
}

const arg = (process.argv[2] || "status").toLowerCase();
if (arg === "status") {
  console.log(JSON.stringify(getStatus(), null, 2));
} else {
  applyMode(arg);
}
