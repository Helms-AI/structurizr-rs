// Go Statistics Calculator Plugin
// ================================
//
// Educational WASM plugin demonstrating Go's simplicity with TinyGo compilation.
//
// Key Learning Points:
// - Go's simple syntax and error handling
// - TinyGo vs standard Go differences
// - Garbage collection in WASM
// - Interface-based design
// - Defer statements in WASM context
// - Slice and map usage
//
// Note: This uses TinyGo, which has some limitations compared to standard Go:
// - No reflection
// - Limited goroutine support
// - Smaller standard library subset

package main

import (
	"unsafe"
)

// =============================================================================
// WASM IMPORTS FROM HOST
// =============================================================================

// Import host functions using //go:wasmimport directive (TinyGo specific)
//
//go:wasmimport env get_workspace_name_len
func getWorkspaceNameLen() int32

//go:wasmimport env log
func hostLog(ptr unsafe.Pointer, len int32)

// =============================================================================
// LOGGING HELPERS
// =============================================================================

// logMessage sends a string to the host console
// Demonstrates Go's simple string handling
func logMessage(msg string) {
	if len(msg) > 0 {
		hostLog(unsafe.Pointer(unsafe.StringData(msg)), int32(len(msg)))
	}
}

// logNumber converts and logs an integer
func logNumber(n int) {
	logMessage(itoa(n))
}

// itoa converts integer to string (simple implementation)
// TinyGo has strconv but we implement for educational purposes
func itoa(n int) string {
	if n == 0 {
		return "0"
	}

	negative := false
	if n < 0 {
		negative = true
		n = -n
	}

	// Build digits in reverse
	digits := make([]byte, 0, 20)
	for n > 0 {
		digits = append(digits, byte('0'+n%10))
		n /= 10
	}

	// Reverse the digits
	for i, j := 0, len(digits)-1; i < j; i, j = i+1, j-1 {
		digits[i], digits[j] = digits[j], digits[i]
	}

	if negative {
		return "-" + string(digits)
	}
	return string(digits)
}

// =============================================================================
// STATISTICS TYPES
// =============================================================================

// Statistics holds computed statistics
// Demonstrates Go struct definition
type Statistics struct {
	Count   int
	Sum     int
	Min     int
	Max     int
	Average float64
}

// Calculator interface demonstrates Go's interface system
type Calculator interface {
	Calculate(values []int) Statistics
}

// BasicCalculator implements Calculator
type BasicCalculator struct {
	name string
}

// NewBasicCalculator creates a new calculator
// Shows Go's constructor pattern
func NewBasicCalculator(name string) *BasicCalculator {
	return &BasicCalculator{name: name}
}

// Calculate computes statistics from values
// Demonstrates methods on structs
func (c *BasicCalculator) Calculate(values []int) Statistics {
	if len(values) == 0 {
		return Statistics{}
	}

	stats := Statistics{
		Count: len(values),
		Min:   values[0],
		Max:   values[0],
	}

	for _, v := range values {
		stats.Sum += v
		if v < stats.Min {
			stats.Min = v
		}
		if v > stats.Max {
			stats.Max = v
		}
	}

	stats.Average = float64(stats.Sum) / float64(stats.Count)
	return stats
}

// =============================================================================
// CHARACTER ANALYSIS
// =============================================================================

// CharFrequency holds character frequency data
type CharFrequency struct {
	Char  rune
	Count int
}

// analyzeCharacters counts character types
// Demonstrates Go's rune handling and maps
func analyzeCharacters(length int) map[string]int {
	// Since we only have name length, we'll do simulated analysis
	result := make(map[string]int)

	// Simulate character distribution based on length
	result["vowels"] = length / 3        // Approximate
	result["consonants"] = length / 2    // Approximate
	result["spaces"] = length / 10       // Approximate
	result["special"] = length - result["vowels"] - result["consonants"] - result["spaces"]

	if result["special"] < 0 {
		result["special"] = 0
	}

	return result
}

// =============================================================================
// MATHEMATICAL OPERATIONS
// =============================================================================

// fibonacci calculates nth Fibonacci number
// Demonstrates Go's simple loop syntax
func fibonacci(n int) int {
	if n <= 1 {
		return n
	}

	prev, curr := 0, 1
	for i := 2; i <= n; i++ {
		prev, curr = curr, prev+curr
	}
	return curr
}

// factorial calculates n!
// Demonstrates recursion in Go
func factorial(n int) int {
	if n <= 1 {
		return 1
	}
	return n * factorial(n-1)
}

// isPrime checks if n is prime
// Demonstrates early return pattern
func isPrime(n int) bool {
	if n < 2 {
		return false
	}
	if n == 2 {
		return true
	}
	if n%2 == 0 {
		return false
	}

	for i := 3; i*i <= n; i += 2 {
		if n%i == 0 {
			return false
		}
	}
	return true
}

// =============================================================================
// DEFER DEMONSTRATION
// =============================================================================

// demonstrateDefer shows Go's defer statement
// Note: In WASM, defer works but there's no real cleanup needed
func demonstrateDefer() {
	logMessage("\n🔄 Defer Demonstration:")
	logMessage("────────────────────────")

	// Defer runs when function returns (LIFO order)
	defer logMessage("   3. Third (deferred first)")
	defer logMessage("   2. Second (deferred second)")

	logMessage("   1. First (immediate)")

	// Note: Deferred functions run in reverse order
}

