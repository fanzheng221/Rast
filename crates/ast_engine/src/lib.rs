//! Rast AST Engine
//! 
//! Core AST analysis engine for JavaScript/TypeScript parsing (MVP version).

use serde::{Deserialize, Serialize};

/// Represents an exported identifier from a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    /// Name of exported item
    pub name: String,
    /// Type of export: "function", "variable", "class", "type", "interface"
    pub kind: String,
    /// Source location (line, column)
    pub location: Option<(usize, usize)>,
}

/// Represents a linting issue found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    /// Category of lint issue
    pub category: String,
    /// Severity level
    pub severity: String,
    /// Description of the issue
    pub message: String,
    /// Source location (line, column)
    pub location: Option<(usize, usize)>,
}

/// Result of AST analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// List of exports found in module
    pub exports: Vec<ExportInfo>,
    /// List of linting issues found
    pub issues: Vec<LintIssue>,
}

/// Analyzes JavaScript/TypeScript source code and extracts AST information
/// 
/// # Arguments
/// * `source` - The source code to analyze
/// 
/// # Returns
/// * `String` - JSON string containing analysis result
pub fn analyze_ast(source: &str) -> String {
    let result = analyze_ast_internal(source);
    serde_json::to_string(&result).unwrap_or_else(|_| String::from("{\"error\":\"Failed to serialize result\"}"))
}

/// Internal function that performs analysis using regex-based parsing
/// This is a simplified MVP implementation. Task 2 will use oxc for proper AST traversal.
fn analyze_ast_internal(source: &str) -> AnalysisResult {
    let mut exports = Vec::new();
    let mut issues = Vec::new();
    let mut line = 0;
    
    for l in source.lines() {
        line += 1;
        
        // Check for exports
        if l.trim().starts_with("export function") {
            if let Some(name) = extract_function_name(l, "export function") {
                exports.push(ExportInfo {
                    name,
                    kind: "function".to_string(),
                    location: Some((line, l.find("export").unwrap_or(0))),
                });
            }
        } else if l.trim().starts_with("export const") {
            if let Some(name) = extract_var_name(l, "export const") {
                exports.push(ExportInfo {
                    name,
                    kind: "variable".to_string(),
                    location: Some((line, l.find("export").unwrap_or(0))),
                });
            }
        } else if l.trim().starts_with("export class") {
            if let Some(name) = extract_class_name(l, "export class") {
                exports.push(ExportInfo {
                    name,
                    kind: "class".to_string(),
                    location: Some((line, l.find("export").unwrap_or(0))),
                });
            }
        } else if l.trim().starts_with("export interface") {
            if let Some(name) = extract_interface_name(l, "export interface") {
                exports.push(ExportInfo {
                    name,
                    kind: "interface".to_string(),
                    location: Some((line, l.find("export").unwrap_or(0))),
                });
            }
        } else if l.trim().starts_with("export type") {
            if let Some(name) = extract_type_name(l, "export type") {
                exports.push(ExportInfo {
                    name,
                    kind: "type".to_string(),
                    location: Some((line, l.find("export").unwrap_or(0))),
                });
            }
        }
        
        // Check for var declarations (lint rule)
        if l.trim().starts_with("var ") {
            issues.push(LintIssue {
                category: "best-practices".to_string(),
                severity: "warning".to_string(),
                message: "Avoid using 'var'. Use 'const' or 'let' instead.".to_string(),
                location: Some((line, l.find("var").unwrap_or(0))),
            });
        }
        
        // Check for console statements (lint rule)
        if l.contains("console.log") || l.contains("console.error") || l.contains("console.warn") {
            issues.push(LintIssue {
                category: "dev-code".to_string(),
                severity: "warning".to_string(),
                message: "Console statement detected. Remove in production code.".to_string(),
                location: Some((line, l.find("console").unwrap_or(0))),
            });
        }
    }
    
    AnalysisResult {
        exports,
        issues,
    }
}

/// Extract function name from a line
fn extract_function_name(line: &str, prefix: &str) -> Option<String> {
    line.replace(prefix, "").trim().split('(').next().and_then(|s| {
        let name = s.trim().trim_end_matches('{').trim();
        if name.is_empty() { None } else { Some(name.to_string()) }
    })
}

/// Extract variable name from a line
fn extract_var_name(line: &str, prefix: &str) -> Option<String> {
    line.replace(prefix, "").trim().split('=').next().and_then(|s| {
        let name = s.trim();
        if name.is_empty() { None } else { Some(name.to_string()) }
    })
}

/// Extract class name from a line
fn extract_class_name(line: &str, prefix: &str) -> Option<String> {
    line.replace(prefix, "").trim().split('{').next().and_then(|s| {
        let name = s.trim();
        if name.is_empty() { None } else { Some(name.to_string()) }
    })
}

/// Extract interface name from a line
fn extract_interface_name(line: &str, prefix: &str) -> Option<String> {
    line.replace(prefix, "").trim().split('{').next().and_then(|s| {
        let name = s.trim();
        if name.is_empty() { None } else { Some(name.to_string()) }
    })
}

/// Extract type name from a line
fn extract_type_name(line: &str, prefix: &str) -> Option<String> {
    line.replace(prefix, "").trim().split('=').next().and_then(|s| {
        let name = s.trim();
        if name.is_empty() { None } else { Some(name.to_string()) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty_code() {
        let result = analyze_ast("");
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.exports.len(), 0);
        assert_eq!(parsed.issues.len(), 0);
    }

    #[test]
    fn test_analyze_function_export() {
        let code = r#"
export function testFunction() {
    return 42;
}
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.exports.len(), 1);
        assert_eq!(parsed.exports[0].name, "testFunction");
        assert_eq!(parsed.exports[0].kind, "function");
    }

    #[test]
    fn test_analyze_multiple_exports() {
        let code = r#"
export const myVar = 123;
export function myFunc() {}
export class MyClass {}
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.exports.len(), 3);
    }

    #[test]
    fn test_detect_var_declaration() {
        let code = r#"
var x = 1;
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(parsed.issues[0].category, "best-practices");
        assert!(parsed.issues[0].message.contains("var"));
    }

    #[test]
    fn test_detect_console_log() {
        let code = r#"
function test() {
    console.log("debug");
}
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(parsed.issues[0].category, "dev-code");
        // Check that the message is about Console (capital C)
        assert!(parsed.issues[0].message.contains("Console"));
    }

    #[test]
    fn test_serialization() {
        let code = "export const x = 1;";
        let result = analyze_ast(code);
        // Should be valid JSON
        serde_json::from_str::<serde_json::Value>(&result).unwrap();
    }
}
