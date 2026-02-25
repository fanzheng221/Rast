//! NAPI bindings for Rast AST engine
//!
//! Exposes Rust AST analysis functions to Node.js via napi-rs.

use ast_engine::analyze_ast as internal_analyze_ast;
use ast_engine::ProjectGraph as InternalProjectGraph;
use napi_derive::napi;
use serde::Serialize;

fn to_json_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|err| format!("{{\"error\":\"Failed to serialize result: {err}\"}}"))
}

#[napi]
pub struct ProjectGraph {
    inner: InternalProjectGraph,
}

#[napi]
impl ProjectGraph {
    #[napi(js_name = "add_file")]
    pub fn add_file(&self, path: String, code: String) -> napi::Result<()> {
        self.inner
            .add_file(&path, &code)
            .map_err(napi::Error::from_reason)
    }

    #[napi(js_name = "get_file_structure")]
    pub fn get_file_structure(&self, path: String) -> Option<String> {
        self.inner
            .get_file_structure(&path)
            .map(|structure| to_json_string(&structure))
    }

    #[napi(js_name = "get_symbol_details")]
    pub fn get_symbol_details(&self, symbol: String) -> Vec<String> {
        self.inner
            .find_symbol(&symbol)
            .iter()
            .map(to_json_string)
            .collect()
    }

    #[napi(js_name = "analyze_dependencies")]
    pub fn analyze_dependencies(&self, paths: Vec<String>) -> String {
        let dependencies = paths
            .iter()
            .map(|path| (path.clone(), self.inner.resolve_dependencies(path)))
            .collect::<Vec<_>>();
        to_json_string(&dependencies)
    }
}

#[napi(js_name = "initialize_graph")]
pub fn initialize_graph(mode: String) -> ProjectGraph {
    let _mode = mode;
    ProjectGraph {
        inner: InternalProjectGraph::new(),
    }
}

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

    fn decode_json(input: &str) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(input).unwrap()
    }

    #[test]
    fn test_analyze_ast() {
        let code = r#"export function test() { return 42; }"#;
        let result = analyze_ast(code.to_string());
        // Should return valid JSON
        serde_json::from_str::<serde_json::Value>(&result).unwrap();
        assert!(result.contains("test"));
    }

    #[test]
    fn test_project_graph_napi_bindings() {
        let graph = initialize_graph("default".to_string());
        let utils = r#"export function helper(): string { return \"ok\"; }"#;
        let app = r#"
import { helper } from './utils';
export function run() {
  return helper();
}
"#;

        graph
            .add_file("src/utils.ts".to_string(), utils.to_string())
            .unwrap();
        graph
            .add_file("src/app.ts".to_string(), app.to_string())
            .unwrap();

        let structure_json = graph
            .get_file_structure("src/app.ts".to_string())
            .expect("app.ts should exist");
        let structure = decode_json(&structure_json);
        assert_eq!(structure["language"], "tsx");
        assert!(structure["imports"].is_array());

        let symbol_details = graph.get_symbol_details("run".to_string());
        assert!(!symbol_details.is_empty());
        let first_symbol = decode_json(&symbol_details[0]);
        assert_eq!(first_symbol["name"], "run");

        let dependencies_json = graph.analyze_dependencies(vec!["src/app.ts".to_string()]);
        let dependencies = decode_json(&dependencies_json);
        assert!(dependencies.is_array());
        assert_eq!(dependencies[0][0], "src/app.ts");
        assert_eq!(dependencies[0][1][0]["source"], "src/utils.ts");
    }
}
