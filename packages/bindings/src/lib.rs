//! NAPI bindings for Rast AST engine
//!
//! Exposes Rust AST analysis functions to Node.js via napi-rs.

use ast_engine::analyze_ast as internal_analyze_ast;
use ast_engine::ProjectGraph as InternalProjectGraph;
use ast_engine::{
    apply_span_replacements, generate_replacement, to_pattern_ast, AstNode, ConflictResolution,
    IntoAstNode, MatchEnvironment, MatchResult, Matcher, NodeSpan, NodeTrait, PatternMatcher,
    Rule, RuleCore, RuleKind, RuleLanguage, SpanReplacement, TemplateFix, VueSfcScriptKind,
};
use napi_derive::napi;
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
use regex::Regex;
use serde::Serialize;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn to_json_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|err| format!("{{\"error\":\"Failed to serialize result: {err}\"}}"))
}

fn to_napi_error(context: &str, err: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(format!("{context}: {err}"))
}

fn default_source_type() -> SourceType {
    SourceType::from_path(Path::new("inline.tsx")).unwrap_or_else(|_| {
        SourceType::unambiguous()
            .with_typescript(true)
            .with_jsx(true)
    })
}

fn source_type_for_language(language: RuleLanguage) -> SourceType {
    let file_name = match language {
        RuleLanguage::Js | RuleLanguage::Javascript => "inline.js",
        RuleLanguage::Jsx => "inline.jsx",
        RuleLanguage::Ts | RuleLanguage::Typescript => "inline.ts",
        RuleLanguage::Tsx => "inline.tsx",
    };
    SourceType::from_path(Path::new(file_name)).unwrap_or_else(|_| default_source_type())
}

fn slice_span_text(source: &str, span: NodeSpan) -> napi::Result<String> {
    let start = span.start as usize;
    let end = span.end as usize;

    if end > source.len() || start > end {
        return Err(napi::Error::from_reason("Invalid match span"));
    }
    if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        return Err(napi::Error::from_reason("Match span is not UTF-8 boundary"));
    }

    Ok(source[start..end].to_string())
}

fn env_to_metavariables(env: &MatchEnvironment) -> Map<String, Value> {
    let mut vars = Map::new();

    for (name, captured) in &env.single_captures {
        vars.insert(name.clone(), Value::String(captured.text.clone()));
    }

    for (name, captured_nodes) in &env.multi_captures {
        if vars.contains_key(name) {
            continue;
        }
        let combined = captured_nodes.iter().map(|node| node.text.as_str()).collect::<String>();
        vars.insert(name.clone(), Value::String(combined));
    }

    vars
}

#[derive(Serialize)]
struct SerializableSpan {
    start: u32,
    end: u32,
}

#[derive(Serialize)]
struct PatternMatchPayload {
    span: SerializableSpan,
    text: String,
    metavariables: Map<String, Value>,
}

#[derive(Serialize)]
struct SerializableLocation {
    line: usize,
    column: usize,
}

#[derive(Serialize)]
struct VueSfcScriptPayload {
    span: SerializableSpan,
    kind: String,
}

#[derive(Serialize)]
struct VueSfcPatternMatchPayload {
    relative_span: SerializableSpan,
    absolute_span: SerializableSpan,
    text: String,
    metavariables: Map<String, Value>,
    location: SerializableLocation,
}

#[derive(Serialize)]
struct VueSfcPatternSearchPayload {
    script: Option<VueSfcScriptPayload>,
    matches: Vec<VueSfcPatternMatchPayload>,
}

#[derive(Debug, Clone)]
enum CompiledRule {
    Pattern(ast_engine::PatternNode),
    Regex(Regex),
    Kind(String),
    All(Vec<CompiledRule>),
    Any(Vec<CompiledRule>),
    Not(Box<CompiledRule>),
    Inside(Box<CompiledRule>),
    Has(Box<CompiledRule>),
}

#[derive(Debug, Clone)]
struct ParsedRule {
    core: RuleCore,
    compiled: CompiledRule,
    fix_template: Option<String>,
}

