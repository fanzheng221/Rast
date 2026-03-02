//! NAPI bindings for Rast AST engine
//!
//! Exposes Rust AST analysis functions to Node.js via napi-rs.

mod napi_api;
mod pattern;
mod project_graph;
mod rule_apply;
mod serde_payload;

pub use napi_api::analyze_ast;
pub use pattern::{find_pattern, find_pattern_in_vue_sfc};
pub use project_graph::{initialize_graph, ProjectGraph};
pub use rule_apply::{apply_rule, scan_directory};

#[cfg(test)]
mod tests;
