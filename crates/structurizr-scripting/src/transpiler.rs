//! Groovy to Lua transpiler.
//!
//! This module provides automatic conversion of common Groovy patterns
//! used in Structurizr scripts to equivalent Lua code.
//!
//! # Supported Patterns
//!
//! - Property access: `workspace.name` → `workspace:getName()`
//! - Property assignment: `workspace.name = "x"` → `workspace:setName("x")`
//! - Method calls: `workspace.addPerson("x")` → `workspace:addPerson("x")`
//! - Closures with `each`: `list.each { item -> }` → `for _, item in ipairs(list) do end`
//! - String interpolation: `"Hello ${name}"` → `"Hello " .. name`
//! - Comments: `//` and `/* */` → `--` and `--[[ ]]`
//!
//! # Limitations
//!
//! - Complex Groovy features (meta-programming, AST transforms) are not supported
//! - Java interop calls will fail
//! - Some Groovy-specific methods may not have equivalents

use crate::error::Result;
use regex_lite::Regex;

/// Transpiler for converting Groovy scripts to Lua.
#[allow(dead_code)]
pub struct GroovyTranspiler {
    // Compiled regex patterns (some reserved for future enhancements)
    property_get: Regex,
    property_set: Regex,
    method_call: Regex,
    each_closure: Regex,
    string_interp: Regex,
    line_comment: Regex,
    block_comment: Regex,
    def_var: Regex,
    println: Regex,
}

impl GroovyTranspiler {
    /// Create a new transpiler instance.
    pub fn new() -> Self {
        Self {
            // Match property access like workspace.name (simplified - no look-around)
            property_get: Regex::new(r"(\w+)\.(\w+)").unwrap(),
            // Match property assignment like workspace.name = "value"
            property_set: Regex::new(r"(\w+)\.(\w+)\s*=\s*(.+)").unwrap(),
            // Match method calls like workspace.addPerson("name")
            method_call: Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
            // Match each closure like list.each { item -> ... }
            each_closure: Regex::new(r"(\w+)\.each\s*\{\s*(\w+)\s*->\s*").unwrap(),
            // Match string interpolation like "${variable}"
            string_interp: Regex::new(r"\$\{(\w+)\}").unwrap(),
            // Match line comments
            line_comment: Regex::new(r"//(.*)$").unwrap(),
            // Match block comments
            block_comment: Regex::new(r"/\*").unwrap(),
            // Match def variable declaration
            def_var: Regex::new(r"\bdef\s+(\w+)\s*=").unwrap(),
            // Match println
            println: Regex::new(r"\bprintln\s*\(").unwrap(),
        }
    }

    /// Transpile a Groovy script to Lua.
    pub fn transpile(&self, groovy: &str) -> Result<String> {
        let mut lua = String::new();
        let mut in_block_comment = false;
        let mut brace_depth = 0;
        let mut each_vars: Vec<String> = Vec::new();

        for line in groovy.lines() {
            let trimmed = line.trim();

            // Handle block comments
            if in_block_comment {
                if trimmed.contains("*/") {
                    in_block_comment = false;
                    lua.push_str(&line.replace("*/", "]]"));
                } else {
                    lua.push_str(line);
                }
                lua.push('\n');
                continue;
            }

            if trimmed.starts_with("/*") {
                in_block_comment = true;
                lua.push_str(&line.replace("/*", "--[["));
                lua.push('\n');
                continue;
            }

            // Convert the line
            let converted = self.transpile_line(line, &mut brace_depth, &mut each_vars)?;
            lua.push_str(&converted);
            lua.push('\n');
        }

        Ok(lua)
    }

    /// Transpile a single line of Groovy to Lua.
    fn transpile_line(
        &self,
        line: &str,
        brace_depth: &mut i32,
        each_vars: &mut Vec<String>,
    ) -> Result<String> {
        let mut result = line.to_string();

        // Convert line comments // -> --
        result = self.line_comment.replace_all(&result, "--$1").to_string();

        // Convert println -> print
        result = self.println.replace_all(&result, "print(").to_string();

        // Convert def -> local
        result = self.def_var.replace_all(&result, "local $1 =").to_string();

        // Handle .each { var -> ... } closures
        if let Some(caps) = self.each_closure.captures(&result) {
            let list_var = caps.get(1).unwrap().as_str();
            let iter_var = caps.get(2).unwrap().as_str();
            each_vars.push(iter_var.to_string());
            result = self.each_closure.replace(&result, &format!(
                "for _, {} in ipairs({}) do",
                iter_var, list_var
            )).to_string();
            *brace_depth += 1;
        }

        // Handle closing braces for each blocks
        if result.trim() == "}" && *brace_depth > 0 {
            result = "end".to_string();
            *brace_depth -= 1;
            each_vars.pop();
        }

        // Convert string interpolation "${var}" -> " .. var .. "
        result = self.string_interp.replace_all(&result, |caps: &regex_lite::Captures| {
            format!("\" .. {} .. \"", &caps[1])
        }).to_string();

        // Convert method calls: obj.method( -> obj:method(
        // But be careful not to convert things like workspace.model.people
        result = self.convert_method_calls(&result);

        // Convert property access for known patterns
        result = self.convert_property_access(&result);

        Ok(result)
    }

