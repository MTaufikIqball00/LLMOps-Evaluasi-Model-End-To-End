#!/usr/bin/env bash
set -e

BACKEND_PORT=8210
FRONTEND_PORT=3210

echo "=========================================================="
echo "   LLMOps Control Center — Launching Services (POSIX)     "
echo "=========================================================="

# Check ports
for port in $BACKEND_PORT $FRONTEND_PORT; do
  if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null ; then
    echo "[ERROR] Port $port is already in use!"
    exit 1
  fi
done

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$ROOT_DIR/.logs"

echo "Starting Rust Axum backend on port $BACKEND_PORT..."
(cd "$ROOT_DIR/backend" && cargo run) > "$ROOT_DIR/.logs/backend.log" 2>&1 &
BACKEND_PID=$!

echo "Starting Next.js frontend on port $FRONTEND_PORT..."
(cd "$ROOT_DIR/frontend" && node ./node_modules/next/dist/bin/next dev -p $FRONTEND_PORT) > "$ROOT_DIR/.logs/frontend.log" 2>&1 &
FRONTEND_PID=$!

echo "$BACKEND_PID" > "$ROOT_DIR/.pids"
echo "$FRONTEND_PID" >> "$ROOT_DIR/.pids"

sleep 3

echo "=========================================================="
echo "   LLMOPS CONTROL CENTER RUNNING SUCCESSFULLY             "
echo "=========================================================="
echo " Dashboard UI:    http://localhost:$FRONTEND_PORT"
echo " Backend API:     http://localhost:$BACKEND_PORT"
echo " Health Endpoint: http://localhost:$BACKEND_PORT/api/v1/health"
echo " Backend PID:     $BACKEND_PID"
echo " Frontend PID:    $FRONTEND_PID"
echo " Stop Script:     ./scripts/stop.sh"
echo "=========================================================="
