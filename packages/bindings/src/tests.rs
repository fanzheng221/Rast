use super::*;
use serde_json::Value;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn decode_json(input: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(input).unwrap()
}

fn unique_temp_dir(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rast-bindings-{name}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn test_analyze_ast() {
    let code = r#"export function test() { return 42; }"#;
    let result = analyze_ast(code.to_string());
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

#[test]
fn test_find_pattern_basic() {
    let code = "const x = 42;";
    let pattern = "const x = 42;";

    let result = find_pattern(code.to_string(), pattern.to_string()).unwrap();
    let matches: Vec<Value> = serde_json::from_str(&result).unwrap();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["text"], "const x = 42;");
    assert!(matches[0]["metavariables"].as_object().unwrap().is_empty());
}

#[test]
fn test_find_pattern_with_metavariables() {
    let code = "const value = fn(answer, foo, bar);";
    let pattern = "const value = fn($A, $$$B);";

    let result = find_pattern(code.to_string(), pattern.to_string()).unwrap();
    let matches: Vec<Value> = serde_json::from_str(&result).unwrap();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["metavariables"]["A"], "answer");
    assert_eq!(matches[0]["metavariables"]["B"], "foobar");
}

#[test]
fn test_find_pattern_in_vue_sfc_offsets() {
    let source = r#"<template>
  <button @click="inc">{{ count }}</button>
</template>
<script setup lang="ts">
const count = 1;
console.log(count);
</script>
"#;

    let result = find_pattern_in_vue_sfc(
        source.to_string(),
        "const count = 1;\nconsole.log(count);".to_string(),
    )
    .unwrap();
    let payload: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(payload["script"]["kind"], "scriptSetup");

    let script_start = payload["script"]["span"]["start"].as_u64().unwrap() as usize;
    let script_end = payload["script"]["span"]["end"].as_u64().unwrap() as usize;
    assert!(script_start < script_end);
    assert!(source[script_start..script_end].contains("const count = 1;"));

    let matches = payload["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    let matched = &matches[0];

    let relative_start = matched["relative_span"]["start"].as_u64().unwrap() as usize;
    let absolute_start = matched["absolute_span"]["start"].as_u64().unwrap() as usize;
    let absolute_end = matched["absolute_span"]["end"].as_u64().unwrap() as usize;
    assert_eq!(absolute_start, script_start + relative_start);
    let matched_text = &source[absolute_start..absolute_end];
    assert!(matched_text.contains("const count = 1;"));
    assert!(matched_text.contains("console.log(count);"));
}

#[test]
fn test_find_pattern_in_vue_sfc_without_script() {
    let source = r#"<template><div>no script</div></template>"#;
    let result =
        find_pattern_in_vue_sfc(source.to_string(), "console.log($A)".to_string()).unwrap();
    let payload: Value = serde_json::from_str(&result).unwrap();
    assert!(payload["script"].is_null());
    assert_eq!(payload["matches"].as_array().unwrap().len(), 0);
}

#[test]
fn test_apply_rule_simple() {
    let code = "console.log(foo);";
    let yaml_rule = r#"
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
"#;

    let result = apply_rule(code.to_string(), yaml_rule.to_string()).unwrap();
    assert_eq!(result, "logger.info(foo)");
}

#[test]
fn test_apply_rule_with_fix() {
    let code = "const value = fn(answer, foo, bar);";
    let yaml_rule = r#"
id: wrap-call
language: ts
rule:
  pattern: const value = fn($A, $$$B);
fix: const value = wrapped($A, $$$B);
"#;

    let result = apply_rule(code.to_string(), yaml_rule.to_string()).unwrap();
    assert_eq!(result, "const value = wrapped(answer, foobar);");
}

#[test]
fn test_scan_directory_dry_run() {
    let dir = unique_temp_dir("scan");
    let file_a = dir.join("a.ts");
    let file_b = dir.join("b.js");
    fs::write(&file_a, "console.log(foo);").unwrap();
    fs::write(&file_b, "const x = 1;").unwrap();

    let yaml_rule = r#"
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
"#;

    let result = scan_directory(
        dir.to_string_lossy().to_string(),
        yaml_rule.to_string(),
        true,
    )
    .unwrap();
    let rows: Vec<Value> = serde_json::from_str(&result).unwrap();

    assert_eq!(rows.len(), 2);
    let matched = rows
        .iter()
        .find(|row| row["path"].as_str().unwrap().ends_with("a.ts"))
        .unwrap();
    assert_eq!(matched["matches"], 1);
    assert!(matched.get("modifications").is_none());

    let unchanged = fs::read_to_string(&file_a).unwrap();
    assert_eq!(unchanged, "console.log(foo);");

    fs::remove_dir_all(dir).unwrap();
}
