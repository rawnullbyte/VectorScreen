#!/usr/bin/env bash
# Debug launcher for VectorScreen
# Runs with debug-level logging for development

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Debug-level logging (file-only, no stdout)
export RUST_LOG="debug"

echo "Starting VectorScreen in debug mode..."
echo "Log file: /tmp/vector-screen.log"
echo "Press Ctrl+C to stop"

cd "$PROJECT_DIR"

# Build and run
cargo run -- "$@"
