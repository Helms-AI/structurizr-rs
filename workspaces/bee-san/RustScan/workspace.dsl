workspace "bee-san/RustScan" "🤖 The Modern Port Scanner 🤖" {

    model {
        beeSanRustScan = softwareSystem "bee-san/RustScan" "Faster Nmap Scanning with Rust" {
            rustscan = container "rustscan" "Faster Nmap Scanning with Rust" "Rust, Async" {
                benchmark = component "Benchmark" "" "Rust Struct"
                namedTimer = component "NamedTimer" "" "Rust Struct"
                scanOrder = component "ScanOrder" "" "Rust Enum"
                scriptsRequired = component "ScriptsRequired" "" "Rust Enum"
                portRange = component "PortRange" "" "Rust Struct"
                opts = component "Opts" "" "Rust Struct"
                config = component "Config" "" "Rust Struct"
                tui = component "tui" "" "Rust Module"
                input = component "input" "" "Rust Module"
                scanner = component "scanner" "" "Rust Module"
                portStrategy = component "port_strategy" "" "Rust Module"
                benchmark = component "benchmark" "" "Rust Module"
                scripts = component "scripts" "" "Rust Module"
                address = component "address" "" "Rust Module"
                generated = component "generated" "" "Rust Module"
                portStrategy = component "PortStrategy" "" "Rust Enum"
                serialRange = component "SerialRange" "" "Rust Struct"
                randomRange = component "RandomRange" "" "Rust Struct"
                rangeIterator = component "RangeIterator" "" "Rust Struct"
                scanner = component "Scanner" "" "Rust Struct"
                socketIterator = component "SocketIterator" "" "Rust Struct"
                script = component "Script" "" "Rust Struct"
                scriptFile = component "ScriptFile" "" "Rust Struct"
                scriptConfig = component "ScriptConfig" "" "Rust Struct"
            }
        }
    }

    views {
        systemContext beeSanRustScan "SystemContext" {
            include *
            autoLayout
        }
        container beeSanRustScan "Containers" {
            include *
            autoLayout
        }
        component rustscan "Components_rustscan" {
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
            element "External" {
                background "#999999"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Component" {
                background "#85bbf0"
                color "#000000"
            }
            element "Database" {
                shape Cylinder
                background "#438dd5"
                color "#ffffff"
            }
        }
    }
}
