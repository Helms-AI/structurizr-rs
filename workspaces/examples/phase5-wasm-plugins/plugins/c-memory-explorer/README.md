# C Memory Explorer Plugin

An educational WASM plugin demonstrating C's manual memory management and pointer arithmetic.

## 🎯 Learning Objectives

This plugin teaches:
- Manual memory allocation (malloc/free patterns)
- Pointer arithmetic and manipulation
- Stack vs heap memory in WASM
- C string handling without stdlib
- FFI between C and WASM host
- Memory safety considerations

## 🏗️ Architecture

```
┌─────────────────────┐
│    Linear Memory    │
├─────────────────────┤
│ Stack (automatic)   │ ← Local variables
├─────────────────────┤
│ Heap (manual)       │ ← malloc/free
├─────────────────────┤
│ Data segment        │ ← Static data
└─────────────────────┘
         ↕ FFI
┌─────────────────────┐
│   Host Functions    │
└─────────────────────┘
```

## 📚 Key Concepts

### 1. **Manual Memory Management**

C requires explicit memory management:

```c
// Allocation
char* buffer = (char*)malloc(256);
if (!buffer) {
    // Handle allocation failure
}

// Use the memory
strcpy(buffer, "Hello");

// Must free when done!
free(buffer);
```

In WASM, we implement a simple bump allocator:
- Memory allocated sequentially
- No fragmentation handling
- No actual free() (for simplicity)

### 2. **Pointer Arithmetic**

C allows direct pointer manipulation:

```c
char* str = "Hello";
char* p = str;

while (*p) {      // Dereference pointer
    p++;          // Move to next byte
}

size_t len = p - str;  // Calculate distance
```

WASM uses 32-bit pointers (i32 type).

### 3. **Stack vs Heap**

```c
// Stack allocation (automatic)
char stack_buffer[256];  // Cleaned up automatically

// Heap allocation (manual)
char* heap_buffer = malloc(256);  // Must call free()
```

Stack:
- Fast allocation/deallocation
- Limited size
- Automatic cleanup

Heap:
- Dynamic size
- Manual management
- Can cause leaks if not freed

### 4. **String Handling**

Without stdlib, we implement basics:

```c
// String length
size_t strlen(const char* s) {
    const char* p = s;
    while (*p) p++;
    return p - s;
}

// String copy
void strcpy(char* dest, const char* src) {
    while ((*dest++ = *src++));
}
```

### 5. **FFI Attributes**

```c
// Import from host
__attribute__((import_module("env"), import_name("log")))
extern void log(const char* ptr, int32_t len);

// Export to host
__attribute__((export_name("_start")))
void _start(void) { ... }
```

## 🛠️ Building

### Prerequisites

Install Emscripten:

```bash
# Download and install
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

### Build Commands

```bash
# Check installation
make check

# Build the plugin
make

# Or directly with emcc
emcc src/main.c -o plugin.wasm \
    -s STANDALONE_WASM \
    -s EXPORTED_FUNCTIONS='["_start","run"]' \
    -O3
```

### Compilation Flags

- `-s STANDALONE_WASM`: No JavaScript glue code
- `-s EXPORTED_FUNCTIONS`: Functions visible to host
- `-O3`: Maximum optimization
- `--no-entry`: No main() function needed

## 📊 Memory Layout

WASM linear memory (32-bit address space):

```
0x0000 ┌──────────────┐
       │ NULL guard   │ (Trap on access)
0x0400 ├──────────────┤
       │ Data segment │ (Static strings)
0x1000 ├──────────────┤
       │ Stack        │ (Grows down ↓)
       │     ...      │
       │     ...      │
       │ Heap         │ (Grows up ↑)
0xFFFF └──────────────┘
```

## 🔍 Common C Pitfalls in WASM

### Buffer Overflows
```c
char buffer[10];
strcpy(buffer, "This string is too long!");  // 💥 Overflow!
```

### Memory Leaks
```c
char* leak = malloc(100);
// Forgot to call free(leak)
```

### Dangling Pointers
```c
char* ptr = malloc(100);
free(ptr);
*ptr = 'A';  // 💥 Use after free!
```

### Null Pointer Dereference
```c
char* ptr = NULL;
*ptr = 'A';  // 💥 Segfault!
```

## 🎓 Exercises

1. **Implement free()**
   - Track allocated blocks
   - Mark as available for reuse
   - Handle fragmentation

2. **Add Bounds Checking**
   - Validate array access
   - Detect buffer overflows
   - Add canary values

3. **String Functions**
   - Implement strcat, strcmp
   - Add safe versions (strncpy)
   - Create sprintf equivalent

4. **Memory Pool**
   - Fixed-size block allocator
   - Reduce fragmentation
   - Faster than malloc

5. **Debug Features**
   - Memory usage tracking
   - Allocation logging
   - Leak detection

## 🆚 Comparison with Rust

| Feature | C | Rust |
|---------|---|------|
| Memory Safety | ❌ Manual | ✅ Compile-time |
| Null Safety | ❌ NULL crashes | ✅ Option type |
| Buffer Overflows | ❌ Possible | ✅ Prevented |
| Use After Free | ❌ Possible | ✅ Prevented |
| Data Races | ❌ Possible | ✅ Prevented |
| Performance | ✅ Excellent | ✅ Excellent |
| Binary Size | ✅ ~5KB | ✅ ~10KB |
| Learning Curve | Moderate | Steep |

## 🐛 Troubleshooting

### "emcc: command not found"
```bash
source /path/to/emsdk/emsdk_env.sh
```

### "Memory access out of bounds"
- Check array indices
- Validate pointer arithmetic
- Use bounds checking

### "Unreachable executed"
- Usually null pointer access
- Check all malloc returns
- Validate pointers before use

### Large binary size
- Remove printf/stdlib includes
- Use -Os optimization
- Strip debug symbols

## 📖 Resources

- [Emscripten Documentation](https://emscripten.org/docs/)
- [WebAssembly C/C++ Guide](https://webassembly.org/getting-started/developers-guide/)
- [C Memory Management](https://en.cppreference.com/w/c/memory)
- [WASM Memory Model](https://webassembly.github.io/spec/core/exec/runtime.html#memory-instances)

## ✨ Key Takeaways

1. **C gives complete control** over memory but requires discipline
2. **Manual management** can be efficient but error-prone
3. **No safety nets** - buffer overflows and leaks are possible
4. **Minimal runtime** produces tiny binaries
5. **Understanding pointers** is crucial for systems programming

This plugin demonstrates that while C can produce **very small and fast** WASM modules, it requires **careful programming** to avoid memory-related bugs!