use ast_engine::ProjectGraph as InternalProjectGraph;
use napi_derive::napi;

use crate::serde_payload::to_json_string;

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
