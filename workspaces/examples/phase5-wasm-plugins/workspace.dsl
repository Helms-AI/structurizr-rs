/*
 * Phase 5 Example: WASM Plugin System
 *
 * This workspace demonstrates the WASM plugin architecture in structurizr-rs.
 * WASM plugins provide language-agnostic extensibility for advanced use cases
 * that require more power than Lua scripting offers.
 *
 * KEY CONCEPTS:
 * - Plugins are sandboxed WebAssembly modules
 * - Each plugin has a manifest (plugin.toml) defining capabilities
 * - Plugins communicate with the host via defined host functions
 * - Capability-based security model controls what plugins can do
 *
 * NOTE: WASM plugins require the "wasm" feature flag (enabled by default).
 * The example plugin in ./plugins/ demonstrates the plugin structure.
 *
 * To use a plugin (future syntax):
 *   !plugin "./plugins/workspace-analyzer"
 */

workspace "WASM Plugin Architecture" "Demonstrates the WASM plugin system for structurizr-rs" {

    !docs "docs"
    !adrs "adrs"
    !plugin "plugins/workspace-analyzer"

    /*
     * This workspace models the plugin system itself,
     * showing how WASM plugins interact with the host runtime.
     */

    model {
        # Plugin Developer
        developer = person "Plugin Developer" "Writes WASM plugins in Rust, C, or other languages"

        # The structurizr-rs system with plugin support
        structurizrRs = softwareSystem "structurizr-rs" "Rust implementation of Structurizr with plugin support" {

            # Core parsing and model
            dslParser = container "DSL Parser" "Parses workspace DSL files" "Rust/Pest"
            coreModel = container "Core Model" "C4 model types and workspace" "Rust"

            # Scripting subsystem
            scriptingEngine = container "Scripting Engine" "Lua runtime for inline scripts" "Rust/mlua"

            # Plugin subsystem
            pluginEngine = container "Plugin Engine" "WASM runtime for plugins" "Rust/wasmtime" {
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

        # External WASM plugin
        wasmPlugin = softwareSystem "WASM Plugin" "Custom plugin compiled to WebAssembly" {
            tags "External"
            tags "Plugin"

            pluginCode = container "Plugin Code" "Business logic in any language" "Rust/C/AssemblyScript"
            wasmModule = container "WASM Module" "Compiled WebAssembly binary" "WASM"
            manifestFile = container "plugin.toml" "Plugin manifest with capabilities" "TOML"
        }

        # Relationships - Plugin Development Flow
        developer -> wasmPlugin "Develops"
        developer -> manifestFile "Configures capabilities in"

        # Relationships - Plugin Loading Flow
        dslParser -> pluginManifest "Triggers plugin loading via"
        pluginManifest -> manifestFile "Reads"
        pluginManifest -> pluginEngine "Configures"
        pluginEngine -> wasmModule "Loads and executes"

        # Relationships - Plugin Execution Flow
        wasmModule -> hostFunctions "Calls"
        hostFunctions -> coreModel "Reads/modifies"

        # Relationships - Internal
        dslParser -> coreModel "Builds"
        dslParser -> scriptingEngine "Executes Lua via"
        scriptingEngine -> coreModel "Modifies"
        webServer -> svgRenderer "Uses"
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
        }
    }
}
