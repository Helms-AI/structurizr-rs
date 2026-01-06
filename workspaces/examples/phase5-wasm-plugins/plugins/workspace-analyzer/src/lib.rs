//! Workspace Analyzer Plugin
//!
//! This is an example WASM plugin for structurizr-rs that demonstrates
//! how to write plugins in Rust that compile to WebAssembly.
//!
//! # Building
//!
//! ```bash
//! # Add WASM target (one-time setup)
//! rustup target add wasm32-unknown-unknown
//!
//! # Build the plugin
//! cargo build --release --target wasm32-unknown-unknown
//!
//! # Copy to plugin directory
//! cp target/wasm32-unknown-unknown/release/workspace_analyzer.wasm plugin.wasm
//! ```
//!
//! # Host Functions
//!
//! The plugin communicates with structurizr-rs through these host functions:
//!
//! - `get_workspace_name_len() -> i32` - Get the length of workspace name
//! - `log(ptr: i32, len: i32)` - Log a message to the console
//!
//! More host functions will be added as the plugin API expands.

#![no_std]

// Import host functions provided by structurizr-rs
// Note: #[link(wasm_import_module = "env")] is required for WASM to properly import
#[link(wasm_import_module = "env")]
extern "C" {
    /// Get the length of the workspace name.
    /// Requires `read_workspace` capability.
    fn get_workspace_name_len() -> i32;

    /// Log a message to the console.
    /// Always available (no capability required).
    fn log(ptr: i32, len: i32);
}

/// Helper function to log a string message.
fn log_message(msg: &str) {
    unsafe {
        log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

/// Plugin entry point.
///
/// This function is called by the structurizr-rs plugin engine when
/// the plugin is executed. The convention is to export either `_start`
/// (WASI-style) or `run` as the entry point.
#[no_mangle]
pub extern "C" fn _start() {
    log_message("=== Workspace Analyzer Plugin ===");
    log_message("Version: 1.0.0");
    log_message("");

    // Analyze workspace
    analyze_workspace();

    log_message("");
    log_message("=== Analysis Complete ===");
}

/// Alternative entry point (also supported).
#[no_mangle]
pub extern "C" fn run() {
    _start();
}

/// Perform workspace analysis.
fn analyze_workspace() {
    log_message("Analyzing workspace structure...");

    // Get workspace name length
    let name_len = unsafe { get_workspace_name_len() };

    // Log findings
    let mut buffer = [0u8; 64];
    let msg = format_message(&mut buffer, "Workspace name length: ", name_len);
    log_message(msg);

    // Future: Add more analysis when additional host functions are available
    // - Count people
    // - Count software systems
    // - Count containers
    // - Analyze relationships
    // - Check for orphaned elements
    // - Validate naming conventions
}

/// Simple number formatting without std.
/// Returns a string slice from the provided buffer.
fn format_message<'a>(buffer: &'a mut [u8], prefix: &str, number: i32) -> &'a str {
    let prefix_bytes = prefix.as_bytes();
    let prefix_len = prefix_bytes.len();

    // Copy prefix
    buffer[..prefix_len].copy_from_slice(prefix_bytes);

    // Convert number to string (simple implementation)
    let mut n = number;
    let mut digits = [0u8; 10];
    let mut digit_count = 0;

    if n == 0 {
        digits[0] = b'0';
        digit_count = 1;
    } else {
        let negative = n < 0;
        if negative {
            n = -n;
        }

        while n > 0 {
            digits[digit_count] = b'0' + (n % 10) as u8;
            n /= 10;
            digit_count += 1;
        }

        if negative {
            digits[digit_count] = b'-';
            digit_count += 1;
        }
    }

    // Reverse digits into buffer
    for i in 0..digit_count {
        buffer[prefix_len + i] = digits[digit_count - 1 - i];
    }

    let total_len = prefix_len + digit_count;

    // Safety: we only wrote ASCII bytes
    unsafe { core::str::from_utf8_unchecked(&buffer[..total_len]) }
}

// Panic handler for no_std environments (optional, for size optimization)
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    log_message("Plugin panic!");
    loop {}
}
