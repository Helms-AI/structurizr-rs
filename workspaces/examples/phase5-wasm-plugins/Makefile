# Unified Makefile for building all WASM plugins
# Supports: Rust, C, AssemblyScript, Go, Zig
#
# Usage:
#   make          - Build all out-of-date plugins
#   make rust     - Build only Rust plugins
#   make c        - Build only C plugins
#   make as       - Build only AssemblyScript plugins
#   make go       - Build only Go plugins
#   make zig      - Build only Zig plugins
#   make clean    - Remove all built WASM files
#   make check    - Check which plugins need rebuilding

.PHONY: all check clean rust c assemblyscript go zig help

# Find all plugin directories
PLUGIN_DIRS := $(wildcard plugins/*)
PLUGIN_WASMS := $(addsuffix /plugin.wasm,$(PLUGIN_DIRS))

# Categorize plugins by language
RUST_PLUGINS := $(wildcard plugins/rust-*)
C_PLUGINS := $(wildcard plugins/c-*)
AS_PLUGINS := $(wildcard plugins/as-*)
GO_PLUGINS := $(wildcard plugins/go-*)
ZIG_PLUGINS := $(wildcard plugins/zig-*)

# Default target: build all plugins
all: $(PLUGIN_WASMS)
	@echo "✅ All plugins built successfully!"

# Pattern rules for each language

# Rust plugins
plugins/rust-%/plugin.wasm: plugins/rust-%/src/*.rs plugins/rust-%/Cargo.toml
	@echo "🦀 Building Rust plugin: $(@D)"
	@cd $(@D) && cargo build --target wasm32-unknown-unknown --release --quiet
	@cp $(@D)/target/wasm32-unknown-unknown/release/$(subst -,_,$(notdir $(basename $(@D)))).wasm $@
	@echo "✓ Built: $@"

# C plugins (using Emscripten)
plugins/c-%/plugin.wasm: plugins/c-%/src/*.c
	@echo "🔧 Building C plugin: $(@D)"
	@cd $(@D) && emcc src/main.c -o plugin.wasm \
		-s STANDALONE_WASM \
		-s ERROR_ON_UNDEFINED_SYMBOLS=0 \
		-s EXPORTED_FUNCTIONS='["_start","_run"]' \
		-O3 --no-entry
	@echo "✓ Built: $@"

# AssemblyScript plugins
plugins/as-%/plugin.wasm: plugins/as-%/assembly/*.ts plugins/as-%/package.json
	@echo "📘 Building AssemblyScript plugin: $(@D)"
	@if [ ! -d "$(@D)/node_modules" ]; then \
		echo "Installing dependencies..."; \
		cd $(@D) && npm install --quiet; \
	fi
	@cd $(@D) && npm run build --silent
	@echo "✓ Built: $@"

# Go plugins (using TinyGo with wasm-unknown target to avoid WASI dependencies)
plugins/go-%/plugin.wasm: plugins/go-%/*.go plugins/go-%/go.mod
	@echo "🐹 Building Go plugin: $(@D)"
	@cd $(@D) && tinygo build -o plugin.wasm -target wasm-unknown -no-debug -opt 2 main.go
	@echo "✓ Built: $@"

# Zig plugins (using build-exe with explicit exports for Zig 0.15+)
plugins/zig-%/plugin.wasm: plugins/zig-%/src/*.zig
	@echo "⚡ Building Zig plugin: $(@D)"
	@cd $(@D) && zig build-exe src/main.zig \
		-target wasm32-freestanding \
		-O ReleaseSmall \
		-femit-bin=plugin.wasm \
		--export=_start --export=run
	@echo "✓ Built: $@"

# Language-specific targets
rust: $(addsuffix /plugin.wasm,$(RUST_PLUGINS))
	@echo "✅ All Rust plugins built"

c: $(addsuffix /plugin.wasm,$(C_PLUGINS))
	@echo "✅ All C plugins built"

assemblyscript as: $(addsuffix /plugin.wasm,$(AS_PLUGINS))
	@echo "✅ All AssemblyScript plugins built"

go: $(addsuffix /plugin.wasm,$(GO_PLUGINS))
	@echo "✅ All Go plugins built"

zig: $(addsuffix /plugin.wasm,$(ZIG_PLUGINS))
	@echo "✅ All Zig plugins built"

# Check which plugins need rebuilding
check:
	@echo "Checking plugin status..."
	@for dir in $(PLUGIN_DIRS); do \
		if [ ! -f $$dir/plugin.wasm ]; then \
			echo "❌ $$dir: Not built"; \
		elif [ $$dir/plugin.wasm -ot $$dir/src ] 2>/dev/null || \
		     [ $$dir/plugin.wasm -ot $$dir/Cargo.toml ] 2>/dev/null || \
		     [ $$dir/plugin.wasm -ot $$dir/assembly ] 2>/dev/null || \
		     [ $$dir/plugin.wasm -ot $$dir/go.mod ] 2>/dev/null || \
		     [ $$dir/plugin.wasm -ot $$dir/build.zig ] 2>/dev/null; then \
			echo "⚠️  $$dir: Out of date"; \
		else \
			echo "✓  $$dir: Up to date"; \
		fi \
	done

# Clean all built WASM files
clean:
	@echo "Cleaning all plugin builds..."
	@rm -f plugins/*/plugin.wasm
	@for dir in $(RUST_PLUGINS); do \
		if [ -d $$dir ]; then \
			echo "Cleaning Rust plugin: $$dir"; \
			cd $$dir && cargo clean 2>/dev/null || true; \
		fi \
	done
	@for dir in $(AS_PLUGINS); do \
		if [ -d $$dir ]; then \
			echo "Cleaning AssemblyScript plugin: $$dir"; \
			rm -rf $$dir/build $$dir/node_modules 2>/dev/null || true; \
		fi \
	done
	@echo "✅ Clean complete"

# Help target
help:
	@echo "WASM Plugin Build System"
	@echo "========================"
	@echo ""
	@echo "Targets:"
	@echo "  make              Build all out-of-date plugins"
	@echo "  make rust         Build all Rust plugins"
	@echo "  make c            Build all C plugins"
	@echo "  make as           Build all AssemblyScript plugins"
	@echo "  make go           Build all Go plugins"
	@echo "  make zig          Build all Zig plugins"
	@echo "  make check        Check which plugins need rebuilding"
	@echo "  make clean        Remove all built WASM files"
	@echo "  make help         Show this help message"
	@echo ""
	@echo "Plugin Structure:"
	@echo "  plugins/"
	@echo "  ├── rust-**/     Rust plugins (cargo + wasm32-unknown-unknown)"
	@echo "  ├── c-**/        C plugins (emscripten)"
	@echo "  ├── as-**/       AssemblyScript plugins (npm + asc)"
	@echo "  ├── go-**/       Go plugins (tinygo)"
	@echo "  └── zig-**/      Zig plugins (zig build-lib)"
	@echo ""
	@echo "Requirements:"
	@echo "  - Rust:          rustup target add wasm32-unknown-unknown"
	@echo "  - C:             emscripten (emcc)"
	@echo "  - AssemblyScript: npm install -g assemblyscript"
	@echo "  - Go:            go install github.com/tinygo-org/tinygo"
	@echo "  - Zig:           Download from https://ziglang.org"