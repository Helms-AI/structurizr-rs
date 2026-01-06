/**
 * AssemblyScript Type-Safe Greeter Plugin
 * ========================================
 *
 * This educational plugin demonstrates how TypeScript developers
 * can create WASM modules using familiar syntax.
 *
 * Key Learning Points:
 * - TypeScript-like syntax compiling to WASM
 * - Type annotations and inference
 * - Managed memory with garbage collection
 * - String handling in AssemblyScript
 * - Decorators and WASM exports
 * - Differences from regular TypeScript
 */

// =============================================================================
// WASM IMPORTS FROM HOST
// =============================================================================

/**
 * Import host functions using declare statements
 * These are provided by the structurizr-rs runtime
 */
@external("env", "get_workspace_name_len")
declare function get_workspace_name_len(): i32;

@external("env", "log")
declare function log(ptr: i32, len: i32): void;

// =============================================================================
// TYPE DEFINITIONS
// =============================================================================

/**
 * Greeting configuration with TypeScript types
 * Shows how interfaces work in AssemblyScript
 */
class GreetingConfig {
    prefix: string;
    suffix: string;
    enthusiasm: i32;  // Note: i32 instead of number!

    constructor(prefix: string = "Hello", suffix: string = "!", enthusiasm: i32 = 1) {
        this.prefix = prefix;
        this.suffix = suffix;
        this.enthusiasm = enthusiasm;
    }
}

/**
 * Statistics tracker showing typed class usage
 */
class Statistics {
    callCount: i32 = 0;
    totalLength: i32 = 0;
    averageLength: f32 = 0.0;  // f32 for float

    addSample(length: i32): void {
        this.callCount++;
        this.totalLength += length;
        this.averageLength = <f32>this.totalLength / <f32>this.callCount;
    }
}

// Global statistics instance
const stats = new Statistics();

// =============================================================================
// STRING UTILITIES
// =============================================================================

/**
 * Helper to log messages to the host
 * Shows string handling in AssemblyScript
 *
 * Note: AssemblyScript ArrayBuffer has a header before data.
 * We use String.UTF8.encode which returns an ArrayBuffer,
 * then get the actual data pointer using changetype with
 * the correct offset (ArrayBuffer header is 16 bytes in AS).
 */
function logMessage(message: string): void {
    // AssemblyScript strings are UTF-16 internally
    // We need to get the UTF-8 bytes for the host
    const buffer = String.UTF8.encode(message);
    // ArrayBuffer data starts after header - in AS the header is implementation detail
    // Use Uint8Array to get direct data access
    const view = Uint8Array.wrap(buffer);
    const ptr = view.dataStart;
    log(ptr, view.length);
}

/**
 * Create a repeated string
 * Demonstrates string operations
 */
function repeatString(str: string, count: i32): string {
    let result = "";
    for (let i = 0; i < count; i++) {
        result += str;
    }
    return result;
}

/**
 * Format a number with type safety
 * Shows number to string conversion
 */
function formatNumber<T extends number>(value: T): string {
    // AssemblyScript has toString() for numbers
    return value.toString();
}

// =============================================================================
// GREETING GENERATION
// =============================================================================

/**
 * Generate a greeting message
 * Demonstrates class methods and string interpolation
 */
function generateGreeting(config: GreetingConfig, nameLength: i32): string {
    // Build excitement string
    const excitement = repeatString("!", config.enthusiasm);

    // Create greeting parts
    const parts: string[] = [
        config.prefix,
        " from AssemblyScript",
        config.suffix,
        excitement
    ];

    // Join parts (no built-in join, so we do it manually)
    let greeting = "";
    for (let i = 0; i < parts.length; i++) {
        greeting += parts[i];
    }

    return greeting;
}

/**
 * Categorize workspace by name length
 * Shows switch statements and enums
 */
function categorizeWorkspace(length: i32): string {
    // AssemblyScript doesn't have enums, use constants
    if (length <= 5) return "Tiny";
    if (length <= 10) return "Small";
    if (length <= 20) return "Medium";
    if (length <= 30) return "Large";
    return "Huge";
}

// =============================================================================
// MATHEMATICAL OPERATIONS
// =============================================================================

/**
 * Calculate factorial (demonstrates recursion)
 */
