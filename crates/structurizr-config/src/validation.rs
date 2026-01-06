//! Configuration validation
//!
//! Validates configuration values for correctness and consistency.

use super::{Config, ConfigError};

/// Validate a configuration
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Validate server configuration
    validate_server(&config.server)?;

    // Validate MCP configuration
    validate_mcp(&config.mcp)?;

    // Validate collaboration configuration
    validate_collaboration(&config.collaboration)?;

    // Validate workspace scope patterns
    validate_workspace_scope(&config.mcp.workspace_scope)?;

    Ok(())
}

/// Validate server configuration
fn validate_server(server: &super::ServerConfig) -> Result<(), ConfigError> {
    // Validate port number
    if server.port == 0 {
        return Err(ConfigError::Validation(
            "Server port must be greater than 0".to_string()
        ));
    }

    // Validate host
    if server.host.is_empty() {
        return Err(ConfigError::Validation(
            "Server host cannot be empty".to_string()
        ));
    }

    // Validate workspaces directory exists
    if !server.workspaces_dir.exists() {
        return Err(ConfigError::Validation(
            format!("Workspaces directory does not exist: {:?}", server.workspaces_dir)
        ));
    }

    Ok(())
}

/// Validate MCP configuration
fn validate_mcp(mcp: &super::McpConfig) -> Result<(), ConfigError> {
    // Validate MCP server port
    if mcp.server.port == 0 {
        return Err(ConfigError::Validation(
            "MCP server port must be greater than 0".to_string()
        ));
    }

    // Validate transport type
    let valid_transports = ["stdio", "websocket", "ws", "sse", "http"];
    if !valid_transports.contains(&mcp.server.transport.as_str()) {
        return Err(ConfigError::Validation(
            format!("Invalid MCP transport: {}. Must be one of: {:?}",
                    mcp.server.transport, valid_transports)
        ));
    }

    // Validate health check interval
    if mcp.server.health_check_interval_ms == 0 {
        return Err(ConfigError::Validation(
            "Health check interval must be greater than 0".to_string()
        ));
    }

    // Validate proxy configuration
    if mcp.proxy.timeout_ms == 0 {
        return Err(ConfigError::Validation(
            "MCP proxy timeout must be greater than 0".to_string()
        ));
    }

    Ok(())
}

/// Validate collaboration configuration
fn validate_collaboration(collab: &super::CollaborationConfig) -> Result<(), ConfigError> {
    // Validate notification transport
    let valid_transports = ["sse", "websocket", "ws"];
    if !valid_transports.contains(&collab.notification_transport.as_str()) {
        return Err(ConfigError::Validation(
            format!("Invalid notification transport: {}. Must be one of: {:?}",
                    collab.notification_transport, valid_transports)
        ));
    }

    // Validate CRDT configuration
    if collab.crdt.enabled {
        let valid_algorithms = ["yjs", "automerge", "custom"];
        if !valid_algorithms.contains(&collab.crdt.algorithm.as_str()) {
            return Err(ConfigError::Validation(
                format!("Invalid CRDT algorithm: {}. Must be one of: {:?}",
                        collab.crdt.algorithm, valid_algorithms)
            ));
        }

        if collab.crdt.sync_interval_ms == 0 {
            return Err(ConfigError::Validation(
                "CRDT sync interval must be greater than 0".to_string()
            ));
        }
    }

    // Validate file watch configuration
    if collab.file_watch.enabled {
        if collab.file_watch.debounce_ms == 0 {
            return Err(ConfigError::Validation(
                "File watch debounce must be greater than 0".to_string()
            ));
        }

        if collab.file_watch.patterns.is_empty() {
            return Err(ConfigError::Validation(
                "File watch patterns cannot be empty when file watching is enabled".to_string()
            ));
        }
    }

    Ok(())
}

/// Validate workspace scope configuration
fn validate_workspace_scope(scope: &super::scope::WorkspaceScope) -> Result<(), ConfigError> {
    use super::scope::ScopeMode;

    // If mode is Allow or Deny, patterns should not be empty
    match scope.mode {
        ScopeMode::Allow | ScopeMode::Deny => {
            if scope.patterns.is_empty() {
                return Err(ConfigError::Validation(
                    format!("Workspace scope patterns cannot be empty for mode: {:?}", scope.mode)
                ));
            }
        }
        ScopeMode::All => {}
    }

    // Validate glob patterns
    for pattern in &scope.patterns {
        // Strip negation prefix if present
        let pattern = pattern.strip_prefix('!').unwrap_or(pattern);

        // Try to compile as glob
        if let Err(e) = globset::Glob::new(pattern) {
            return Err(ConfigError::Validation(
                format!("Invalid glob pattern '{}': {}", pattern, e)
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let workspaces_dir = temp_dir.path().join("workspaces");
        fs::create_dir(&workspaces_dir).unwrap();

        let mut config = Config::default();
        config.server.workspaces_dir = workspaces_dir;

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut config = Config::default();
        config.server.port = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port"));
    }

    #[test]
    fn test_validate_invalid_transport() {
        let mut config = Config::default();
        config.mcp.server.transport = "invalid".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("transport"));
    }

    #[test]
    fn test_validate_invalid_glob_pattern() {
        let mut config = Config::default();
        config.mcp.workspace_scope.mode = super::super::scope::ScopeMode::Allow;
        config.mcp.workspace_scope.patterns = vec!["[invalid".to_string()];

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("glob"));
    }
}