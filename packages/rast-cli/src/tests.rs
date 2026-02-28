use crate::{run, scan, OutputFormat};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_run_command_basic() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let file = dir.path().join("test.ts");
    fs::write(&file, "const x = 42;").expect("failed to write test file");

    let rule = r#"
id: test-rule
language: ts
rule:
  pattern: const $A = $B
fix: const $A = /* $B */
"#;

    let result = run(&file, rule, OutputFormat::Json, false).expect("run failed");
    assert!(result.contains("/* 42 */"));
}

#[test]
fn test_scan_command_basic() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let file_a = dir.path().join("a.js");
    let file_b = dir.path().join("b.ts");
    fs::write(&file_a, "console.log('foo');").expect("failed to write a.js");
    fs::write(&file_b, "const x = 1;").expect("failed to write b.ts");

    let rule = r#"
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
"#;

    let result = scan(
        &dir.path().to_path_buf(),
        rule,
        true,
        OutputFormat::Json,
        None,
        false,
    )
    .expect("scan failed");

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("invalid json result");
    assert_eq!(parsed.as_array().expect("expected array").len(), 2);

    let content_a = fs::read_to_string(&file_a).expect("failed to read a.js");
    let content_b = fs::read_to_string(&file_b).expect("failed to read b.ts");
    assert_eq!(content_a, "console.log('foo');");
    assert_eq!(content_b, "const x = 1;");
}

#[test]
fn test_parse_extensions_via_scan_filtering() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let file_a = dir.path().join("a.js");
    let file_b = dir.path().join("b.ts");
    fs::write(&file_a, "console.log('foo');").expect("failed to write a.js");
    fs::write(&file_b, "console.log('bar');").expect("failed to write b.ts");

    let rule = r#"
id: no-console
language: ts
rule:
  pattern: console.log($A)
fix: logger.info($A)
"#;

    let result = scan(
        &dir.path().to_path_buf(),
        rule,
        true,
        OutputFormat::Json,
        Some("js".to_string()),
        false,
    )
    .expect("scan failed");

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("invalid json result");
    let entries = parsed.as_array().expect("expected array");
    assert_eq!(entries.len(), 1);
    assert!(entries[0]["path"].as_str().expect("path string").ends_with("a.js"));
}