    /// Convert method calls from dot notation to colon notation.
    fn convert_method_calls(&self, line: &str) -> String {
        // Simple pattern: word.word( -> word:word(
        // This is a simplified conversion that handles common cases
        let mut result = line.to_string();

        // Convert method calls on workspace object
        let patterns = [
            (r"workspace\.addPerson\(", "workspace:addPerson("),
            (r"workspace\.addSoftwareSystem\(", "workspace:addSoftwareSystem("),
            (r"workspace\.addContainer\(", "workspace:addContainer("),
            (r"workspace\.addComponent\(", "workspace:addComponent("),
            (r"workspace\.addRelationship\(", "workspace:addRelationship("),
            (r"workspace\.setName\(", "workspace:setName("),
            (r"workspace\.setDescription\(", "workspace:setDescription("),
            (r"workspace\.getName\(", "workspace:getName("),
            (r"workspace\.getDescription\(", "workspace:getDescription("),
            (r"workspace\.findElementByName\(", "workspace:findElementByName("),
            (r"workspace\.getPeople\(", "workspace:getPeople("),
            (r"workspace\.getSoftwareSystems\(", "workspace:getSoftwareSystems("),
        ];

        for (pattern, replacement) in patterns {
            let re = Regex::new(pattern).unwrap();
            result = re.replace_all(&result, replacement).to_string();
        }

        result
    }

    /// Convert property access patterns.
    fn convert_property_access(&self, line: &str) -> String {
        let mut result = line.to_string();

        // Property setters: workspace.name = "x" -> workspace:setName("x")
        // We do setters FIRST so that getters don't match assignment targets
        let setter_patterns = [
            (r#"workspace\.name\s*=\s*"([^"]*)""#, r#"workspace:setName("$1")"#),
            (r#"workspace\.description\s*=\s*"([^"]*)""#, r#"workspace:setDescription("$1")"#),
        ];

        for (pattern, replacement) in setter_patterns {
            let re = Regex::new(pattern).unwrap();
            result = re.replace_all(&result, replacement).to_string();
        }

        // Property getters: workspace.name -> workspace:getName()
        // Since setters were already converted above, remaining workspace.name
        // patterns are getters (no need for look-ahead)
        let getter_patterns = [
            (r"workspace\.name\b", "workspace:getName()"),
            (r"workspace\.description\b", "workspace:getDescription()"),
        ];

        for (pattern, replacement) in getter_patterns {
            let re = Regex::new(pattern).unwrap();
            result = re.replace_all(&result, replacement).to_string();
        }

        result
    }

    /// Check if a Groovy script can be transpiled.
    /// Returns a list of unsupported features if found.
    pub fn check_compatibility(&self, groovy: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for unsupported features
        let unsupported = [
            (r"@\w+", "Annotations (@) are not supported"),
            (r"import\s+", "Import statements are not supported"),
            (r"class\s+\w+", "Class definitions are not supported"),
            (r"new\s+\w+\(", "Object instantiation (new) is not supported"),
            (r"\.metaClass", "MetaClass modifications are not supported"),
            (r"\.with\s*\{", ".with closures are not supported"),
            (r"switch\s*\(", "Switch statements need manual conversion"),
            (r"try\s*\{", "Try-catch blocks need manual conversion"),
        ];

        for (pattern, message) in unsupported {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(groovy) {
                    issues.push(message.to_string());
                }
            }
        }

        issues
    }
}

impl Default for GroovyTranspiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_simple_assignment() {
        let transpiler = GroovyTranspiler::new();
        let groovy = r#"workspace.name = "Modified""#;
        let lua = transpiler.transpile(groovy).unwrap();
        assert!(lua.contains("workspace:setName(\"Modified\")"));
    }

    #[test]
    fn test_transpile_method_call() {
        let transpiler = GroovyTranspiler::new();
        let groovy = r#"workspace.addPerson("Alice", "A user")"#;
        let lua = transpiler.transpile(groovy).unwrap();
        assert!(lua.contains("workspace:addPerson(\"Alice\", \"A user\")"));
    }

    #[test]
    fn test_transpile_def_to_local() {
        let transpiler = GroovyTranspiler::new();
        let groovy = r#"def name = "test""#;
        let lua = transpiler.transpile(groovy).unwrap();
        assert!(lua.contains("local name ="));
    }

    #[test]
    fn test_transpile_comments() {
        let transpiler = GroovyTranspiler::new();
        let groovy = "// This is a comment";
        let lua = transpiler.transpile(groovy).unwrap();
        assert!(lua.contains("-- This is a comment"));
    }

    #[test]
    fn test_transpile_println() {
        let transpiler = GroovyTranspiler::new();
        let groovy = r#"println("Hello")"#;
        let lua = transpiler.transpile(groovy).unwrap();
        assert!(lua.contains("print(\"Hello\")"));
    }

    #[test]
    fn test_compatibility_check() {
        let transpiler = GroovyTranspiler::new();

        // Should find issues
        let groovy = r#"
            @Grab('something')
            import groovy.json.JsonSlurper
            class MyPlugin { }
        "#;
        let issues = transpiler.check_compatibility(groovy);
        assert!(!issues.is_empty());

        // Should be compatible
        let simple = r#"workspace.name = "test""#;
        let issues = transpiler.check_compatibility(simple);
        assert!(issues.is_empty());
    }
}
