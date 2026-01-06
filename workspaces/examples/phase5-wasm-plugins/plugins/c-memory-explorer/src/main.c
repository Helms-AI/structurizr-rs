/*
 * C Memory Explorer Plugin
 * ========================
 *
 * Educational WASM plugin demonstrating C's manual memory management.
 *
 * Key Learning Points:
 * - Manual memory allocation with malloc/free
 * - Pointer arithmetic and manipulation
 * - Stack vs heap memory in WASM
 * - C string handling without standard library
 * - FFI between C and WASM host
 * - Memory safety considerations
 */

// We're not using standard library to keep binary small
// These are custom implementations
typedef unsigned long size_t;
typedef int int32_t;

// =============================================================================
// WASM IMPORTS FROM HOST
// =============================================================================

// Import functions from the host environment
// These are provided by structurizr-rs runtime
__attribute__((import_module("env"), import_name("get_workspace_name_len")))
extern int32_t get_workspace_name_len(void);

__attribute__((import_module("env"), import_name("log")))
extern void host_log(const char* ptr, int32_t len);

// =============================================================================
// MEMORY MANAGEMENT
// =============================================================================

// Simple memory allocator for educational purposes
// In production, you'd use a more sophisticated allocator

#define HEAP_SIZE 4096  // 4KB heap for demonstration
static char heap[HEAP_SIZE];
static size_t heap_offset = 0;

/*
 * Simple bump allocator
 * - Allocates memory by incrementing a pointer
 * - No free() implementation (memory never reclaimed)
 * - Good for demonstration, not for production
 */
void* simple_malloc(size_t size) {
    // Align to 8 bytes for better performance
    size = (size + 7) & ~7;

    if (heap_offset + size > HEAP_SIZE) {
        // Out of memory!
        return 0;  // NULL
    }

    void* ptr = &heap[heap_offset];
    heap_offset += size;
    return ptr;
}

/*
 * Reset the heap (free all memory)
 * Since we're using a bump allocator, we just reset the pointer
 */
void reset_heap(void) {
    heap_offset = 0;
}

// =============================================================================
// STRING UTILITIES
// =============================================================================

/*
 * Calculate string length (like strlen)
 * Demonstrates pointer arithmetic
 */
size_t string_length(const char* str) {
    const char* p = str;
    while (*p) {  // Loop until null terminator
        p++;      // Pointer arithmetic
    }
    return p - str;  // Difference gives length
}

/*
 * Copy string (like strcpy)
 * Demonstrates manual memory copying
 */
void string_copy(char* dest, const char* src) {
    // Copy byte by byte including null terminator
    while ((*dest++ = *src++));
}

/*
 * Reverse a string in place
 * Demonstrates pointer manipulation and swapping
 */
void string_reverse(char* str) {
    if (!str || !*str) return;

    char* start = str;
    char* end = str;

    // Find end of string
    while (*end) {
        end++;
    }
    end--;  // Back up to last character

    // Swap characters from both ends
    while (start < end) {
        // XOR swap (no temporary variable needed!)
        *start ^= *end;
        *end ^= *start;
        *start ^= *end;

        start++;
        end--;
    }
}

/*
 * Convert integer to string
 * Demonstrates manual number formatting
 */
void int_to_string(int32_t num, char* buffer) {
    int i = 0;
    int is_negative = 0;

    // Handle negative numbers
    if (num < 0) {
        is_negative = 1;
        num = -num;
    }

    // Handle zero case
    if (num == 0) {
        buffer[i++] = '0';
        buffer[i] = '\0';
        return;
    }

    // Extract digits in reverse order
    while (num > 0) {
        buffer[i++] = '0' + (num % 10);
        num /= 10;
    }

    // Add negative sign if needed
    if (is_negative) {
        buffer[i++] = '-';
    }

    buffer[i] = '\0';

    // Reverse the string to get correct order
    string_reverse(buffer);
}

// =============================================================================
// CHECKSUM CALCULATION
// =============================================================================

/*
 * Calculate a simple checksum
 * Demonstrates byte-level operations
 */
unsigned int calculate_checksum(const char* data, size_t len) {
    unsigned int checksum = 0;

    for (size_t i = 0; i < len; i++) {
        // Rotate left by 1 bit and add byte
        checksum = ((checksum << 1) | (checksum >> 31)) + (unsigned char)data[i];
    }

    return checksum;
}

// =============================================================================
// LOGGING HELPERS
// =============================================================================

/*
 * Log a message to the host
 * Wrapper around the imported log function
 */