fn parse_rule_and_fix(yaml_rule: &str) -> napi::Result<(RuleCore, Option<String>)> {
    let core = RuleCore::from_yaml(yaml_rule)
        .map_err(|err| to_napi_error("Invalid YAML rule", err))?;

    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(yaml_rule).map_err(|err| to_napi_error("Invalid YAML rule", err))?;
    let fix_template = yaml_value
        .get("fix")
        .and_then(serde_yaml::Value::as_str)
        .map(str::to_string);

    Ok((core, fix_template))
}

fn parse_pattern_ast(pattern: &str, source_type: SourceType) -> napi::Result<ast_engine::PatternNode> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, pattern, source_type).parse();
    if let Some(err) = parsed.errors.first() {
        return Err(to_napi_error("Invalid pattern", err));
    }
    Ok(to_pattern_ast(parsed.program.as_node(pattern)))
}

fn compile_rule(rule: &Rule, source_type: SourceType) -> napi::Result<CompiledRule> {
    match &rule.core {
        RuleKind::Pattern(pattern_rule) => {
            let pattern = parse_pattern_ast(&pattern_rule.pattern, source_type)?;
            Ok(CompiledRule::Pattern(pattern))
        }
        RuleKind::Regex(regex_rule) => {
            let regex = Regex::new(&regex_rule.regex)
                .map_err(|err| to_napi_error("Invalid regex rule", err))?;
            Ok(CompiledRule::Regex(regex))
        }
        RuleKind::Kind(kind_rule) => Ok(CompiledRule::Kind(kind_rule.kind.clone())),
        RuleKind::All(all_rule) => {
            let mut compiled = Vec::with_capacity(all_rule.all.len());
            for item in &all_rule.all {
                compiled.push(compile_rule(item, source_type)?);
            }
            Ok(CompiledRule::All(compiled))
        }
        RuleKind::Any(any_rule) => {
            let mut compiled = Vec::with_capacity(any_rule.any.len());
            for item in &any_rule.any {
                compiled.push(compile_rule(item, source_type)?);
            }
            Ok(CompiledRule::Any(compiled))
        }
        RuleKind::Not(not_rule) => Ok(CompiledRule::Not(Box::new(compile_rule(
            &not_rule.not,
            source_type,
        )?))),
        RuleKind::Inside(inside_rule) => Ok(CompiledRule::Inside(Box::new(compile_rule(
            &inside_rule.inside,
            source_type,
        )?))),
        RuleKind::Has(has_rule) => Ok(CompiledRule::Has(Box::new(compile_rule(
            &has_rule.has,
            source_type,
        )?))),
    }
}

fn merge_environment(base: &mut MatchEnvironment, incoming: &MatchEnvironment) -> bool {
    for (name, captured) in &incoming.single_captures {
        if let Some(existing) = base.single_captures.get(name) {
            if existing.text != captured.text || existing.kind != captured.kind || existing.span != captured.span {
                return false;
            }
        } else {
            base.single_captures.insert(name.clone(), captured.clone());
        }
    }

    for (name, captured_nodes) in &incoming.multi_captures {
        if let Some(existing_nodes) = base.multi_captures.get(name) {
            if existing_nodes.len() != captured_nodes.len() {
                return false;
            }
            for (left, right) in existing_nodes.iter().zip(captured_nodes.iter()) {
                if left.text != right.text || left.kind != right.kind || left.span != right.span {
                    return false;
                }
            }
        } else {
            base.multi_captures
                .insert(name.clone(), captured_nodes.clone());
        }
    }

    true
}

fn evaluate_compiled_rule<'a>(
    node: AstNode<'a>,
    ancestors: &[AstNode<'a>],
    rule: &CompiledRule,
    matcher: &PatternMatcher,
) -> Option<MatchEnvironment> {
    match rule {
        CompiledRule::Pattern(pattern) => matcher.match_node(node, pattern),
        CompiledRule::Regex(regex) => {
            if regex.is_match(&node.text()) {
                Some(MatchEnvironment::default())
            } else {
                None
            }
        }
        CompiledRule::Kind(expected_kind) => {
            if node.kind() == expected_kind {
                Some(MatchEnvironment::default())
            } else {
                None
            }
        }
        CompiledRule::All(rules) => {
            let mut env = MatchEnvironment::default();
            for rule in rules {
                let child_env = evaluate_compiled_rule(node, ancestors, rule, matcher)?;
                if !merge_environment(&mut env, &child_env) {
                    return None;
                }
            }
            Some(env)
        }
        CompiledRule::Any(rules) => {
            for rule in rules {
                if let Some(env) = evaluate_compiled_rule(node, ancestors, rule, matcher) {
                    return Some(env);
                }
            }
            None
        }
        CompiledRule::Not(rule) => {
            if evaluate_compiled_rule(node, ancestors, rule, matcher).is_none() {
                Some(MatchEnvironment::default())
            } else {
                None
            }
        }
        CompiledRule::Inside(rule) => {
            for ancestor in ancestors.iter().rev() {
                if let Some(env) = evaluate_compiled_rule(*ancestor, ancestors, rule, matcher) {
                    return Some(env);
                }
            }
            None
        }
        CompiledRule::Has(rule) => {
            let mut stack = node.children();
            while let Some(child) = stack.pop() {
                if let Some(env) = evaluate_compiled_rule(child, ancestors, rule, matcher) {
                    return Some(env);
                }
                stack.extend(child.children());
            }
            None
        }
    }
}

