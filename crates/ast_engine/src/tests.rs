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
    let code = "export function testFunction() { return 42; }";
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

#[test]
fn test_rule_core_yaml_pattern_rule() {
    let yaml = r#"
id: no-console-log
language: ts
rule:
  pattern: console.log($$$ARGS)
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    assert_eq!(parsed.id, "no-console-log");
    assert_eq!(parsed.language, RuleLanguage::Ts);

    match parsed.rule.core {
        RuleKind::Pattern(rule) => {
            assert_eq!(rule.pattern, "console.log($$$ARGS)");
        }
        _ => panic!("expected pattern rule"),
    }
}

#[test]
fn test_rule_core_yaml_regex_rule() {
    let yaml = r#"
id: no-debugger
language: javascript
rule:
  regex: "\\bdebugger\\b"
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    assert_eq!(parsed.language, RuleLanguage::Javascript);

    match parsed.rule.core {
        RuleKind::Regex(rule) => {
            assert_eq!(rule.regex, "\\bdebugger\\b");
        }
        _ => panic!("expected regex rule"),
    }
}

#[test]
fn test_rule_core_yaml_kind_rule() {
    let yaml = r#"
id: avoid-with
language: js
rule:
  kind: WithStatement
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    assert_eq!(parsed.language, RuleLanguage::Js);

    match parsed.rule.core {
        RuleKind::Kind(rule) => {
            assert_eq!(rule.kind, "WithStatement");
        }
        _ => panic!("expected kind rule"),
    }
}

#[test]
fn test_rule_core_yaml_all_rule() {
    let yaml = r#"
id: no-console
language: ts
rule:
  all:
    - pattern: console.log($$$ARGS)
    - kind: CallExpression
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    match parsed.rule.core {
        RuleKind::All(rule) => {
            assert_eq!(rule.all.len(), 2);
            assert!(matches!(rule.all[0].core, RuleKind::Pattern(_)));
            assert!(matches!(rule.all[1].core, RuleKind::Kind(_)));
        }
        _ => panic!("expected all composite rule"),
    }
}

#[test]
fn test_rule_core_yaml_any_rule() {
    let yaml = r#"
id: avoid-debug
language: ts
rule:
  any:
    - regex: "\\bdebugger\\b"
    - pattern: console.debug($$$ARGS)
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    match parsed.rule.core {
        RuleKind::Any(rule) => {
            assert_eq!(rule.any.len(), 2);
            assert!(matches!(rule.any[0].core, RuleKind::Regex(_)));
            assert!(matches!(rule.any[1].core, RuleKind::Pattern(_)));
        }
        _ => panic!("expected any composite rule"),
    }
}

#[test]
fn test_rule_core_yaml_not_rule() {
    let yaml = r#"
id: non-call
language: js
rule:
  not:
    kind: CallExpression
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    match parsed.rule.core {
        RuleKind::Not(rule) => {
            assert!(matches!(rule.not.core, RuleKind::Kind(_)));
        }
        _ => panic!("expected not composite rule"),
    }
}

#[test]
fn test_rule_core_yaml_inside_rule() {
    let yaml = r#"
id: inside-call
language: js
rule:
  inside:
    kind: CallExpression
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    match parsed.rule.core {
        RuleKind::Inside(rule) => {
            assert!(matches!(rule.inside.core, RuleKind::Kind(_)));
        }
        _ => panic!("expected inside relational rule"),
    }
}

#[test]
fn test_rule_core_yaml_has_rule() {
    let yaml = r#"
id: has-call
language: js
rule:
  has:
    kind: CallExpression
"#;

    let parsed = RuleCore::from_yaml(yaml).unwrap();
    match parsed.rule.core {
        RuleKind::Has(rule) => {
            assert!(matches!(rule.has.core, RuleKind::Kind(_)));
        }
        _ => panic!("expected has relational rule"),
    }
}

#[test]
fn test_rule_core_yaml_rejects_multiple_atomic_keys() {
    let yaml = r#"
id: invalid-rule
language: tsx
rule:
  pattern: foo($A)
  kind: CallExpression
"#;

    let parsed = RuleCore::from_yaml(yaml);
    assert!(parsed.is_err());
}

#[test]
fn test_rule_core_yaml_rejects_multiple_composite_keys() {
    let yaml = r#"
id: invalid-composite
language: ts
rule:
  all:
    - kind: CallExpression
  any:
    - kind: FunctionDeclaration
"#;

    let parsed = RuleCore::from_yaml(yaml);
    assert!(parsed.is_err());
}

