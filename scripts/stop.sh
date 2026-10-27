#!/usr/bin/env bash

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PID_FILE="$ROOT_DIR/.pids"

echo "=========================================================="
echo "   Stopping LLMOps Control Center Services (POSIX)        "
echo "=========================================================="

if [ -f "$PID_FILE" ]; then
  while IFS= read -r pid; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      echo "Stopping process PID $pid..."
      kill "$pid" 2>/dev/null || true
    fi
  done < "$PID_FILE"
  rm -f "$PID_FILE"
  echo "[OK] Processes stopped and .pids cleared."
else
  echo "No .pids file found."
fi