fn collect_rule_matches<'a>(root: AstNode<'a>, rule: &CompiledRule) -> Vec<MatchResult> {
    let matcher = PatternMatcher::default();
    let mut matches = Vec::new();

    fn walk<'a>(
        node: AstNode<'a>,
        ancestors: &mut Vec<AstNode<'a>>,
        rule: &CompiledRule,
        matcher: &PatternMatcher,
        out: &mut Vec<MatchResult>,
    ) {
        if let Some(environment) = evaluate_compiled_rule(node, ancestors, rule, matcher) {
            out.push(MatchResult {
                span: node.span(),
                environment,
            });
        }

        ancestors.push(node);
        for child in node.children() {
            walk(child, ancestors, rule, matcher, out);
        }
        ancestors.pop();
    }

    walk(root, &mut Vec::new(), rule, &matcher, &mut matches);
    matches
}

fn parse_rule(yaml_rule: &str) -> napi::Result<ParsedRule> {
    let (core, fix_template) = parse_rule_and_fix(yaml_rule)?;
    let source_type = source_type_for_language(core.language);
    let compiled = compile_rule(&core.rule, source_type)?;
    Ok(ParsedRule {
        core,
        compiled,
        fix_template,
    })
}

fn apply_rule_to_source(source: &str, parsed_rule: &ParsedRule) -> napi::Result<(String, usize, usize)> {
    let allocator = Allocator::default();
    let source_type = source_type_for_language(parsed_rule.core.language);
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if let Some(err) = parsed.errors.first() {
        return Err(to_napi_error("Rule evaluation failed", err));
    }

    let root = parsed.program.as_node(source);
    let matches = collect_rule_matches(root, &parsed_rule.compiled);
    let match_count = matches.len();

    let Some(fix_template) = parsed_rule.fix_template.as_deref() else {
        return Ok((source.to_string(), match_count, 0));
    };

    let template = TemplateFix::from(fix_template);
    let mut replacements = Vec::with_capacity(matches.len());
    for matched in &matches {
        let replacement = generate_replacement(&template, &matched.environment);
        replacements.push(SpanReplacement::new(matched.span, replacement));
    }

    if replacements.is_empty() {
        return Ok((source.to_string(), 0, 0));
    }

    let updated_source = apply_span_replacements(source, &replacements)
        .map_err(|err| to_napi_error("Failed to apply replacements", err))?;

    Ok((updated_source, match_count, replacements.len()))
}

