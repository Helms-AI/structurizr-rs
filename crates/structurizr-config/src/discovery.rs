//! Configuration file discovery
//!
//! Searches for configuration files in multiple locations
//! with a defined precedence order.

use std::path::{Path, PathBuf};
use tracing::debug;

/// Find the configuration file to use
///
/// Search order (first found wins):
/// 1. `$STRUCTURIZR_CONFIG` environment variable (explicit path)
/// 2. `./structurizr.toml` (project root)
/// 3. `$XDG_CONFIG_HOME/structurizr/config.toml` or `~/.config/structurizr/config.toml`
/// 4. `/etc/structurizr/config.toml` (system-wide)
pub fn find_config_file() -> Option<PathBuf> {
    // 1. Check explicit environment variable
    if let Ok(path) = std::env::var("STRUCTURIZR_CONFIG") {
        let path = PathBuf::from(path);
        if path.exists() {
            debug!("Using config from STRUCTURIZR_CONFIG: {:?}", path);
            return Some(path);
        } else {
            debug!("STRUCTURIZR_CONFIG points to non-existent file: {:?}", path);
        }
    }

    // 2. Check current directory
    let project_config = Path::new("structurizr.toml");
    if project_config.exists() {
        debug!("Using project config: {:?}", project_config);
        return Some(project_config.to_path_buf());
    }

    // 3. Check user config directory
    if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("structurizr/config.toml");
        if user_config.exists() {
            debug!("Using user config: {:?}", user_config);
            return Some(user_config);
        }
    }

    // 4. Check system config
    let system_config = Path::new("/etc/structurizr/config.toml");
    if system_config.exists() {
        debug!("Using system config: {:?}", system_config);
        return Some(system_config.to_path_buf());
    }

    debug!("No configuration file found, will use defaults");
    None
}

/// Get all config file locations that should be checked
///
/// Returns paths in priority order (highest to lowest)
pub fn get_config_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Explicit path from environment
    if let Ok(path) = std::env::var("STRUCTURIZR_CONFIG") {
        paths.push(PathBuf::from(path));
    }

    // Project root
    paths.push(PathBuf::from("structurizr.toml"));

    // User config
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("structurizr/config.toml"));
    }

    // System config
    paths.push(PathBuf::from("/etc/structurizr/config.toml"));

    paths
}

/// Check if running in a production environment
pub fn is_production() -> bool {
    std::env::var("STRUCTURIZR_ENV")
        .map(|v| v.to_lowercase() == "production")
        .unwrap_or(false)
}

/// Get the active profile name
pub fn get_active_profile() -> String {
    std::env::var("STRUCTURIZR_PROFILE")
        .unwrap_or_else(|_| {
            if is_production() {
                "production".to_string()
            } else {
                "development".to_string()
            }
        })
}