#!/bin/bash
# Serve the Markdown Demo example on a random available port

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

find_available_port() {
    while true; do
        PORT=$((RANDOM % 55535 + 10000))
        if ! lsof -i:$PORT > /dev/null 2>&1; then
            echo $PORT
            return
        fi
    done
}

PORT=$(find_available_port)

echo "Starting Markdown Demo example..."
echo "URL: http://localhost:$PORT"
echo ""

cd "$PROJECT_ROOT"
cargo run --quiet -- serve --data-dir "$SCRIPT_DIR" --port $PORT