#[derive(Serialize)]
struct ScanResult {
    path: String,
    matches: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    modifications: Option<usize>,
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

#[napi]
pub fn find_pattern(source: String, pattern: String) -> napi::Result<String> {
    let allocator = Allocator::default();
    let source_type = default_source_type();

    let parsed_source = Parser::new(&allocator, &source, source_type).parse();
    if let Some(err) = parsed_source.errors.first() {
        return Err(to_napi_error("Invalid source", err));
    }

    let pattern_ast = parse_pattern_ast(&pattern, source_type)?;
    let matcher = PatternMatcher::default();
    let matches = matcher.find_all_matches(
        parsed_source.program.as_node(&source),
        &pattern_ast,
        ConflictResolution::PreferOuter,
    );

    let mut payload = Vec::with_capacity(matches.len());
    for matched in matches {
        payload.push(PatternMatchPayload {
            span: SerializableSpan {
                start: matched.span.start,
                end: matched.span.end,
            },
            text: slice_span_text(&source, matched.span)?,
            metavariables: env_to_metavariables(&matched.environment),
        });
    }

    Ok(to_json_string(&payload))
}

#[napi]
pub fn find_pattern_in_vue_sfc(source: String, pattern: String) -> napi::Result<String> {
    let extractor = ast_engine::VueSfcExtractor::new(&source);
    let Some(block) = extractor.extract_script_block() else {
        return Ok(to_json_string(&VueSfcPatternSearchPayload {
            script: None,
            matches: Vec::new(),
        }));
    };

    let allocator = Allocator::default();
    let source_type = default_source_type();
    let parsed_source = Parser::new(&allocator, block.content, source_type).parse();
    if let Some(err) = parsed_source.errors.first() {
        return Err(to_napi_error("Invalid Vue SFC script block", err));
    }

    let pattern_ast = parse_pattern_ast(&pattern, source_type)?;
    let matcher = PatternMatcher::default();
    let matches = matcher.find_all_matches(
        parsed_source.program.as_node(block.content),
        &pattern_ast,
        ConflictResolution::PreferOuter,
    );

    let script_span = block.offset_map.script_span();
    let script_payload = VueSfcScriptPayload {
        span: SerializableSpan {
            start: script_span.start,
            end: script_span.end,
        },
        kind: match block.kind {
            VueSfcScriptKind::Script => "script".to_string(),
            VueSfcScriptKind::ScriptSetup => "scriptSetup".to_string(),
        },
    };

    let mut payload = Vec::with_capacity(matches.len());
    for matched in matches {
        let absolute_span = block
            .offset_map
            .relative_to_absolute_span(matched.span)
            .ok_or_else(|| {
                napi::Error::from_reason("Failed to map relative span to absolute Vue SFC offset")
            })?;
        let (line, column) = block.offset_map.absolute_offset_to_line_col(absolute_span.start);
        payload.push(VueSfcPatternMatchPayload {
            relative_span: SerializableSpan {
                start: matched.span.start,
                end: matched.span.end,
            },
            absolute_span: SerializableSpan {
                start: absolute_span.start,
                end: absolute_span.end,
            },
            text: slice_span_text(&source, absolute_span)?,
            metavariables: env_to_metavariables(&matched.environment),
            location: SerializableLocation { line, column },
        });
    }

    Ok(to_json_string(&VueSfcPatternSearchPayload {
        script: Some(script_payload),
        matches: payload,
    }))
}

#[napi]
pub fn apply_rule(source: String, yaml_rule: String) -> napi::Result<String> {
    let parsed_rule = parse_rule(&yaml_rule)?;

    if parsed_rule.fix_template.is_none() {
        return Err(napi::Error::from_reason(
            "Invalid YAML rule: missing top-level fix template",
        ));
    }

    let (updated_source, _matches, _modifications) = apply_rule_to_source(&source, &parsed_rule)?;
    Ok(updated_source)
}

#[napi]
pub fn scan_directory(root_path: String, yaml_rule: String, dry_run: bool) -> napi::Result<String> {
    let parsed_rule = parse_rule(&yaml_rule)?;
    if !dry_run && parsed_rule.fix_template.is_none() {
        return Err(napi::Error::from_reason(
            "Invalid YAML rule: missing top-level fix template",
        ));
    }

    let mut results = Vec::new();
    for entry in WalkDir::new(&root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let extension = entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        if !matches!(extension, "js" | "ts" | "jsx" | "tsx") {
            continue;
        }

        let path = entry.path();
        let original_source = fs::read_to_string(path)
            .map_err(|err| to_napi_error("Failed to read file", err))?;
        let (updated_source, match_count, modifications) =
            apply_rule_to_source(&original_source, &parsed_rule)?;

        if !dry_run && modifications > 0 && updated_source != original_source {
            fs::write(path, updated_source).map_err(|err| to_napi_error("Failed to write file", err))?;
        }

        results.push(ScanResult {
            path: path.to_string_lossy().to_string(),
            matches: match_count,
            modifications: if dry_run { None } else { Some(modifications) },
        });
    }

    Ok(to_json_string(&results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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
        let result = find_pattern_in_vue_sfc(
            source.to_string(),
            "console.log($A)".to_string(),
        )
        .unwrap();
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
}