// =============================================================================
// SLICE OPERATIONS
// =============================================================================

// demonstrateSlices shows Go slice operations
func demonstrateSlices(length int) {
	logMessage("\n📦 Slice Operations:")
	logMessage("────────────────────")

	// Create a slice
	numbers := make([]int, 0, length)

	// Append values
	for i := 1; i <= min(length, 10); i++ {
		numbers = append(numbers, i*i)
	}

	logMessage("• Created slice with squares: [1, 4, 9, ...]")
	logMessage("• Length: " + itoa(len(numbers)))
	logMessage("• Capacity: " + itoa(cap(numbers)))

	// Slice of slice
	if len(numbers) >= 3 {
		sub := numbers[1:3]
		logMessage("• Subslice [1:3] length: " + itoa(len(sub)))
	}
}

// min returns minimum of two ints
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// =============================================================================
// ERROR HANDLING PATTERN
// =============================================================================

// Result represents a computation result
// Demonstrates Go's error handling pattern without using error type
type Result struct {
	Value int
	OK    bool
}

// safeDivide demonstrates Go-style error handling
func safeDivide(a, b int) Result {
	if b == 0 {
		return Result{Value: 0, OK: false}
	}
	return Result{Value: a / b, OK: true}
}

// =============================================================================
// MAIN ANALYSIS
// =============================================================================

// analyze performs the main plugin analysis
func analyze() {
	logMessage("┌────────────────────────────────────┐")
	logMessage("│  🐹 Go Statistics Calculator       │")
	logMessage("│  TinyGo → WASM Demonstration       │")
	logMessage("└────────────────────────────────────┘")

	// Get workspace name length
	nameLength := int(getWorkspaceNameLen())

	logMessage("\n📊 Workspace Analysis:")
	logMessage("──────────────────────")
	logMessage("• Name length: " + itoa(nameLength) + " characters")

	// Calculate statistics on derived values
	calc := NewBasicCalculator("workspace")
	values := []int{nameLength, nameLength * 2, nameLength / 2, nameLength + 10}
	stats := calc.Calculate(values)

	logMessage("\n📈 Statistics (derived values):")
	logMessage("────────────────────────────────")
	logMessage("• Count: " + itoa(stats.Count))
	logMessage("• Sum: " + itoa(stats.Sum))
	logMessage("• Min: " + itoa(stats.Min))
	logMessage("• Max: " + itoa(stats.Max))

	// Character analysis
	logMessage("\n🔤 Character Analysis (estimated):")
	logMessage("───────────────────────────────────")
	charStats := analyzeCharacters(nameLength)
	logMessage("• Vowels: ~" + itoa(charStats["vowels"]))
	logMessage("• Consonants: ~" + itoa(charStats["consonants"]))

	// Math operations
	logMessage("\n🔢 Mathematical Operations:")
	logMessage("────────────────────────────")
	n := min(nameLength, 10) // Cap for performance
	logMessage("• Fibonacci(" + itoa(n) + ") = " + itoa(fibonacci(n)))
	logMessage("• Factorial(" + itoa(min(n, 12)) + ") = " + itoa(factorial(min(n, 12))))
	logMessage("• IsPrime(" + itoa(nameLength) + ") = " + boolToString(isPrime(nameLength)))

	// Safe division demo
	logMessage("\n🛡️ Safe Division (Go error pattern):")
	logMessage("──────────────────────────────────────")
	result := safeDivide(nameLength, 3)
	if result.OK {
		logMessage("• " + itoa(nameLength) + " / 3 = " + itoa(result.Value))
	}
	result = safeDivide(nameLength, 0)
	if !result.OK {
		logMessage("• " + itoa(nameLength) + " / 0 = error (division by zero)")
	}

	// Defer demonstration
	demonstrateDefer()

	// Slice demonstration
	demonstrateSlices(nameLength)

	// Educational notes
	logMessage("\n🎓 Go/TinyGo Features:")
	logMessage("───────────────────────")
	logMessage("• Simple, readable syntax")
	logMessage("• Garbage collection included")
	logMessage("• Interface-based polymorphism")
	logMessage("• Defer for cleanup (LIFO)")
	logMessage("• No exceptions - explicit errors")
	logMessage("• ~20KB binary with TinyGo runtime")

	logMessage("\n⚠️ TinyGo Limitations:")
	logMessage("────────────────────────")
	logMessage("• No reflection support")
	logMessage("• Limited goroutine support")
	logMessage("• Smaller stdlib subset")
	logMessage("• Some packages unsupported")

	logMessage("\n✅ Analysis complete!")
}

// boolToString converts bool to string
func boolToString(b bool) string {
	if b {
		return "true"
	}
	return "false"
}

// =============================================================================
// ENTRY POINTS
// =============================================================================

// init_plugin is our custom entry point
// TinyGo runtime owns _start and _initialize
//
//export init_plugin
func initPlugin() {
	logMessage("🚀 Starting Go plugin (init_plugin)")
	analyze()
}

// run is an alternative entry point
//
//export run
func run() {
	logMessage("🚀 Starting Go plugin (run)")
	analyze()
}

// main is required but not used in WASM
func main() {
	initPlugin()
}