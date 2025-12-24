workspace "Markdown Demo" "Demonstrates all supported Markdown features in structurizr-rs documentation" {

    !docs "docs"
    !adrs "adrs"

    model {
        user = person "Documentation Reader" "Views and tests markdown rendering"

        markdownSystem = softwareSystem "Markdown Renderer" "Renders GitHub Flavored Markdown to HTML using comrak" {
            parser = container "comrak Parser" "CommonMark + GFM parser" "Rust"
            renderer = container "HTML Renderer" "Converts AST to styled HTML" "Rust"
        }

        user -> markdownSystem "Reads documentation"
        parser -> renderer "Emits AST nodes"
    }

    views {
        systemContext markdownSystem "Context" "Markdown rendering context" {
            include *
            autoLayout tb
        }

        container markdownSystem "Containers" "Markdown rendering components" {
            include *
            autoLayout tb
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
        }
    }
}