#[test]
fn test_complex_typescript_structure() {
    let code = r#"
import type { User } from "./types";
import React, { useMemo } from "react";

/** User service */
export interface Service<T> {
  get(id: string): Promise<T>;
}

export type Maybe<T> = T | null;

export class UserService implements Service<User> {
  get(id: string): Promise<User> {
    return fetchUser(id);
  }
}

export function buildName(user: User): string {
  console.log(user.id);
  return user.name;
}
"#;

    let result = analyze_ast(code);
    let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
    assert!(parsed.file_structure.imports.len() >= 2);
    assert!(parsed
        .file_structure
        .call_graph
        .edges
        .iter()
        .any(|e| e.callee.contains("console")));
    assert!(parsed
        .exports
        .iter()
        .any(|export| export.name == "buildName" && export.kind == "function"));
}

#[test]
fn test_project_graph_add_and_query_files() {
    let graph = ProjectGraph::new();
    let utils = r#"export function helper(): string { return \"ok\"; }"#;
    let app = r#"
import { helper } from "./utils";
export function run() {
  return helper();
}
"#;

    graph.add_file("src/utils.ts", utils).unwrap();
    graph.add_file("src/app.ts", app).unwrap();

    let files = graph.get_all_files();
    assert_eq!(
        files,
        vec!["src/app.ts".to_string(), "src/utils.ts".to_string()]
    );
    assert!(graph.get_file_structure("src/app.ts").is_some());
    assert!(graph.get_file_structure("src/missing.ts").is_none());
}

#[test]
fn test_project_graph_resolve_dependencies_cross_file() {
    let graph = ProjectGraph::new();
    graph
        .add_file("src/utils.ts", "export const helper = () => 1;")
        .unwrap();
    graph
        .add_file(
            "src/app.ts",
            "import { helper } from './utils'; export const run = () => helper();",
        )
        .unwrap();

    let deps = graph.resolve_dependencies("src/app.ts");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].source, "src/utils.ts");
    assert_eq!(deps[0].kind, "import");
}

#[test]
fn test_project_graph_find_symbol_and_clear() {
    let graph = ProjectGraph::new();
    graph
        .add_file(
            "src/a.ts",
            "export interface User { id: string } export const userName = 'A';",
        )
        .unwrap();
    graph
        .add_file("src/b.ts", "export const userName = 'B';")
        .unwrap();

    let user_name_symbols = graph.find_symbol("userName");
    assert_eq!(user_name_symbols.len(), 2);

    let user_symbols = graph.find_symbol("User");
    assert_eq!(user_symbols.len(), 1);
    assert_eq!(user_symbols[0].kind, "interface");

    graph.clear();
    assert!(graph.get_all_files().is_empty());
    assert!(graph.find_symbol("userName").is_empty());
}

#[test]
fn test_project_graph_repeated_add_uses_cache() {
    let graph = ProjectGraph::new();
    let code = "export const cached = 1;";
    graph.add_file("src/cache.ts", code).unwrap();
    graph.add_file("src/cache.ts", code).unwrap();

    let files = graph.get_all_files();
    assert_eq!(files, vec!["src/cache.ts".to_string()]);
    let found = graph.find_symbol("cached");
    assert_eq!(found.len(), 1);
}

#[test]
fn test_node_trait_unified_api() {
    let allocator = Allocator::default();
    let source = "const value = foo(1, bar);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    let root = parsed.program.as_node(source);
    assert_eq!(root.kind(), "Program");
    assert_eq!(root.text(), source);
    let root_span = root.span();
    assert_eq!(root_span.start, 0);
    assert_eq!(root_span.end as usize, source.len());

    let statements = root.children();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].kind(), "VariableDeclaration");

    let variable_nodes = statements[0].children();
    assert_eq!(variable_nodes.len(), 1);
    let call_nodes = variable_nodes[0].children();
    assert_eq!(call_nodes.len(), 1);
    assert_eq!(call_nodes[0].kind(), "CallExpression");
    assert_eq!(call_nodes[0].text(), "foo(1, bar)");
}

#[test]
fn test_node_trait_upcast_and_downcast() {
    let allocator = Allocator::default();
    let source = "export function greet(name) { return name; }";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    let root = parsed.program.as_node(source);
    let statements = root.children();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].kind(), "ExportNamedDeclaration");
    assert!(statements[0].as_statement().is_some());

    let declaration_nodes = statements[0].children();
    assert_eq!(declaration_nodes.len(), 1);
    assert_eq!(declaration_nodes[0].kind(), "FunctionDeclaration");
    assert!(declaration_nodes[0].as_declaration().is_some());

    let lowered_nodes = declaration_nodes[0].children();
    assert_eq!(lowered_nodes.len(), 1);
    assert_eq!(lowered_nodes[0].kind(), "Function");
    assert!(lowered_nodes[0].text().contains("greet"));
}