void log_message(const char* msg) {
    host_log(msg, string_length(msg));
}

/*
 * Log a number
 */
void log_number(int32_t num) {
    char buffer[32];
    int_to_string(num, buffer);
    log_message(buffer);
}

// =============================================================================
// MEMORY STATISTICS
// =============================================================================

/*
 * Display memory usage statistics
 * Shows educational information about memory layout
 */
void show_memory_stats(void) {
    log_message("\n📊 Memory Statistics:");
    log_message("───────────────────────");

    log_message("• Heap size: ");
    log_number(HEAP_SIZE);
    log_message(" bytes");

    log_message("• Heap used: ");
    log_number(heap_offset);
    log_message(" bytes");

    log_message("• Heap free: ");
    log_number(HEAP_SIZE - heap_offset);
    log_message(" bytes");

    log_message("• Heap address: 0x");
    // Convert pointer to integer for display
    unsigned long addr = (unsigned long)heap;
    char hex_buffer[20];
    int i = 0;

    // Convert to hexadecimal
    if (addr == 0) {
        hex_buffer[i++] = '0';
    } else {
        char temp[20];
        int j = 0;
        while (addr > 0) {
            int digit = addr % 16;
            temp[j++] = digit < 10 ? '0' + digit : 'A' + (digit - 10);
            addr /= 16;
        }
        // Reverse the hex string
        while (j > 0) {
            hex_buffer[i++] = temp[--j];
        }
    }
    hex_buffer[i] = '\0';
    log_message(hex_buffer);
}

// =============================================================================
// MAIN PLUGIN LOGIC
// =============================================================================

/*
 * Demonstrate memory operations
 */
void demonstrate_memory(void) {
    log_message("\n🔍 Memory Operations Demo:");
    log_message("──────────────────────────");

    // Get workspace name length
    int32_t name_len = get_workspace_name_len();

    log_message("\n1️⃣ Stack Allocation:");
    char stack_buffer[256];  // Stack allocated
    log_message("   • Created 256-byte stack buffer");

    log_message("\n2️⃣ Heap Allocation:");
    char* heap_buffer = (char*)simple_malloc(128);
    if (heap_buffer) {
        log_message("   • Allocated 128 bytes on heap");

        // Fill with test pattern
        for (int i = 0; i < 128; i++) {
            heap_buffer[i] = 'A' + (i % 26);
        }
        log_message("   • Filled with test pattern");
    }

    log_message("\n3️⃣ String Operations:");

    // Create a test string
    const char* test = "Hello WASM";
    string_copy(stack_buffer, test);
    log_message("   • Original: ");
    log_message(stack_buffer);

    // Reverse it
    string_reverse(stack_buffer);
    log_message("   • Reversed: ");
    log_message(stack_buffer);

    // Calculate checksum
    unsigned int checksum = calculate_checksum(test, string_length(test));
    log_message("   • Checksum: ");
    log_number(checksum);

    log_message("\n4️⃣ Pointer Arithmetic:");
    log_message("   • String length by pointer diff: ");
    log_number(string_length(test));
    log_message(" chars");

    log_message("\n5️⃣ Workspace Analysis:");
    log_message("   • Name length: ");
    log_number(name_len);
    log_message(" characters");
}

// =============================================================================
// EDUCATIONAL NOTES
// =============================================================================

void show_educational_notes(void) {
    log_message("\n🎓 C Memory Management in WASM:");
    log_message("────────────────────────────────");
    log_message("• Stack: Local variables, automatic cleanup");
    log_message("• Heap: Dynamic allocation, manual management");
    log_message("• Pointers: Direct memory addresses (32-bit in WASM)");
    log_message("• No garbage collection: Manual free() required");
    log_message("• Memory safety: Buffer overflows possible!");
    log_message("• Binary size: Minimal (~5KB) due to no stdlib");
}

// =============================================================================
// ENTRY POINTS
// =============================================================================

/*
 * WASI-style entry point
 * Called by the WASM runtime
 */
__attribute__((export_name("_start")))
void _start(void) {
    log_message("🔧 C Memory Explorer Plugin");
    log_message("============================");

    // Reset heap for fresh start
    reset_heap();

    // Run demonstrations
    demonstrate_memory();
    show_memory_stats();
    show_educational_notes();

    log_message("\n✅ Analysis complete!");
}

/*
 * Alternative entry point
 * For compatibility with different runtimes
 */
__attribute__((export_name("run")))
void run(void) {
    _start();
}

// =============================================================================
// EOF
// =============================================================================