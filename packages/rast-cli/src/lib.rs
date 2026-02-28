use std::fs;
use std::path::{Path, PathBuf};

use ast_engine::{
    apply_span_replacements, generate_replacement, to_pattern_ast, AstNode, IntoAstNode,
    MatchEnvironment, MatchResult, Matcher, NodeTrait, PatternMatcher, Rule, RuleCore, RuleKind,
    RuleLanguage, SpanReplacement, TemplateFix,
};
use clap::ValueEnum;
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
use regex::Regex;
use serde::Serialize;

pub fn run(file: &PathBuf, rule: &str, _output: OutputFormat, _verbose: bool) -> Result<String, String> {
    let source = fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;
    let resolved_rule = resolve_rule(rule)?;
    let parsed_rule = parse_rule(&resolved_rule)?;

    let (updated_source, _, _) = apply_rule_to_source(&source, &parsed_rule)?;
    Ok(updated_source)
}

pub fn scan(
    dir: &PathBuf,
    rule: &str,
    dry_run: bool,
    _output: OutputFormat,
    extensions: Option<String>,
    _verbose: bool,
) -> Result<String, String> {
    let resolved_rule = resolve_rule(rule)?;
    let parsed_rule = parse_rule(&resolved_rule)?;
    let exts = parse_extensions(extensions);
    let mut files = Vec::new();
    collect_files(dir, &exts, &mut files)?;

    let mut results = Vec::new();

    for path in files {
        let original_source = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
        let (updated_source, match_count, modifications) =
            apply_rule_to_source(&original_source, &parsed_rule)?;

        if !dry_run && modifications > 0 && updated_source != original_source {
            fs::write(&path, updated_source)
                .map_err(|e| format!("Failed to write file {}: {}", path.display(), e))?;
        }

        results.push(ScanResult {
            path: path.to_string_lossy().to_string(),
            matches: match_count,
            modifications: if dry_run { None } else { Some(modifications) },
        });
    }

    serde_json::to_string(&results).map_err(|e| format!("Failed to serialize scan result: {}", e))
}

fn resolve_rule(rule: &str) -> Result<String, String> {
    let path = Path::new(rule);
    if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("Failed to read rule file: {}", e))
    } else {
        Ok(rule.to_string())
    }
}

fn default_source_type() -> SourceType {
    SourceType::from_path(Path::new("inline.tsx"))
        .unwrap_or_else(|_| SourceType::unambiguous().with_typescript(true).with_jsx(true))
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

fn parse_rule_and_fix(yaml_rule: &str) -> Result<(RuleCore, Option<String>), String> {
    let core = RuleCore::from_yaml(yaml_rule).map_err(|e| format!("Invalid YAML rule: {}", e))?;
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(yaml_rule).map_err(|e| format!("Invalid YAML rule: {}", e))?;

    let fix_template = yaml_value
        .get("fix")
        .and_then(serde_yaml::Value::as_str)
        .map(str::to_string);

    Ok((core, fix_template))
}

fn parse_pattern_ast(pattern: &str, source_type: SourceType) -> Result<ast_engine::PatternNode, String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, pattern, source_type).parse();
    if let Some(err) = parsed.errors.first() {
        return Err(format!("Invalid pattern: {}", err));
    }

    Ok(to_pattern_ast(parsed.program.as_node(pattern)))
}

fn compile_rule(rule: &Rule, source_type: SourceType) -> Result<CompiledRule, String> {
    match &rule.core {
        RuleKind::Pattern(pattern_rule) => {
            let pattern = parse_pattern_ast(&pattern_rule.pattern, source_type)?;
            Ok(CompiledRule::Pattern(pattern))
        }
        RuleKind::Regex(regex_rule) => {
            let regex = Regex::new(&regex_rule.regex)
                .map_err(|e| format!("Invalid regex rule: {}", e))?;
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
        RuleKind::Not(not_rule) => {
            Ok(CompiledRule::Not(Box::new(compile_rule(&not_rule.not, source_type)?)))
        }
        RuleKind::Inside(inside_rule) => Ok(CompiledRule::Inside(Box::new(compile_rule(
            &inside_rule.inside,
            source_type,
        )?))),
        RuleKind::Has(has_rule) => {
            Ok(CompiledRule::Has(Box::new(compile_rule(&has_rule.has, source_type)?)))
        }
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

fn parse_rule(yaml_rule: &str) -> Result<ParsedRule, String> {
    let (core, fix_template) = parse_rule_and_fix(yaml_rule)?;
    let source_type = source_type_for_language(core.language);
    let compiled = compile_rule(&core.rule, source_type)?;
    Ok(ParsedRule {
        core,
        compiled,
        fix_template,
    })
}

fn apply_rule_to_source(source: &str, parsed_rule: &ParsedRule) -> Result<(String, usize, usize), String> {
    let allocator = Allocator::default();
    let source_type = source_type_for_language(parsed_rule.core.language);
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if let Some(err) = parsed.errors.first() {
        return Err(format!("Rule evaluation failed: {}", err));
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
        .map_err(|e| format!("Failed to apply replacements: {}", e))?;

    Ok((updated_source, match_count, replacements.len()))
}

#[derive(Serialize)]
struct ScanResult {
    path: String,
    matches: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    modifications: Option<usize>,
}

fn collect_files(dir: &Path, exts: &[String], out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, exts, out)?;
            continue;
        }

        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();
        if exts.iter().any(|allowed| allowed == ext) {
            out.push(path);
        }
    }

    Ok(())
}

fn parse_extensions(extensions: Option<String>) -> Vec<String> {
    extensions
        .map(|s| {
            s.split(',')
                .map(|e| e.trim().trim_start_matches('.').to_string())
                .filter(|e| !e.is_empty())
                .collect()
        })
        .unwrap_or_else(|| {
            vec![
                "js".to_string(),
                "ts".to_string(),
                "jsx".to_string(),
                "tsx".to_string(),
            ]
        })
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum OutputFormat {
    Json,
    Text,
}

#[cfg(test)]
mod tests;
