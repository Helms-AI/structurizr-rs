# Zig Pattern Matcher Plugin

An educational WASM plugin showcasing Zig's compile-time features and explicit control.

## 🎯 Learning Objectives

This plugin teaches:
- **Comptime**: Compile-time code execution
- **Error Unions**: Explicit error handling with `!T`
- **Optionals**: Nullable values with `?T`
- **Explicit Allocators**: No hidden memory allocation
- **No Hidden Control Flow**: What you see is what you get
- **First-Class WASM Support**: Excellent WASM target

## 🏗️ Architecture

```
┌─────────────────────┐
│   Zig Source Code   │
│   (comptime + runtime)
└─────────┬───────────┘
          │ Compile
          ↓
┌─────────────────────┐
│  Comptime Evaluated │ ← Lookup tables, constants
├─────────────────────┤
│  Runtime Code       │ ← Actual WASM instructions
└─────────┬───────────┘
          ↓
┌─────────────────────┐
│   WASM Binary ~8KB  │ (No runtime needed!)
└─────────────────────┘
```

## 📚 Key Concepts

### 1. **Comptime (Compile-Time Execution)**

Zig can execute code at compile time:

```zig
// This function runs at COMPILE TIME
fn comptimeFactorial(comptime n: u32) u32 {
    comptime {
        var result: u32 = 1;
        var i: u32 = 2;
        while (i <= n) : (i += 1) {
            result *= i;
        }
        return result;
    }
}

// Usage: computed at compile time, embedded as constant
const fact_10 = comptimeFactorial(10); // 3628800 in binary
```

**Benefits**:
- Zero runtime cost
- Guaranteed to complete (no infinite loops at runtime)
- Can generate code, types, and data structures

### 2. **Compile-Time Generated Data**

```zig
// Generate lookup table at compile time
fn generateLookupTable(comptime size: usize) [size]u32 {
    comptime {
        var table: [size]u32 = undefined;
        for (0..size) |i| {
            table[i] = @intCast(i * i);
        }
        return table;
    }
}

// Embedded in binary - no runtime computation!
const SQUARE_TABLE = generateLookupTable(256);
```

### 3. **Error Unions (`!T`)**

Explicit error handling without exceptions:

```zig
const PatternError = error{
    InvalidPattern,
    TooLong,
    EmptyInput,
};

// Function can return error or u32
fn validateLength(len: i32) PatternError!u32 {
    if (len < 0) return PatternError.InvalidPattern;
    if (len == 0) return PatternError.EmptyInput;
    if (len > 1000) return PatternError.TooLong;
    return @intCast(len);
}

// Usage with catch
const validated = validateLength(raw_len) catch |err| {
    // Handle specific error
    return;
};
```

### 4. **Optionals (`?T`)**

Nullable values without null pointer problems:

```zig
// Returns null or u32
fn findPattern(len: u32, target: u32) ?u32 {
    if (target == 0) return null;
    if (len % target == 0) return len / target;
    return null;
}

// Usage with if
if (findPattern(len, 3)) |result| {
    // result is guaranteed non-null here
    use(result);
} else {
    // Handle null case
}
```

### 5. **Explicit Allocators**

Zig never allocates memory implicitly:

```zig
// You must provide an allocator
const allocator = std.heap.page_allocator;

// Allocation is explicit
const data = try allocator.alloc(u8, 1024);
defer allocator.free(data); // Explicit cleanup

// No hidden allocations in string operations, etc.
```

### 6. **No Hidden Control Flow**

```zig
// What you see is what happens:
// - No exceptions thrown behind the scenes
// - No hidden function calls
// - No implicit type conversions

// Explicit integer casting required:
const big: i64 = 1000;
const small: i32 = @intCast(big); // Explicit!
```

## 🛠️ Building

### Prerequisites

Install Zig:

```bash
# Download from https://ziglang.org/download/

# macOS (Homebrew)
brew install zig

# Linux
wget https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz
tar xf zig-linux-x86_64-0.11.0.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64-0.11.0
```

### Build Commands

```bash
# Using build.zig
zig build

# Or directly
zig build-lib src/main.zig \
    -target wasm32-freestanding \
    -O ReleaseSmall \
    -femit-bin=plugin.wasm

# Check binary size
ls -lh plugin.wasm

# View WASM text format
wasm2wat plugin.wasm | head -50
```

