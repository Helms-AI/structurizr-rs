// Zig Pattern Matcher Plugin
// ==========================
//
// Educational WASM plugin demonstrating Zig's unique features:
// - Comptime (compile-time execution)
// - Explicit error handling with error unions
// - Manual memory management with allocators
// - No hidden control flow
// - First-class WASM support
//
// Key Learning Points:
// - comptime: Code that runs at compile time
// - Error unions: ?T and !T for optional/error handling
// - Allocators: Explicit memory management
// - No hidden allocations or control flow
// - Safety with runtime checks (debug mode)

// =============================================================================
// WASM IMPORTS FROM HOST
// =============================================================================

/// Import host functions using extern declarations
/// These are provided by the structurizr-rs runtime
extern "env" fn get_workspace_name_len() i32;
extern "env" fn log(ptr: [*]const u8, len: i32) void;

// =============================================================================
// LOGGING UTILITIES
// =============================================================================

/// Log a message to the host console
/// Demonstrates Zig's slice syntax
fn logMessage(msg: []const u8) void {
    log(msg.ptr, @intCast(msg.len));
}

/// Log multiple strings in sequence
/// Demonstrates comptime iteration
fn logStrings(comptime strings: []const []const u8) void {
    inline for (strings) |s| {
        logMessage(s);
    }
}

// =============================================================================
// COMPTIME DEMONSTRATIONS
// =============================================================================

/// Compile-time string concatenation
/// This runs at compile time - no runtime cost!
/// Returns a pointer to a comptime-computed constant string
fn comptimeConcat(comptime a: []const u8, comptime b: []const u8) []const u8 {
    const concat_result = a ++ b;
    return concat_result;
}

/// Compile-time factorial calculation
/// Result is computed during compilation
fn comptimeFactorial(comptime n: u32) comptime_int {
    comptime {
        var result: u32 = 1;
        var i: u32 = 2;
        while (i <= n) : (i += 1) {
            result *= i;
        }
        return result;
    }
}

/// Compile-time Fibonacci
fn comptimeFibonacci(comptime n: u32) comptime_int {
    comptime {
        if (n <= 1) return n;

        var prev: u32 = 0;
        var curr: u32 = 1;
        var i: u32 = 2;
        while (i <= n) : (i += 1) {
            const next = prev + curr;
            prev = curr;
            curr = next;
        }
        return curr;
    }
}

/// Generate a lookup table at compile time
/// The entire table is embedded in the binary
fn generateLookupTable(comptime size: usize) [size]u32 {
    return comptime blk: {
        var table: [size]u32 = undefined;
        for (0..size) |i| {
            table[i] = @intCast(i * i); // Square values
        }
        break :blk table;
    };
}

// Compile-time generated lookup table
const SQUARE_TABLE = generateLookupTable(16);

// =============================================================================
// ERROR HANDLING
// =============================================================================

/// Custom error type
/// Demonstrates Zig's error union system
const PatternError = error{
    InvalidPattern,
    TooLong,
    EmptyInput,
    OutOfBounds,
};

/// Result type using error union
/// !T means the function can return an error or T
fn validateLength(len: i32) PatternError!u32 {
    if (len < 0) return PatternError.InvalidPattern;
    if (len == 0) return PatternError.EmptyInput;
    if (len > 1000) return PatternError.TooLong;
    return @intCast(len);
}

/// Optional type demonstration
/// ?T means the value can be null or T
fn findPattern(len: u32, target: u32) ?u32 {
    // Simple pattern: check if length is divisible by target
    if (target == 0) return null;
    if (len % target == 0) return len / target;
    return null;
}

// =============================================================================
// PATTERN MATCHING
// =============================================================================

/// Pattern categories based on length
const Category = enum {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,

    /// Convert category to string (comptime known)
    fn toString(self: Category) []const u8 {
        return switch (self) {
            .Tiny => "Tiny",
            .Small => "Small",
            .Medium => "Medium",
            .Large => "Large",
            .Huge => "Huge",
        };
    }

    /// Create category from length
    fn fromLength(len: u32) Category {
        return if (len <= 5) .Tiny else if (len <= 10) .Small else if (len <= 20) .Medium else if (len <= 30) .Large else .Huge;
    }
};

/// Score calculation
fn calculateScore(len: u32) u32 {
    const base_score = len * 10;
    const bonus: u32 = if (len > 20) 50 else if (len > 10) 25 else 0;
    return @min(base_score + bonus, 1000);
}

// =============================================================================
// INTEGER TO STRING CONVERSION
// =============================================================================

