//! # Rust Hello Architecture Plugin
//!
//! This educational plugin demonstrates key Rust concepts in WASM:
//! - `no_std` environment (no heap allocation, no standard library)
//! - Ownership and borrowing without a heap
//! - Zero-cost abstractions
//! - FFI (Foreign Function Interface) with the host
//! - String handling in WASM
//!
//! ## Key Learning Points:
//!
//! 1. **no_std**: We don't use Rust's standard library, which means:
//!    - No heap allocator (no Vec, String, HashMap, etc.)
//!    - No panic handler by default
//!    - Minimal binary size
//!
//! 2. **FFI with Host**: We import functions from the host environment
//!    using `extern "C"` blocks and the wasm_import_module attribute
//!
//! 3. **Memory Management**: All data lives on the stack or in static memory
//!
//! 4. **Zero-Cost Abstractions**: Rust's abstractions compile away to
//!    efficient WASM instructions

#![no_std]  // Don't link the standard library
#![no_main] // We define our own entry point

// Import host functions from the "env" module
// These are provided by the structurizr-rs WASM runtime
#[link(wasm_import_module = "env")]
extern "C" {
    /// Get the length of the workspace name
    /// This demonstrates read-only access to host data
    fn get_workspace_name_len() -> i32;

    /// Log a message to the host console
    /// Parameters: pointer to string data, length of string
    /// This demonstrates passing data from WASM to host
    fn log(ptr: i32, len: i32);
}

// =============================================================================
// PANIC HANDLER
// =============================================================================

/// Custom panic handler required in no_std environments
/// In WASM, we can't really recover from panics, so we just loop forever
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a production plugin, you might want to log the panic info
    // For now, we just enter an infinite loop
    loop {}
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Safe wrapper for logging messages to the host
///
/// This demonstrates:
/// - How to safely wrap unsafe FFI calls
/// - String handling without heap allocation
/// - Pointer casting for WASM interop
fn log_message(msg: &str) {
    unsafe {
        // Cast the string pointer to i32 (WASM uses 32-bit addressing)
        // The host will read `len` bytes starting from this address
        log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

/// Format a number as a string using a static buffer
///
/// This demonstrates:
/// - Working without heap allocation (no String type)
/// - Using static buffers for temporary data
/// - Manual string formatting
fn format_number(n: i32) -> &'static str {
    // Static buffer for formatting numbers
    // In no_std, we can't use format! macro or String::from
    static mut BUFFER: [u8; 32] = [0; 32];

    unsafe {
        // Reset buffer
        BUFFER = [0; 32];

        // Handle negative numbers
        let (mut num, negative) = if n < 0 {
            (-n as u32, true)
        } else {
            (n as u32, false)
        };

        // Convert number to string (reverse order initially)
        let mut i = 0;
        if num == 0 {
            BUFFER[0] = b'0';
            i = 1;
        } else {
            while num > 0 && i < 31 {
                BUFFER[i] = b'0' + (num % 10) as u8;
                num /= 10;
                i += 1;
            }
        }

        // Add negative sign if needed
        let start = if negative && i < 31 {
            BUFFER[i] = b'-';
            i += 1;
            0
        } else {
            0
        };

        // Reverse the string in place
        BUFFER[start..i].reverse();

        // Convert to &str
        core::str::from_utf8(&BUFFER[0..i]).unwrap_or("error")
    }
}

/// Count vowels in a string
///
/// This demonstrates:
/// - Pattern matching in Rust
/// - Iterator methods with zero-cost abstractions
/// - Character processing
fn count_vowels(s: &str) -> usize {
    s.chars().filter(|&c| {
        matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')
    }).count()
}

/// Calculate a "complexity score" based on string properties
///
/// This demonstrates:
/// - Multiple metrics calculation
/// - Rust's expression-based syntax
/// - Safe integer operations
fn calculate_complexity_score(name_len: i32) -> i32 {
    // Simple scoring algorithm (educational purposes)
    // Real complexity scoring would analyze the actual architecture

    let base_score = name_len * 10;

    // Bonus points for longer names (might indicate more detailed naming)
    let length_bonus = if name_len > 20 {
        50
    } else if name_len > 10 {
        25
    } else {
        0
    };

    // Cap the score at 1000
    core::cmp::min(base_score + length_bonus, 1000)
}

// =============================================================================
// MAIN PLUGIN LOGIC
// =============================================================================

/// Analyze the workspace and report statistics
///
/// This is the main logic of our plugin, demonstrating:
/// - Calling host functions
/// - Processing data without heap allocation
/// - Logging formatted output
fn analyze_workspace() {
    log_message("┌─────────────────────────────────────┐");
    log_message("│  🦀 Rust Hello Architecture Plugin   │");
    log_message("│  Educational WASM Demonstration      │");
    log_message("└─────────────────────────────────────┘");
    log_message("");

    // Get workspace name length from host
    let name_len = unsafe { get_workspace_name_len() };

    log_message("📊 Workspace Analysis:");
    log_message("──────────────────────");

    // Log the name length
    log_message("• Workspace name length: ");
    log_message(format_number(name_len));
    log_message(" characters");

    // Calculate and log complexity score
    let complexity = calculate_complexity_score(name_len);
    log_message("• Complexity score: ");
    log_message(format_number(complexity));
    log_message("/1000");

    // Categorize the workspace
    let category = match name_len {
        0..=5 => "Minimal",
        6..=10 => "Simple",
        11..=20 => "Standard",
        21..=30 => "Detailed",
        _ => "Comprehensive",
    };
    log_message("• Category: ");
    log_message(category);

    log_message("");
    log_message("🎓 Educational Notes:");
    log_message("──────────────────────");
    log_message("• This plugin runs with no heap allocation");
    log_message("• All strings are stack-allocated or static");
    log_message("• Binary size is minimal due to no_std");
    log_message("• Zero-cost abstractions compile to efficient WASM");

    log_message("");
    log_message("✅ Analysis complete!");
}

// =============================================================================
// ENTRY POINTS
// =============================================================================

/// WASI-style entry point
///
/// This is the primary entry point for WASM modules.
/// The structurizr-rs runtime will look for this function first.
#[no_mangle]
pub extern "C" fn _start() {
    log_message("🚀 Starting Rust WASM plugin (using _start entry point)");
    analyze_workspace();
}

/// Alternative entry point
///
/// If _start is not found, the runtime will try this function.
/// We include both for maximum compatibility.
#[no_mangle]
pub extern "C" fn run() {
    log_message("🚀 Starting Rust WASM plugin (using run entry point)");
    analyze_workspace();
}

// =============================================================================
// WASM MEMORY MANAGEMENT
// =============================================================================

/// Export memory allocator functions (even though we don't use heap)
/// These might be called by the host for memory introspection
#[no_mangle]
pub extern "C" fn __data_end() {}

#[no_mangle]
pub extern "C" fn __heap_base() {}

// =============================================================================
// RUST-SPECIFIC WASM EXPORTS
// =============================================================================

/// These functions are sometimes required by WASM runtimes
/// for Rust-compiled modules, even in no_std environments

#[no_mangle]
pub extern "C" fn __rust_alloc(_size: usize, _align: usize) -> *mut u8 {
    // We don't actually allocate in no_std
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn __rust_dealloc(_ptr: *mut u8, _size: usize, _align: usize) {
    // Nothing to deallocate in no_std
}

#[no_mangle]
pub extern "C" fn __rust_realloc(_ptr: *mut u8, _old_size: usize, _align: usize, _new_size: usize) -> *mut u8 {
    // No reallocation in no_std
    core::ptr::null_mut()
}