#!/bin/bash
# Serve the Banking Dynamics example on a random available port

# Find project root (3 levels up from this script)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Find a random available port above 10000
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

echo "Starting Banking Dynamics example..."
echo "URL: http://localhost:$PORT"
echo ""

cd "$PROJECT_ROOT"
cargo run --quiet -- serve --data-dir "$SCRIPT_DIR" --port $PORT
