# Dark Mode Demo

This example demonstrates the dark mode theming capabilities in structurizr-rs.

## Features Demonstrated

- **Dark Background**: Using `background "#1a1a1a"` in view definitions
- **Custom Element Styles**: Color schemes optimized for dark backgrounds
- **Various Shapes**: WebBrowser, Hexagon, Cylinder, Pipe shapes
- **Relationship Styling**: Dashed lines with curved routing

## Quick Start

```bash
./serve.sh
```

This will start the web server and open the workspace in your browser.

## Structure

- `workspace.dsl` - The DSL workspace definition
- `docs/` - Documentation rendered in the web UI
- `adrs/` - Architecture Decision Records

## Architecture

The example models a banking system with:
- Web Application (React frontend)
- API Gateway
- Microservices (Authentication, Accounts, Payments, Notifications)
- Data stores (PostgreSQL, Redis, RabbitMQ)
- External integrations (Payment Gateway, Email System)

## See Also

- [docs/index.md](docs/index.md) - Full documentation
- [adrs/001-dark-mode-implementation.md](adrs/001-dark-mode-implementation.md) - Implementation decisions