### Optimization Levels

- `-O Debug`: Debug info, runtime safety checks
- `-O ReleaseSafe`: Optimized with safety checks
- `-O ReleaseFast`: Maximum speed
- `-O ReleaseSmall`: Minimum size (recommended for WASM)

## 🆚 Zig vs Other Languages

| Feature | Zig | Rust | C | Go |
|---------|-----|------|---|-----|
| **Comptime** | ✅ First-class | ⚠️ Macros only | ❌ None | ❌ None |
| **Memory Safety** | ⚠️ Runtime checks | ✅ Compile-time | ❌ None | ✅ GC |
| **Error Handling** | ✅ Error unions | ✅ Result<T,E> | ❌ Return codes | ⚠️ Error values |
| **Hidden Allocations** | ❌ None | ❌ None | ❌ None | ✅ GC managed |
| **Binary Size** | ✅ ~8KB | ✅ ~10KB | ✅ ~5KB | ⚠️ ~20KB |
| **WASM Support** | ✅ Excellent | ✅ Excellent | ⚠️ Via tools | ⚠️ TinyGo |

## 🔍 Comptime Deep Dive

### Type-Level Programming

```zig
// Create types at compile time
fn Matrix(comptime T: type, comptime rows: usize, comptime cols: usize) type {
    return [rows][cols]T;
}

const Mat3x3 = Matrix(f32, 3, 3);
var matrix: Mat3x3 = undefined;
```

### Compile-Time Strings

```zig
// String manipulation at compile time
fn comptimeConcat(comptime a: []const u8, comptime b: []const u8) *const [a.len + b.len]u8 {
    comptime {
        var result: [a.len + b.len]u8 = undefined;
        @memcpy(result[0..a.len], a);
        @memcpy(result[a.len..], b);
        return &result;
    }
}

const greeting = comptimeConcat("Hello, ", "World!"); // In binary
```

### Inline Loops

```zig
// Unroll loops at compile time
fn logStrings(comptime strings: []const []const u8) void {
    inline for (strings) |s| {
        logMessage(s); // Each call is separate in the binary
    }
}
```

## 🎓 Exercises

1. **Comptime Lookup Tables**
   - Generate prime number table
   - Create character category lookup
   - Build Fibonacci sequence

2. **Error Handling**
   - Add more error types
   - Create error context
   - Implement error logging

3. **Optional Patterns**
   - Chain optional operations
   - Create optional mapping
   - Handle multiple optionals

4. **Memory Management**
   - Use page_allocator
   - Implement arena allocator
   - Track allocations

5. **Type-Level Programming**
   - Create generic data structures
   - Implement compile-time validation
   - Generate specialized code

## 🐛 Troubleshooting

### "zig: command not found"
```bash
# Add to PATH
export PATH=$PATH:/path/to/zig
```

### "extern function not found"
Ensure host functions match exactly:
```zig
extern "env" fn log(ptr: [*]const u8, len: i32) void;
// Module: "env", Name: "log"
```

### "comptime evaluation exceeded"
Reduce comptime computation complexity or increase limits.

### Large binary size
```bash
# Use ReleaseSmall
zig build-lib src/main.zig -target wasm32-freestanding -O ReleaseSmall
```

## 📖 Resources

- [Zig Documentation](https://ziglang.org/documentation/master/)
- [Zig WASM Guide](https://ziglang.org/documentation/master/#WebAssembly)
- [Comptime Deep Dive](https://ziglearn.org/chapters/1/)
- [Error Handling Guide](https://ziglang.org/documentation/master/#Errors)
- [Zig Style Guide](https://ziglang.org/documentation/master/#Style-Guide)

## ✨ Key Takeaways

1. **Comptime is revolutionary** - run code at compile time, zero runtime cost
2. **Error unions are explicit** - no hidden exceptions
3. **Optionals prevent null bugs** - safe nullable values
4. **No hidden allocations** - you control all memory
5. **First-class WASM** - excellent toolchain support
6. **Tiny binaries** - ~8KB without runtime

This plugin demonstrates Zig's **unique compile-time capabilities** and **explicit control philosophy**, making it ideal for:
- Performance-critical code
- Systems programming
- Embedded/WASM targets
- When you need predictable behavior

Zig's philosophy: **"No hidden behavior, no hidden costs"**!