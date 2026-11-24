import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const STATE_PATH = path.join(__dirname, ".last_sent.json");
const TAIL_BYTES = 256 * 1024;
const MAX_STATE_KEYS = 50;

function readTail(filePath) {
  const stat = fs.statSync(filePath);
  const start = Math.max(0, stat.size - TAIL_BYTES);
  const length = stat.size - start;
  const fd = fs.openSync(filePath, "r");
  try {
    const buf = Buffer.allocUnsafe(length);
    fs.readSync(fd, buf, 0, length, start);
    let text = buf.toString("utf8");
    if (start > 0) {
      const nl = text.indexOf("\n");
      text = nl >= 0 ? text.slice(nl + 1) : text;
    }
    return text;
  } finally {
    fs.closeSync(fd);
  }
}

function stripRequestWrappers(text) {
  if (!text) return "";
  let cleaned = String(text);
  cleaned = cleaned.replace(/<ADDITIONAL_METADATA>[\s\S]*?<\/ADDITIONAL_METADATA>/gi, "");
  cleaned = cleaned.replace(/<USER_SETTINGS_CHANGE>[\s\S]*?<\/USER_SETTINGS_CHANGE>/gi, "");
  cleaned = cleaned.replace(/<\/?USER_REQUEST>/gi, "");
  return cleaned.trim().slice(0, 4000);
}

function loadDedupState() {
  try {
    return JSON.parse(fs.readFileSync(STATE_PATH, "utf8"));
  } catch {
    return {};
  }
}

function saveDedupState(state) {
  const entries = Object.entries(state);
  const trimmed =
    entries.length > MAX_STATE_KEYS
      ? Object.fromEntries(entries.slice(entries.length - MAX_STATE_KEYS))
      : state;
  fs.writeFileSync(STATE_PATH, JSON.stringify(trimmed, null, 2), "utf8");
}

function parseTurnFromTranscript(transcriptPath, fallbackPrompt, fallbackModel) {
  const text = readTail(transcriptPath);
  const lines = text.split(/\r?\n/);
  const entries = [];

  for (const rawLine of lines) {
    const line = rawLine.replace(/^\uFEFF/, "").trim();
    if (!line) continue;
    try {
      entries.push(JSON.parse(line));
    } catch {
      // Ignore partial or malformed lines cleanly
    }
  }

  let lastUserIdx = -1;
  for (let i = entries.length - 1; i >= 0; i--) {
    if (entries[i]?.type === "USER_INPUT") {
      lastUserIdx = i;
      break;
    }
  }

  const turnEntries = lastUserIdx >= 0 ? entries.slice(lastUserIdx) : entries.slice(-10);
  const lastUser = lastUserIdx >= 0 ? entries[lastUserIdx] : null;
  const responses = turnEntries.filter((e) => e?.type === "PLANNER_RESPONSE");
  const contextEntries = turnEntries.filter(
    (e) => e?.type && e.type !== "USER_INPUT" && e.type !== "PLANNER_RESPONSE" && typeof e?.content === "string"
  );

  const lastResp = responses.length > 0 ? responses[responses.length - 1] : null;
  const lastEntry = turnEntries.length > 0 ? turnEntries[turnEntries.length - 1] : null;

  const rawPrompt = lastUser?.content || fallbackPrompt || "";
  const prompt = stripRequestWrappers(rawPrompt);

  let completion = "";
  for (let i = responses.length - 1; i >= 0; i--) {
    if (typeof responses[i]?.content === "string" && responses[i].content.trim()) {
      completion = responses[i].content.trim().slice(0, 4000);
      break;
    }
  }

  let promptTokens = 0;
  let cacheReadTokens = 0;
  let completionTokens = 0;
  for (const resp of responses) {
    promptTokens += Number(resp?.input_tokens || 0);
    cacheReadTokens += Number(resp?.cache_read_tokens || 0);
    completionTokens += Number(resp?.output_tokens || 0);
  }

  if (promptTokens <= 0 && prompt.length > 0) {
    promptTokens = Math.max(1, Math.ceil(prompt.length / 4));
  }
  if (completionTokens <= 0 && completion.length > 0) {
    completionTokens = Math.max(1, Math.ceil(completion.length / 4));
  }

  let latencyMs = 250;
  const startMs = Date.parse(lastUser?.created_at || "");
  const endMs = Date.parse(lastResp?.created_at || lastEntry?.created_at || "");
  if (Number.isFinite(startMs) && Number.isFinite(endMs) && endMs >= startMs) {
    latencyMs = Math.max(50, endMs - startMs);
  }

  const modelName = String(lastResp?.model || fallbackModel || "gemini-antigravity");
  const contextText = contextEntries
    .map((e) => e.content)
    .join("\n")
    .trim()
    .slice(0, 1500);

  return {
    lastStepIndex: lastResp?.step_index ?? lastEntry?.step_index ?? null,
    prompt,
    completion,
    promptTokens,
    cacheReadTokens,
    completionTokens,
    latencyMs,
    modelName,
    contextText,
  };
}

