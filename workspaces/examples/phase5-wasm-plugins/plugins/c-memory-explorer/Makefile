# Makefile for C Memory Explorer Plugin
# Builds WASM using Emscripten

# Emscripten compiler
CC = emcc

# Source and output files
SRC = src/main.c
OUT = plugin.wasm

# Compiler flags for WASM
# Note: Use EXPORT_ALL to let export_name attributes control exports
CFLAGS = -O3 \
         -s STANDALONE_WASM \
         -s ERROR_ON_UNDEFINED_SYMBOLS=0 \
         -s EXPORT_ALL=1 \
         -s INITIAL_MEMORY=65536 \
         -s TOTAL_MEMORY=65536 \
         -s ALLOW_MEMORY_GROWTH=0 \
         --no-entry

# Default target
all: $(OUT)

# Build the WASM module
$(OUT): $(SRC)
	$(CC) $(CFLAGS) $< -o $@
	@echo "✅ Built C plugin: $@"
	@ls -lh $@

# Clean build artifacts
clean:
	rm -f $(OUT)
	rm -f *.js *.wasm.map

# Check if emscripten is installed
check:
	@which emcc > /dev/null || (echo "❌ Emscripten not found. Install from: https://emscripten.org" && exit 1)
	@echo "✅ Emscripten found: $$(emcc --version | head -1)"

.PHONY: all clean check