#[test]
fn test_identify_meta_variables_detects_single_and_multi_capture() {
    let allocator = Allocator::default();
    let source = "const value = fn($A, $$$B, $$$);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    let root = parsed.program.as_node(source);
    let meta_vars = identify_meta_variables(&root);
    assert_eq!(meta_vars.len(), 3);

    let mut single = false;
    let mut multi = false;
    let mut wildcard = false;

    for kind in meta_vars.values() {
        match kind {
            PatternNodeKind::MetaVar(node) => {
                if node.name == "A" {
                    single = true;
                }
            }
            PatternNodeKind::MultiMetaVar(node) => {
                if node.name == "B" {
                    multi = true;
                }
            }
            PatternNodeKind::MultiWildcard => wildcard = true,
            PatternNodeKind::Node { .. } => {}
        }
    }

    assert!(single);
    assert!(multi);
    assert!(wildcard);
}

#[test]
fn test_to_pattern_ast_marks_meta_variable_nodes() {
    let allocator = Allocator::default();
    let source = "const value = fn($A, $$$B);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    let root = parsed.program.as_node(source);
    let pattern = to_pattern_ast(root);

    fn count_kind(node: &PatternNode, target: &PatternNodeKind) -> usize {
        let mut count = usize::from(&node.kind == target);
        for child in &node.children {
            count += count_kind(child, target);
        }
        count
    }

    assert_eq!(
        count_kind(
            &pattern,
            &PatternNodeKind::MetaVar(WildcardNode {
                name: "A".to_string(),
            })
        ),
        1
    );
    assert_eq!(
        count_kind(
            &pattern,
            &PatternNodeKind::MultiMetaVar(WildcardNode {
                name: "B".to_string(),
            })
        ),
        1
    );
    assert_eq!(pattern.text, source);
}

#[test]
fn test_pattern_matcher_captures_single_and_multi_meta_var() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(answer, foo, bar);";
    let pattern_source = "const value = fn($A, $$$B);";

    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::default();
    let env = matcher
        .match_node(target, &pattern)
        .expect("pattern should match target");

    assert_eq!(env.single_captures.get("A").unwrap().text, "answer");
    assert_eq!(env.multi_captures.get("B").unwrap().len(), 2);
    assert_eq!(env.multi_captures.get("B").unwrap()[0].text, "foo");
    assert_eq!(env.multi_captures.get("B").unwrap()[1].text, "bar");
}

#[test]
fn test_pattern_matcher_exposes_match_result_and_capture_queries() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(answer, foo, bar);";
    let pattern_source = "const value = fn($A, $$$B);";

    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::default();
    let result = matcher.match_result(target, &pattern);

    assert!(result.is_match());
    assert!(result.environment().has_single_capture("A"));
    assert!(result.environment().has_multi_capture("B"));
    assert_eq!(
        result.environment().get_single_capture("A").unwrap().text,
        "answer"
    );
    assert_eq!(
        result.environment().get_multi_capture("B").unwrap().len(),
        2
    );
}

#[test]
fn test_pattern_matcher_requires_consistent_meta_var_binding() {
    let allocator = Allocator::default();
    let pattern_source = "const pair = fn($A, $A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);
    let matcher = PatternMatcher::default();

    let matched_source = "const pair = fn(x, x);";
    let parsed_matched = Parser::new(&allocator, matched_source, source_type).parse();
    let matched_target = parsed_matched.program.as_node(matched_source);
    assert!(matcher.match_node(matched_target, &pattern).is_some());

    let unmatched_source = "const pair = fn(x, y);";
    let parsed_unmatched = Parser::new(&allocator, unmatched_source, source_type).parse();
    let unmatched_target = parsed_unmatched.program.as_node(unmatched_source);
    assert!(matcher.match_node(unmatched_target, &pattern).is_none());
}

#[test]
fn test_pattern_matcher_requires_consistent_multi_meta_var_binding() {
    let allocator = Allocator::default();
    let pattern_source = "const value = fn($$$A, $$$A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);
    let matcher = PatternMatcher::default();

    let matched_source = "const value = fn(x, y, x, y);";
    let parsed_matched = Parser::new(&allocator, matched_source, source_type).parse();
    let matched_target = parsed_matched.program.as_node(matched_source);
    assert!(matcher.match_node(matched_target, &pattern).is_some());

    let unmatched_source = "const value = fn(x, y, x, z);";
    let parsed_unmatched = Parser::new(&allocator, unmatched_source, source_type).parse();
    let unmatched_target = parsed_unmatched.program.as_node(unmatched_source);
    assert!(matcher.match_node(unmatched_target, &pattern).is_none());
}

