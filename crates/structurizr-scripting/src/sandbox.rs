//! Sandbox configuration for secure script execution.
//!
//! This module provides security controls for script execution to prevent
//! malicious or resource-intensive scripts from affecting the host system.

use std::time::Duration;

/// Configuration for the script execution sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum execution time before timeout (default: 5 seconds)
    pub timeout: Duration,

    /// Maximum memory usage in bytes (default: 10MB)
    pub max_memory: usize,

    /// Maximum number of instructions to execute (default: 1_000_000)
    pub max_instructions: u64,

    /// Allow file system access (default: false)
    pub allow_filesystem: bool,

    /// Allow network access (default: false)
    pub allow_network: bool,

    /// Allow loading external Lua modules (default: false)
    pub allow_require: bool,

    /// Allow os library functions (default: false)
    pub allow_os: bool,

    /// Allow io library functions (default: false)
    pub allow_io: bool,

    /// Allow debug library functions (default: false)
    pub allow_debug: bool,

    /// Allow package library functions (default: false)
    pub allow_package: bool,

    /// Custom allowed globals (beyond workspace API)
    pub allowed_globals: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_memory: 10 * 1024 * 1024, // 10MB
            max_instructions: 1_000_000,
            allow_filesystem: false,
            allow_network: false,
            allow_require: false,
            allow_os: false,
            allow_io: false,
            allow_debug: false,
            allow_package: false,
            allowed_globals: Vec::new(),
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permissive sandbox (for trusted scripts).
    ///
    /// **Warning**: Only use for scripts you fully trust.
    pub fn permissive() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            max_memory: 100 * 1024 * 1024, // 100MB
            max_instructions: 100_000_000,
            allow_filesystem: true,
            allow_network: false, // Still no network
            allow_require: true,
            allow_os: false, // Still no OS commands
            allow_io: true,
            allow_debug: false,
            allow_package: true,
            allowed_globals: Vec::new(),
        }
    }

    /// Set the execution timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum memory limit.
    pub fn with_max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = bytes;
        self
    }

    /// Set the maximum instruction count.
    pub fn with_max_instructions(mut self, count: u64) -> Self {
        self.max_instructions = count;
        self
    }

    /// Allow filesystem access.
    pub fn with_filesystem(mut self, allow: bool) -> Self {
        self.allow_filesystem = allow;
        self
    }

    /// Allow loading Lua modules with require().
    pub fn with_require(mut self, allow: bool) -> Self {
        self.allow_require = allow;
        self
    }

    /// Add custom allowed globals.
    pub fn with_allowed_global(mut self, name: impl Into<String>) -> Self {
        self.allowed_globals.push(name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.max_memory, 10 * 1024 * 1024);
        assert!(!config.allow_filesystem);
        assert!(!config.allow_network);
    }

    #[test]
    fn test_permissive_config() {
        let config = SandboxConfig::permissive();
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.allow_filesystem);
        assert!(!config.allow_network); // Network still disabled
    }

    #[test]
    fn test_builder_pattern() {
        let config = SandboxConfig::new()
            .with_timeout(Duration::from_secs(10))
            .with_max_memory(50 * 1024 * 1024)
            .with_filesystem(true);

        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_memory, 50 * 1024 * 1024);
        assert!(config.allow_filesystem);
    }
}
