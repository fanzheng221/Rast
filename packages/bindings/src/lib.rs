//! NAPI bindings for Rast AST engine
//!
//! Exposes Rust AST analysis functions to Node.js via napi-rs.

use napi_derive::napi;
use ast_engine::analyze_ast as internal_analyze_ast;

/// Analyzes JavaScript/TypeScript source code and returns JSON AST analysis result
///
/// # Arguments
/// * `source` - The source code to analyze
///
/// # Returns
/// * JSON string containing exports and linting issues
#[napi]
pub fn analyze_ast(source: String) -> String {
    internal_analyze_ast(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_ast() {
        let code = r#"export function test() { return 42; }"#;
        let result = analyze_ast(code.to_string());
        // Should return valid JSON
        serde_json::from_str::<serde_json::Value>(&result).unwrap();
        assert!(result.contains("test"));
    }
}
