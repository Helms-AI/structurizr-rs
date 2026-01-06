# Rust Hello Architecture Plugin

An educational WASM plugin demonstrating Rust's unique capabilities in WebAssembly.

## 🎯 Learning Objectives

This plugin teaches:
- Writing `no_std` Rust for minimal WASM binaries
- FFI (Foreign Function Interface) between Rust and host
- Memory management without heap allocation
- Zero-cost abstractions in WASM
- Ownership and borrowing in constrained environments

## 🏗️ Architecture

```
┌─────────────────────┐
│   Host (Runtime)    │
├─────────────────────┤
│ - get_workspace_name│
│ - log()             │
└─────────┬───────────┘
          │ FFI
          │
┌─────────┴───────────┐
│   WASM Plugin       │
├─────────────────────┤
│ - No heap           │
│ - Stack only        │
│ - Static buffers    │
└─────────────────────┘
```

## 📚 Key Concepts

### 1. **no_std Environment**

```rust
#![no_std]  // No standard library
#![no_main] // Custom entry point
```

Benefits:
- Minimal binary size (~10KB)
- No heap allocator overhead
- Predictable memory usage
- Fast instantiation

Limitations:
- No `Vec`, `String`, `HashMap`
- No dynamic allocation
- Must handle panics manually

### 2. **FFI with Host**

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    fn get_workspace_name_len() -> i32;
    fn log(ptr: i32, len: i32);
}
```

- Functions imported from host environment
- Use `i32` for WASM pointers (32-bit address space)
- All calls are `unsafe` (crossing FFI boundary)

### 3. **Memory Management**

Without heap allocation, we use:
- **Stack allocation**: Local variables
- **Static buffers**: Pre-allocated memory
- **String slices**: References to existing data

Example:
```rust
static mut BUFFER: [u8; 32] = [0; 32]; // Static buffer
let local_var = 42;                     // Stack variable
let slice = &buffer[0..len];            // Borrowed slice
```

### 4. **Zero-Cost Abstractions**

Rust's high-level constructs compile to efficient WASM:

```rust
// This iterator code...
s.chars().filter(|&c| is_vowel(c)).count()

// ...compiles to the same efficiency as:
let mut count = 0;
for c in s.chars() {
    if is_vowel(c) { count += 1; }
}
```

## 🛠️ Building

### Prerequisites

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Add WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Build Commands

```bash
# Debug build (larger, with debug symbols)
cargo build --target wasm32-unknown-unknown

# Release build (optimized for size)
cargo build --target wasm32-unknown-unknown --release

# Copy to plugin.wasm
cp target/wasm32-unknown-unknown/release/rust_hello_arch.wasm plugin.wasm
```

### Build Optimization

The `Cargo.toml` includes size optimizations:

```toml
[profile.release]
opt-level = "s"    # Optimize for size
lto = true         # Link-time optimization
strip = true       # Remove debug symbols
codegen-units = 1  # Better optimization
panic = "abort"    # No unwinding code
```

Result: ~10KB WASM binary!

## 📊 Binary Analysis

Analyze the generated WASM:

```bash
# Check binary size
ls -lh plugin.wasm

# Examine exports
wasm-objdump -x plugin.wasm | grep Export

# View text format
wasm2wat plugin.wasm | head -50

# Count instructions
wasm-objdump -d plugin.wasm | wc -l
```

## 🔍 Comparison with Other Languages

| Feature | Rust | C | Go | AssemblyScript |
|---------|------|---|-----|----------------|
| Binary Size | ~10KB | ~5KB | ~20KB | ~15KB |
| Memory Safety | ✅ Compile-time | ❌ Manual | ✅ Runtime GC | ✅ Runtime |
| No Runtime | ✅ | ✅ | ❌ GC included | ❌ GC included |
| Type Safety | ✅ Strong | ⚠️ Weak | ✅ Strong | ✅ Strong |
| Learning Curve | Steep | Moderate | Easy | Easy |

## 🎓 Exercises

Try modifying the plugin to:

1. **Add Consonant Counting**:
   - Count consonants in addition to vowels
   - Calculate vowel/consonant ratio

2. **Improve Number Formatting**:
   - Add thousands separators (1,234)
   - Support floating-point display

3. **Add Pattern Detection**:
   - Check if name follows naming conventions
   - Detect CamelCase vs snake_case

4. **Memory Experiments**:
   - Try to use heap allocation (it will fail!)
   - Measure stack usage
   - Test buffer overflow protection

5. **Performance Testing**:
   - Add computation-heavy algorithm
   - Measure execution time
   - Compare with other languages

## 🐛 Common Issues

### "wasm32-unknown-unknown target not found"
```bash
rustup target add wasm32-unknown-unknown
```

### "Binary too large"
- Ensure release mode: `--release`
- Check optimization settings in Cargo.toml
- Remove unnecessary dependencies

### "Panic handler not found"
- Must define `#[panic_handler]` in no_std
- Can't use std panic machinery

### "Memory allocation failed"
- no_std means no heap!
- Use stack or static allocation only

## 📖 Resources

- [Rust WASM Book](https://rustwasm.github.io/book/)
- [no_std Handbook](https://docs.rust-embedded.org/book/intro/no-std.html)
- [WebAssembly Reference](https://webassembly.github.io/spec/)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)

## ✨ Key Takeaways

1. **Rust brings memory safety to WASM** without runtime overhead
2. **no_std produces tiny binaries** perfect for plugins
3. **Zero-cost abstractions** mean idiomatic code stays fast
4. **Ownership model** prevents memory bugs at compile time
5. **FFI is explicit** about unsafe boundaries

This plugin demonstrates that Rust can produce **safe, fast, and small** WASM modules, making it ideal for performance-critical plugins!