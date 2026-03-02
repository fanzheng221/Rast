use ast_engine::analyze_ast as internal_analyze_ast;
use napi_derive::napi;

/// Analyzes JavaScript/TypeScript source code and returns JSON AST analysis result
///
/// # Arguments
/// * `source` - The source code to analyze
///
/// # Returns
/// * JSON string containing exports and linting issues
#[napi(js_name = "analyze_ast")]
pub fn analyze_ast(source: String) -> String {
    internal_analyze_ast(&source)
}
