import fs from "node:fs";
import path from "node:path";

const ALLOW_JSON = JSON.stringify({ decision: "allow" });

const TEST_OR_EXAMPLE_PATH_RE =
  /(?:_test\.rs|\.test\.|\.spec\.|security_audit|observability[/\\]mod\.rs|pre_tool_guard\.mjs|\.env\.example|qa-engine[/\\])/i;

const PLACEHOLDER_RE = /(?:example|placeholder|your_|your-|dummy|fake|xxxx|redacted)/i;

const SECRET_RULES = [
  {
    label: "OpenAI / Anthropic API Key",
    regex: /(?:^|[^a-zA-Z0-9_-])(sk-(?:proj-|ant-)?[a-zA-Z0-9_-]{20,})/i,
  },
  {
    label: "GitHub Personal Access Token",
    regex: /\b(gh[pousr]_[A-Za-z0-9_]{30,})\b/,
  },
  {
    label: "AWS Access Key ID",
    regex: /\b(AKIA[0-9A-Z]{16})\b/,
  },
  {
    label: "Slack Token",
    regex: /\b(xox[baprs]-[0-9A-Za-z-]{10,})\b/,
  },
  {
    label: "HuggingFace Token",
    regex: /\b(hf_[A-Za-z0-9]{20,})\b/,
  },
  {
    label: "Private Key Block",
    regex: /(-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----)/,
  },
];

const WRITE_TOOLS = new Set([
  "write_file",
  "edit_file",
  "write_to_file",
  "replace_file_content",
  "multi_replace_file_content",
]);

function extractContentStrings(args) {
  if (!args || typeof args !== "object") return "";
  const parts = [];
  for (const key of ["Content", "Replacement", "CodeContent", "ReplacementContent"]) {
    if (typeof args[key] === "string") {
      parts.push(args[key]);
    }
  }
  if (Array.isArray(args.ReplacementChunks)) {
    for (const chunk of args.ReplacementChunks) {
      if (chunk && typeof chunk.ReplacementContent === "string") {
        parts.push(chunk.ReplacementContent);
      }
    }
  }
  return parts.join("\n");
}

function main() {
  let raw = "";
  try {
    raw = fs.readFileSync(0, "utf8");
  } catch {
    process.stdout.write(ALLOW_JSON);
    return;
  }

  let payload;
  try {
    payload = JSON.parse(raw);
  } catch {
    process.stdout.write(ALLOW_JSON);
    return;
  }

  const toolCall = payload?.toolCall;
  const toolName = String(toolCall?.name || "").toLowerCase();
  if (!WRITE_TOOLS.has(toolName)) {
    process.stdout.write(ALLOW_JSON);
    return;
  }

  const args = toolCall?.args || {};
  const targetPath = String(args.Path || args.TargetFile || "");
  if (targetPath && TEST_OR_EXAMPLE_PATH_RE.test(targetPath)) {
    process.stdout.write(ALLOW_JSON);
    return;
  }

  const content = extractContentStrings(args);
  if (!content) {
    process.stdout.write(ALLOW_JSON);
    return;
  }

  for (const rule of SECRET_RULES) {
    const match = content.match(rule.regex);
    if (match && match[1] && !PLACEHOLDER_RE.test(match[1])) {
      const base = targetPath ? path.basename(targetPath) : "target file";
      const result = {
        decision: "force_ask",
        reason: `[LLMOps Security Guard] Terdeteksi kemungkinan kredensial/API key (${rule.label}) yang akan ditulis ke ${base}. Mohon konfirmasi manual sebelum melanjutkan.`,
      };
      process.stdout.write(JSON.stringify(result));
      return;
    }
  }

  process.stdout.write(ALLOW_JSON);
}

main();
