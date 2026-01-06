/*
 * Phase 5 Example: Multi-Language WASM Plugin System
 *
 * This workspace demonstrates the WASM plugin architecture in structurizr-rs
 * with plugins implemented in 5 different programming languages:
 *
 * EDUCATIONAL PLUGINS:
 * - Rust (rust-hello-arch): Zero-cost abstractions, no_std, ownership model
 * - C (c-memory-explorer): Manual memory management, pointer arithmetic
 * - AssemblyScript (as-type-safe-greeter): TypeScript-to-WASM compilation
 * - Go (go-stats-calc): TinyGo simplicity, interfaces, defer patterns
 * - Zig (zig-pattern-matcher): Compile-time (comptime) features, explicit control
 *
 * KEY CONCEPTS:
 * - Plugins are sandboxed WebAssembly modules
 * - Each plugin has a manifest (plugin.toml) defining capabilities
 * - Plugins communicate with the host via defined host functions
 * - Capability-based security model controls what plugins can do
 * - Auto-build system rebuilds plugins when source changes
 *
 * BUILD ALL PLUGINS:
 *   cd workspaces/examples/phase5-wasm-plugins && make all
 *
 * NOTE: WASM plugins require the "wasm" feature flag (enabled by default).
 */

workspace "Multi-Language WASM Plugins" "Educational demonstration of WASM plugins in Rust, C, AssemblyScript, Go, and Zig" {

    !docs "docs"
    !adrs "adrs"

    # Original example plugin
    !plugin "plugins/workspace-analyzer"

    # Educational plugins demonstrating different languages
    # All plugins are now built and enabled
    !plugin "plugins/rust-hello-arch"        # Rust: Zero-cost abstractions, no_std
    !plugin "plugins/c-memory-explorer"      # C: Manual memory management
    !plugin "plugins/as-type-safe-greeter"   # AssemblyScript: TypeScript syntax
    !plugin "plugins/go-stats-calc"          # Go: Simplicity, interfaces (TinyGo)
    !plugin "plugins/zig-pattern-matcher"    # Zig: Comptime, explicit control

    /*
     * This workspace models the multi-language plugin system,
     * showing how WASM plugins in different languages interact with the host runtime.
     */

    model {
        # Plugin Developers (different language expertise)
        rustDev = person "Rust Developer" "Writes no_std WASM plugins with zero-cost abstractions" {
            tags "Developer,Rust"
        }
        cDev = person "C Developer" "Writes plugins with manual memory management" {
            tags "Developer,C"
        }
        tsDev = person "TypeScript Developer" "Writes plugins using AssemblyScript" {
            tags "Developer,AssemblyScript"
        }
        goDev = person "Go Developer" "Writes plugins using TinyGo" {
            tags "Developer,Go"
        }
        zigDev = person "Zig Developer" "Writes plugins with comptime features" {
            tags "Developer,Zig"
        }

        # The structurizr-rs system with plugin support
        structurizrRs = softwareSystem "structurizr-rs" "Rust implementation of Structurizr with multi-language plugin support" {

            # Core parsing and model
            dslParser = container "DSL Parser" "Parses workspace DSL files" "Rust/Pest"
            coreModel = container "Core Model" "C4 model types and workspace" "Rust"

            # Scripting subsystem
            scriptingEngine = container "Scripting Engine" "Lua runtime for inline scripts" "Rust/mlua"

            # Plugin subsystem
            pluginEngine = container "Plugin Engine" "WASM runtime for plugins" "Rust/wasmtime" {
                tags "Plugin System"
            }
            pluginBuilder = container "Plugin Builder" "Auto-builds plugins from source" "Rust" {
                tags "Plugin System"
            }
            pluginManifest = container "Manifest Parser" "Parses plugin.toml files" "Rust/toml" {
                tags "Plugin System"
            }
            hostFunctions = container "Host Functions" "Bridge between WASM and workspace" "Rust" {
                tags "Plugin System"
            }

            # Web and rendering
            webServer = container "Web Server" "Serves workspace UI" "Rust/Axum"
            svgRenderer = container "SVG Renderer" "Renders diagrams" "Rust"
        }

        # Multi-language WASM plugins
        rustPlugin = softwareSystem "Rust Plugin" "rust-hello-arch: Zero-cost abstractions demo (~3KB)" {
            tags "External,Plugin,Rust"
        }
        cPlugin = softwareSystem "C Plugin" "c-memory-explorer: Manual memory management (~5KB)" {
            tags "External,Plugin,C"
        }
        asPlugin = softwareSystem "AssemblyScript Plugin" "as-type-safe-greeter: TypeScript syntax (~15KB)" {
            tags "External,Plugin,AssemblyScript"
        }
        goPlugin = softwareSystem "Go Plugin" "go-stats-calc: TinyGo simplicity (~20KB)" {
            tags "External,Plugin,Go"
        }
        zigPlugin = softwareSystem "Zig Plugin" "zig-pattern-matcher: Comptime features (~8KB)" {
            tags "External,Plugin,Zig"
        }

        # Relationships - Plugin Development Flow
        rustDev -> rustPlugin "Develops with cargo"
        cDev -> cPlugin "Develops with emscripten"
        tsDev -> asPlugin "Develops with asc"
        goDev -> goPlugin "Develops with tinygo"
        zigDev -> zigPlugin "Develops with zig"

        # Relationships - Auto-build Flow
        pluginBuilder -> rustPlugin "Builds with cargo"
        pluginBuilder -> cPlugin "Builds with emcc"
        pluginBuilder -> asPlugin "Builds with npm"
        pluginBuilder -> goPlugin "Builds with tinygo"
        pluginBuilder -> zigPlugin "Builds with zig"

        # Relationships - Plugin Loading Flow
        dslParser -> pluginManifest "Triggers plugin loading via"
        pluginManifest -> pluginEngine "Configures"
        pluginEngine -> rustPlugin "Loads and executes"
        pluginEngine -> cPlugin "Loads and executes"
        pluginEngine -> asPlugin "Loads and executes"
        pluginEngine -> goPlugin "Loads and executes"
        pluginEngine -> zigPlugin "Loads and executes"

        # Relationships - Plugin Execution Flow
        rustPlugin -> hostFunctions "Calls"
        cPlugin -> hostFunctions "Calls"
        asPlugin -> hostFunctions "Calls"
        goPlugin -> hostFunctions "Calls"
        zigPlugin -> hostFunctions "Calls"
        hostFunctions -> coreModel "Reads/modifies"

        # Relationships - Internal
        dslParser -> coreModel "Builds"
        dslParser -> scriptingEngine "Executes Lua via"
        scriptingEngine -> coreModel "Modifies"
        webServer -> svgRenderer "Uses"
        webServer -> pluginBuilder "Triggers on startup"
        svgRenderer -> coreModel "Reads"
    }

    views {
        systemLandscape "PluginLandscape" "Plugin system overview" {
            include *
            autoLayout
        }

        systemContext structurizrRs "StructurizrContext" "structurizr-rs with plugin support" {
            include *
            autoLayout
        }

        container structurizrRs "StructurizrContainers" "Internal architecture" {
            include *
            autoLayout
        }

        container wasmPlugin "PluginContainers" "Plugin structure" {
            include *
            autoLayout
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
                color "#ffffff"
            }
            element "Developer" {
                shape Person
                background "#2e8b57"
                color "#ffffff"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "External" {
                background "#999999"
                color "#ffffff"
            }
            element "Plugin System" {
                background "#6b8e23"
                color "#ffffff"
            }
            element "Plugin" {
                background "#ff6347"
                color "#ffffff"
            }
            # Language-specific colors
            element "Rust" {
                background "#dea584"
                color "#000000"
            }
            element "C" {
                background "#555555"
                color "#ffffff"
            }
            element "AssemblyScript" {
                background "#007acc"
                color "#ffffff"
            }
            element "Go" {
                background "#00add8"
                color "#ffffff"
            }
            element "Zig" {
                background "#f7a41d"
                color "#000000"
            }
        }
    }
}