#[test]
fn test_pattern_matcher_rejects_mixed_single_and_multi_capture_name() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(a, b);";
    let pattern_source = "const value = fn($A, $$$A);";

    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::default();
    assert!(matcher.match_node(target, &pattern).is_none());
}

#[test]
fn test_pattern_matcher_relaxed_skips_whitespace_and_comments() {
    let allocator = Allocator::default();
    let target_source = "const /*comment*/ value = 1;";
    let pattern_source = "const value = 1;";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::new(MatchStrictness::Relaxed);
    assert!(matcher.match_node(target, &pattern).is_some());
}

#[test]
fn test_pattern_matcher_ast_ignores_whitespace_and_comments() {
    let allocator = Allocator::default();
    let target_source = "const /*comment*/ value = 1;";
    let pattern_source = "const value = 1;";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::new(MatchStrictness::Ast);
    assert!(matcher.match_node(target, &pattern).is_some());
}

#[test]
fn test_pattern_matcher_cst_requires_exact_text() {
    let allocator = Allocator::default();
    let target_source = "const /*comment*/ value = 1;";
    let pattern_source = "const value = 1;";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::new(MatchStrictness::Cst);
    assert!(matcher.match_node(target, &pattern).is_none());
}

#[test]
fn test_pattern_matcher_signature_ignores_function_body() {
    let allocator = Allocator::default();
    let target_source = "function foo(a) { return a + 1; }";
    let pattern_source = "function foo(a) { return a + 2; }";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::new(MatchStrictness::Signature);
    assert!(matcher.match_node(target, &pattern).is_some());
}

#[test]
fn test_pattern_matcher_template_supports_multi_wildcard() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(a, b, c);";
    let pattern_source = "const value = fn($$$);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let matcher = PatternMatcher::new(MatchStrictness::Template);
    assert!(matcher.match_node(target, &pattern).is_some());
}

#[test]
fn test_composite_matcher_keeps_capture_environment() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(answer);";
    let pattern_source = "const value = fn($A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let mut composite = CompositeMatcher::new();
    composite.push(PatternMatcher::default());
    composite.push(PatternMatcher::default());

    let env = composite
        .match_node(target, &pattern)
        .expect("composite matcher should match and preserve captures");
    assert_eq!(env.single_captures.get("A").unwrap().text, "answer");
}

#[test]
fn test_all_matcher_requires_all_matchers() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(answer);";
    let pattern_source = "const value = fn($A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let mut matcher = AllMatcher::new();
    matcher.push(PatternMatcher::default());
    matcher.push(CaptureMatcher { name: "MARK" });
    let env = matcher
        .match_node(target, &pattern)
        .expect("all matcher should match when all children match");

    assert!(env.has_single_capture("A"));
    assert!(env.has_single_capture("MARK"));

    let mut failing_matcher = AllMatcher::new();
    failing_matcher.push(PatternMatcher::default());
    failing_matcher.push(RejectMatcher);
    assert!(failing_matcher.match_node(target, &pattern).is_none());
}

#[test]
fn test_any_matcher_accepts_first_success_and_keeps_failed_branch_side_effects_isolated() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(answer);";
    let pattern_source = "const value = fn($A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let mut matcher = AnyMatcher::new();
    matcher.push(RejectMatcher);
    matcher.push(PatternMatcher::default());

    let env = matcher
        .match_node(target, &pattern)
        .expect("any matcher should match when one child matches");
    assert!(env.has_single_capture("A"));

    let mut baseline = MatchEnvironment::default();
    baseline.single_captures.insert(
        "PRE".to_string(),
        CapturedNode {
            kind: "Preset".to_string(),
            text: "preset".to_string(),
            span: NodeSpan { start: 0, end: 0 },
        },
    );
    let mut failing_any = AnyMatcher::new();
    failing_any.push(RejectMatcher);
    failing_any.push(RejectMatcher);

    let before = baseline.clone();
    assert!(!failing_any.match_node_with_env(target, &pattern, &mut baseline));
    assert_eq!(baseline, before);
}