/// Convert integer to string using fixed buffer
/// Demonstrates explicit buffer management
fn intToString(value: i32, buffer: []u8) []const u8 {
    if (buffer.len == 0) return "";

    var val: u32 = if (value < 0) @intCast(-value) else @intCast(value);
    var idx: usize = buffer.len;

    // Handle zero case
    if (val == 0) {
        if (idx > 0) {
            idx -= 1;
            buffer[idx] = '0';
        }
        return buffer[idx..];
    }

    // Convert digits (reverse order)
    while (val > 0 and idx > 0) {
        idx -= 1;
        buffer[idx] = @intCast('0' + (val % 10));
        val /= 10;
    }

    // Add negative sign if needed
    if (value < 0 and idx > 0) {
        idx -= 1;
        buffer[idx] = '-';
    }

    return buffer[idx..];
}

// =============================================================================
// MAIN ANALYSIS
// =============================================================================

/// Main analysis function
fn analyze() void {
    logMessage("+-----------------------------------------+");
    logMessage("|  Zig Pattern Matcher Plugin             |");
    logMessage("|  Comptime & Explicit Control Demo       |");
    logMessage("+-----------------------------------------+");

    // Get workspace name length
    const raw_len = get_workspace_name_len();

    logMessage("");
    logMessage("Workspace Analysis:");
    logMessage("--------------------");

    // Use buffer for number conversion
    var buffer: [32]u8 = undefined;

    logMessage("* Name length: ");
    logMessage(intToString(raw_len, &buffer));
    logMessage(" characters");

    // Validate and handle errors
    const validated = validateLength(raw_len) catch |err| {
        logMessage("* Error: ");
        logMessage(switch (err) {
            PatternError.InvalidPattern => "Invalid pattern",
            PatternError.TooLong => "Too long",
            PatternError.EmptyInput => "Empty input",
            PatternError.OutOfBounds => "Out of bounds",
        });
        return;
    };

    // Category
    const category = Category.fromLength(validated);
    logMessage("* Category: ");
    logMessage(category.toString());

    // Score
    const score = calculateScore(validated);
    logMessage("* Score: ");
    logMessage(intToString(@intCast(score), &buffer));
    logMessage("/1000");

    // Comptime demonstrations
    logMessage("");
    logMessage("Comptime Demonstrations:");
    logMessage("-------------------------");

    // These values are computed at compile time!
    const fact_10 = comptimeFactorial(10);
    logMessage("* Factorial(10) [comptime]: ");
    logMessage(intToString(@intCast(fact_10), &buffer));

    const fib_10 = comptimeFibonacci(10);
    logMessage("* Fibonacci(10) [comptime]: ");
    logMessage(intToString(@intCast(fib_10), &buffer));

    // Compile-time string
    const greeting = comptimeConcat("Hello, ", "WASM!");
    logMessage("* Comptime concat: ");
    logMessage(greeting);

    // Lookup table (generated at compile time)
    logMessage("* Square table[5] [comptime]: ");
    logMessage(intToString(@intCast(SQUARE_TABLE[5]), &buffer));

    // Pattern matching
    logMessage("");
    logMessage("Pattern Matching:");
    logMessage("------------------");

    // Optional handling
    if (findPattern(validated, 3)) |result| {
        logMessage("* Divisible by 3, quotient: ");
        logMessage(intToString(@intCast(result), &buffer));
    } else {
        logMessage("* Not divisible by 3");
    }

    if (findPattern(validated, 5)) |result| {
        logMessage("* Divisible by 5, quotient: ");
        logMessage(intToString(@intCast(result), &buffer));
    } else {
        logMessage("* Not divisible by 5");
    }

    // Educational notes
    logMessage("");
    logMessage("Zig Features Demonstrated:");
    logMessage("---------------------------");
    logMessage("* comptime: Compile-time evaluation");
    logMessage("* Error unions: !T for error handling");
    logMessage("* Optionals: ?T for nullable values");
    logMessage("* Explicit allocators (not used here)");
    logMessage("* No hidden control flow");
    logMessage("* First-class WASM support");
    logMessage("* ~8KB binary (no runtime needed)");

    logMessage("");
    logMessage("Analysis complete!");
}

// =============================================================================
// ENTRY POINTS
// =============================================================================

/// WASI-style entry point
export fn _start() void {
    logMessage("Starting Zig plugin (_start)");
    analyze();
}

/// Alternative entry point
export fn run() void {
    logMessage("Starting Zig plugin (run)");
    analyze();
}

// =============================================================================
// MEMORY MANAGEMENT NOTES
// =============================================================================
//
// Zig requires explicit memory management via allocators.
// In this WASM plugin, we avoid dynamic allocation entirely:
//
// 1. Use stack buffers (var buffer: [32]u8)
// 2. Use comptime computed values
// 3. Use slices to reference existing data
//
// If dynamic allocation were needed, you would:
// 1. Use std.heap.page_allocator (WASM page allocator)
// 2. Pass allocator to functions that need it
// 3. Free memory explicitly
//
// Example (not used in this plugin):
//
// const allocator = std.heap.page_allocator;
// const data = try allocator.alloc(u8, 1024);
// defer allocator.free(data);
//