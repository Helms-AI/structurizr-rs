# Go Statistics Calculator Plugin

An educational WASM plugin demonstrating Go's simplicity and TinyGo's WASM compilation.

## 🎯 Learning Objectives

This plugin teaches:
- Go's simple, readable syntax
- TinyGo compilation to WASM
- Interface-based design patterns
- Error handling without exceptions
- Defer statements and cleanup
- Slice and map operations
- Garbage collection in WASM

## 🏗️ Architecture

```
┌─────────────────────┐
│   Go Source Code    │
│   (main.go)         │
└─────────┬───────────┘
          │ TinyGo
          ↓
┌─────────────────────┐
│   WASM Binary       │
│   + GC Runtime      │
└─────────┬───────────┘
          │
          ↓
┌─────────────────────┐
│   Host Runtime      │
└─────────────────────┘
```

## 📚 Key Concepts

### 1. **Simple Syntax**

Go prioritizes readability:

```go
// Variable declaration
name := "workspace"
count := 42

// Function definition
func add(a, b int) int {
    return a + b
}

// Struct definition
type Statistics struct {
    Count   int
    Average float64
}
```

### 2. **Interface-Based Design**

Go uses interfaces for polymorphism:

```go
// Define interface
type Calculator interface {
    Calculate(values []int) Statistics
}

// Any type implementing Calculate() satisfies the interface
type BasicCalculator struct{}

func (c *BasicCalculator) Calculate(values []int) Statistics {
    // implementation
}
```

### 3. **Error Handling**

Go uses explicit error values instead of exceptions:

```go
// Traditional Go error pattern
func divide(a, b int) (int, error) {
    if b == 0 {
        return 0, errors.New("division by zero")
    }
    return a / b, nil
}

// Usage
result, err := divide(10, 2)
if err != nil {
    // handle error
}
```

### 4. **Defer Statement**

Defer schedules cleanup at function return:

```go
func process() {
    defer cleanup()  // Runs when function returns

    // ... do work ...

    // cleanup() runs here, even if we return early
}
```

Deferred calls execute in LIFO order:
```go
defer fmt.Println("3")
defer fmt.Println("2")
defer fmt.Println("1")
// Output: 1, 2, 3
```

### 5. **Slices and Maps**

Go's built-in collection types:

```go
// Slice (dynamic array)
numbers := make([]int, 0, 10)
numbers = append(numbers, 1, 2, 3)

// Map (hash table)
counts := make(map[string]int)
counts["hello"] = 5
```

### 6. **TinyGo WASM Imports**

```go
//go:wasmimport env get_workspace_name_len
func getWorkspaceNameLen() int32

//go:wasmimport env log
func hostLog(ptr unsafe.Pointer, len int32)
```

## 🛠️ Building

### Prerequisites

Install TinyGo:

```bash
# macOS
brew install tinygo

# Linux
wget https://github.com/tinygo-org/tinygo/releases/download/v0.30.0/tinygo_0.30.0_amd64.deb
sudo dpkg -i tinygo_0.30.0_amd64.deb

# Or via Go
go install github.com/tinygo-org/tinygo@latest
```

### Build Commands

```bash
# Build with TinyGo
tinygo build -o plugin.wasm -target wasm -no-debug -opt 2 main.go

# Check binary size
ls -lh plugin.wasm

# View WASM text format
wasm2wat plugin.wasm | head -50
```

### Build Optimization

TinyGo options for size optimization:
- `-no-debug`: Remove debug info
- `-opt 2`: Maximum optimization
- `-gc=leaking`: Simplest GC (if no cleanup needed)
- `-panic=trap`: Smaller panic handler

## 🆚 TinyGo vs Standard Go

| Feature | Standard Go | TinyGo |
|---------|-------------|---------|
| Binary Size | Large (~2MB+) | Small (~20KB) |
| Reflection | ✅ Full | ❌ None |
| Goroutines | ✅ Full | ⚠️ Limited |
| Channels | ✅ Full | ⚠️ Basic |
| Standard Library | ✅ Complete | ⚠️ Subset |
| CGO | ✅ Yes | ❌ No |
| WASM Support | ⚠️ Basic | ✅ Excellent |

## 🔍 TinyGo Limitations

### No Reflection
```go
// This won't work in TinyGo:
reflect.TypeOf(value)
json.Marshal(struct{}{})  // Uses reflection
```

### Limited Goroutines
```go
// Works but limited:
go func() {
    // Simple concurrent task
}()

// May not work well:
// Complex channel patterns
// Large numbers of goroutines
```

### Unsupported Packages
Some stdlib packages don't work:
- `net/http` (use alternatives)
- `encoding/json` (reflection-based)
- `database/sql`
- Most `reflect`-dependent packages

## 🎓 Exercises

1. **Add More Statistics**
   - Implement median calculation
   - Add standard deviation
   - Calculate percentiles

2. **Interface Exercise**
   - Create `AdvancedCalculator`
   - Implement different algorithms
   - Use interface polymorphism

3. **Defer Patterns**
   - Implement resource cleanup
   - Create nested defers
   - Measure defer overhead

4. **Error Handling**
   - Create custom error types
   - Implement error wrapping
   - Add error context

5. **Map Operations**
   - Implement word frequency counter
   - Create simple cache
   - Handle missing keys

## 🐛 Troubleshooting

### "tinygo: command not found"
```bash
# Add to PATH or install properly
export PATH=$PATH:$(go env GOPATH)/bin
```

### "package not supported"
Check TinyGo compatibility:
```bash
tinygo info
```

### Large binary size
```bash
# Use optimization flags
tinygo build -o plugin.wasm -target wasm -no-debug -opt 2 -gc=leaking main.go
```

### "undefined: reflect.Type"
Reflection isn't supported - use code generation or avoid reflection-based packages.

## 📖 Resources

- [TinyGo Documentation](https://tinygo.org/docs/)
- [TinyGo WASM Guide](https://tinygo.org/docs/guides/webassembly/)
- [Go by Example](https://gobyexample.com/)
- [Effective Go](https://golang.org/doc/effective_go)
- [TinyGo Package Support](https://tinygo.org/docs/reference/lang-support/stdlib/)

## ✨ Key Takeaways

1. **Go's simplicity** makes code readable and maintainable
2. **TinyGo** produces small WASM binaries (~20KB)
3. **Interfaces** enable clean abstraction without inheritance
4. **Explicit errors** are clearer than exceptions
5. **Defer** simplifies cleanup code
6. **Limitations exist** - know what TinyGo can't do

This plugin demonstrates that Go can be an **excellent choice for WASM plugins** when you need:
- Readable, maintainable code
- Garbage collection
- Simple concurrency (limited)
- Fast development iteration

Just be aware of TinyGo's limitations compared to standard Go!