#[test]
fn test_not_matcher_negates_without_mutating_environment() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(answer);";
    let pattern_source = "const value = fn($A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_root = parsed_pattern.program.as_node(pattern_source);
    let pattern = to_pattern_ast(pattern_root);

    let before = MatchEnvironment::default();
    let mut env = before.clone();
    let not_pattern = NotMatcher::new(PatternMatcher::default());
    assert!(!not_pattern.match_node_with_env(target, &pattern, &mut env));
    assert_eq!(env, before);

    let mut env = MatchEnvironment::default();
    let not_reject = NotMatcher::new(RejectMatcher);
    assert!(not_reject.match_node_with_env(target, &pattern, &mut env));
    assert!(!env.has_single_capture("A"));
}

#[test]
fn test_find_all_matches_prefers_outer_overlap() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(fn(a));";
    let pattern_source = "const value = fn($$$A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_program = parsed_pattern.program.as_node(pattern_source);
    let pattern_call = pattern_program.children()[0].children()[0].children()[0];
    let pattern = to_pattern_ast(pattern_call);

    let matcher = PatternMatcher::default();
    let results = find_all_matches(&matcher, target, &pattern, ConflictResolution::PreferOuter);

    assert_eq!(results.len(), 1);
    assert_eq!(
        &target_source[results[0].span.start as usize..results[0].span.end as usize],
        "fn(fn(a))"
    );
    assert_eq!(
        results[0].environment.get_multi_capture("A").unwrap().len(),
        1
    );
}

#[test]
fn test_find_all_matches_prefers_inner_overlap() {
    let allocator = Allocator::default();
    let target_source = "const value = fn(fn(a));";
    let pattern_source = "const value = fn($$$A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_program = parsed_pattern.program.as_node(pattern_source);
    let pattern_call = pattern_program.children()[0].children()[0].children()[0];
    let pattern = to_pattern_ast(pattern_call);

    let matcher = PatternMatcher::default();
    let results = find_all_matches(&matcher, target, &pattern, ConflictResolution::PreferInner);

    assert_eq!(results.len(), 1);
    assert_eq!(
        &target_source[results[0].span.start as usize..results[0].span.end as usize],
        "fn(a)"
    );
}

#[test]
fn test_find_all_matches_sorts_and_dedups_non_overlapping_matches() {
    let allocator = Allocator::default();
    let target_source = "const a = fn(1); const b = fn(2); const c = fn(3);";
    let pattern_source = "const value = fn($A);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

    let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
    let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

    let target = parsed_target.program.as_node(target_source);
    let pattern_program = parsed_pattern.program.as_node(pattern_source);
    let pattern_call = pattern_program.children()[0].children()[0].children()[0];
    let pattern = to_pattern_ast(pattern_call);

    let matcher = PatternMatcher::default();
    let results = FindAllMatches(&matcher, target, &pattern, ConflictResolution::PreferOuter);

    assert_eq!(results.len(), 3);
    assert_eq!(
        &target_source[results[0].span.start as usize..results[0].span.end as usize],
        "fn(1)"
    );
    assert_eq!(
        &target_source[results[1].span.start as usize..results[1].span.end as usize],
        "fn(2)"
    );
    assert_eq!(
        &target_source[results[2].span.start as usize..results[2].span.end as usize],
        "fn(3)"
    );
    assert_eq!(
        results[0].environment.get_single_capture("A").unwrap().text,
        "1"
    );
    assert_eq!(
        results[1].environment.get_single_capture("A").unwrap().text,
        "2"
    );
    assert_eq!(
        results[2].environment.get_single_capture("A").unwrap().text,
        "3"
    );
}

#[test]
fn test_identify_meta_variables_rejects_non_conforming_names() {
    let allocator = Allocator::default();
    let source = "const value = fn($a, $$$b, $1, $_OK);";
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed = Parser::new(&allocator, source, source_type).parse();

    let root = parsed.program.as_node(source);
    let meta_vars = identify_meta_variables(&root);

    assert_eq!(meta_vars.len(), 1);
    assert!(meta_vars.values().any(|kind| {
        matches!(
            kind,
            PatternNodeKind::MetaVar(WildcardNode { name }) if name == "_OK"
        )
    }));
}

struct RejectMatcher;

impl Matcher for RejectMatcher {
    fn match_node_with_env<'a>(
        &self,
        _target: AstNode<'a>,
        _pattern: &PatternNode,
        _env: &mut MatchEnvironment,
    ) -> bool {
        false
    }
}

struct CaptureMatcher {
    name: &'static str,
}

impl Matcher for CaptureMatcher {
    fn match_node_with_env<'a>(
        &self,
        target: AstNode<'a>,
        _pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool {
        env.single_captures
            .insert(self.name.to_string(), CapturedNode::from(target));
        true
    }
}