function factorial(n: i32): i32 {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

/**
 * Calculate Fibonacci (demonstrates iteration)
 */
function fibonacci(n: i32): i32 {
    if (n <= 1) return n;

    let prev = 0;
    let curr = 1;

    for (let i = 2; i <= n; i++) {
        const next = prev + curr;
        prev = curr;
        curr = next;
    }

    return curr;
}

// =============================================================================
// TYPE DEMONSTRATIONS
// =============================================================================

/**
 * Demonstrate AssemblyScript's type system
 */
function demonstrateTypes(): void {
    logMessage("\n📘 TypeScript → WASM Type Mapping:");
    logMessage("────────────────────────────────────");

    // Integer types (not available in regular TypeScript!)
    const int8: i8 = 127;           // -128 to 127
    const uint8: u8 = 255;          // 0 to 255
    const int32: i32 = 2147483647;  // 32-bit signed
    const uint32: u32 = 4294967295; // 32-bit unsigned
    const int64: i64 = 9223372036854775807; // 64-bit signed

    logMessage("• i8 max: " + int8.toString());
    logMessage("• u8 max: " + uint8.toString());
    logMessage("• i32 max: " + int32.toString());
    logMessage("• u32 max: " + uint32.toString());

    // Float types
    const float32: f32 = 3.14159;   // 32-bit float
    const float64: f64 = 2.718281828459045; // 64-bit float

    logMessage("• f32 pi: " + float32.toString());
    logMessage("• f64 e: " + float64.toString());

    // Boolean (same as TypeScript)
    const bool: boolean = true;
    logMessage("• boolean: " + bool.toString());

    // Arrays (typed!)
    const numbers: i32[] = [1, 2, 3, 4, 5];
    logMessage("• Array length: " + numbers.length.toString());

    // Type casting
    const floatVal: f32 = 10.7;
    const intVal: i32 = <i32>floatVal;  // Explicit cast
    logMessage("• Cast f32(10.7) to i32: " + intVal.toString());
}

// =============================================================================
// MAIN PLUGIN LOGIC
// =============================================================================

/**
 * Main analysis function
 */
function analyze(): void {
    logMessage("┌──────────────────────────────────────┐");
    logMessage("│  📘 AssemblyScript Type-Safe Greeter │");
    logMessage("│  TypeScript → WASM Demonstration     │");
    logMessage("└──────────────────────────────────────┘");

    logMessage("");
    logMessage("Getting workspace name length...");

    // Get workspace name length
    const nameLength = get_workspace_name_len();

    logMessage("Got length: " + nameLength.toString());
    stats.addSample(nameLength);

    logMessage("\n🔍 Workspace Analysis:");
    logMessage("──────────────────────");
    logMessage("• Name length: " + nameLength.toString() + " characters");
    logMessage("• Category: " + categorizeWorkspace(nameLength));

    // Generate greetings with different configs
    logMessage("\n👋 Greetings Demo:");
    logMessage("──────────────────");

    const configs: GreetingConfig[] = [
        new GreetingConfig("Hello", "!", 1),
        new GreetingConfig("Hi", "!!", 2),
        new GreetingConfig("Welcome", "!!!", 3)
    ];

    for (let i = 0; i < configs.length; i++) {
        const greeting = generateGreeting(configs[i], nameLength);
        logMessage("• " + greeting);
    }

    // Math demonstrations
    logMessage("\n🔢 Math Functions:");
    logMessage("──────────────────");
    const n: i32 = nameLength < 10 ? nameLength : 10;  // Cap at 10 for performance
    logMessage("• Factorial(" + n.toString() + ") = " + factorial(n).toString());
    logMessage("• Fibonacci(" + n.toString() + ") = " + fibonacci(n).toString());

    // Type system demo
    demonstrateTypes();

    // Statistics
    logMessage("\n📊 Statistics:");
    logMessage("──────────────────");
    logMessage("• Total calls: " + stats.callCount.toString());
    logMessage("• Average length: " + stats.averageLength.toString());

    // Educational notes
    logMessage("\n🎓 AssemblyScript Features:");
    logMessage("────────────────────────────");
    logMessage("• TypeScript syntax with WASM types");
    logMessage("• Garbage collection included");
    logMessage("• Strong typing at compile time");
    logMessage("• ~15KB binary with runtime");
    logMessage("• Familiar to web developers");

    logMessage("\n✅ Analysis complete!");
}

// =============================================================================
// ENTRY POINTS
// =============================================================================

/**
 * Main entry point for the plugin
 * Note: _start is reserved in AssemblyScript, using 'main' instead
 */
export function main(): void {
    logMessage("🚀 Starting AssemblyScript plugin (main)");
    analyze();
}

/**
 * Alternative entry point
 */
export function run(): void {
    logMessage("🚀 Starting AssemblyScript plugin (run)");
    analyze();
}

// =============================================================================
// MEMORY MANAGEMENT (OPTIONAL EXPORTS)
// =============================================================================

/**
 * AssemblyScript manages memory automatically,
 * but we can export memory functions for debugging
 */
export function getHeapSize(): i32 {
    return memory.size() * 65536;  // Pages to bytes
}

export function getAllocatedMemory(): i32 {
    // This is a simplified version
    return <i32>__heap_base;
}

// AssemblyScript built-in memory management
// No manual malloc/free needed!