function detectGuardMode() {
  try {
    const hooksPath = path.resolve(__dirname, "..", "hooks.json");
    const hooks = JSON.parse(fs.readFileSync(hooksPath, "utf8"));
    return hooks?.["llmops-pre-tool-guard"]?.enabled !== false ? "mode1" : "mode2";
  } catch {
    return "mode1";
  }
}

async function run() {
  const raw = fs.readFileSync(0, "utf8");
  const payload = JSON.parse(raw);
  const guardMode = detectGuardMode();

  const apiUrl = (process.env.LLMOPS_API_URL || "http://127.0.0.1:8210/api/v1").replace(/\/+$/, "");
  const adminKey = process.env.LLMOPS_ADMIN_API_KEY || "";
  const convId = String(payload?.conversationId || "unknown");
  const transcriptPath = payload?.transcriptPath;
  const workspace = Array.isArray(payload?.workspacePaths) && payload.workspacePaths[0]
    ? path.basename(String(payload.workspacePaths[0]))
    : path.basename(String(payload?.cwd || "antigravity"));

  let turn = {
    lastStepIndex: null,
    prompt: stripRequestWrappers(payload?.lastUserInput || ""),
    completion: "",
    promptTokens: Math.max(1, Math.ceil(String(payload?.lastUserInput || "").length / 4)),
    cacheReadTokens: 0,
    completionTokens: 1,
    latencyMs: 250,
    modelName: String(payload?.modelName || "gemini-antigravity"),
    contextText: `Antigravity workspace: ${workspace}`,
  };

  if (transcriptPath && fs.existsSync(transcriptPath)) {
    turn = parseTurnFromTranscript(transcriptPath, payload?.lastUserInput, payload?.modelName);
    if (!turn.contextText) {
      turn.contextText = `Antigravity workspace: ${workspace}`;
    }
  }

  const state = loadDedupState();
  if (turn.lastStepIndex !== null && state[convId] === turn.lastStepIndex) {
    return;
  }

  const headers = { "Content-Type": "application/json" };
  if (adminKey) {
    headers["X-API-Key"] = adminKey;
  }

  const traceBody = {
    span_name: "antigravity::turn",
    model_name: turn.modelName,
    provider: "google_antigravity",
    status_code: payload?.error ? 500 : 200,
    latency_ms: turn.latencyMs,
    prompt_tokens: turn.promptTokens,
    completion_tokens: turn.completionTokens,
    prompt: turn.prompt || null,
    completion: turn.completion || null,
    metadata: {
      source: "antigravity_hook",
      llmops_guard: true,
      guard_mode: guardMode,
      conversation_id: convId,
      cache_read_tokens: turn.cacheReadTokens,
      termination_reason: payload?.terminationReason || "NO_TOOL_CALL",
      workspace,
    },
    error_message: payload?.error ? String(payload.error) : null,
  };

  const traceRes = await fetch(`${apiUrl}/traces`, {
    method: "POST",
    headers,
    body: JSON.stringify(traceBody),
    signal: AbortSignal.timeout(1500),
  });

  if (!traceRes.ok) {
    return;
  }

  if (turn.lastStepIndex !== null) {
    state[convId] = turn.lastStepIndex;
    saveDedupState(state);
  }

  if (turn.completion.length >= 80 && turn.prompt.length >= 10) {
    const evalBody = {
      name: `Antigravity Turn Eval (${convId.slice(0, 8)})`,
      model_name: turn.modelName,
      dataset_name: "antigravity-live-session",
      evaluator_framework: "Ragas & TruLens Suite",
      use_llm_judge: false,
      sample_test_cases: [
        {
          question: turn.prompt,
          ground_truth: null,
          generated_answer: turn.completion,
          retrieved_context: turn.contextText,
        },
      ],
    };

    await fetch(`${apiUrl}/evaluations`, {
      method: "POST",
      headers,
      body: JSON.stringify(evalBody),
      signal: AbortSignal.timeout(1500),
    }).catch(() => {});
  }
}

run().catch(() => {
  process.exit(0);
});
