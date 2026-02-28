use ast_engine::{
    matcher::{MatchEnvironment, MatchStrictness, Matcher, PatternMatcher},
    to_pattern_ast, IntoAstNode, PatternNode,
};
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

fn with_target_and_pattern(
    target_source: &str,
    pattern_source: &str,
    run: impl for<'a> FnOnce(ast_engine::AstNode<'a>, PatternNode),
) {
    let target_allocator = Allocator::default();
    let pattern_allocator = Allocator::default();
    let source_type = SourceType::default();

    let target_parsed = Parser::new(&target_allocator, target_source, source_type).parse();
    let pattern_parsed = Parser::new(&pattern_allocator, pattern_source, source_type).parse();

    run(
        target_parsed.program.as_node(target_source),
        to_pattern_ast(pattern_parsed.program.as_node(pattern_source)),
    );
}

#[test]
fn test_match_node_with_env_and_capture_records_single_and_multi() {
    with_target_and_pattern(
        "const value = fn(alpha, beta, gamma);",
        "const value = fn($A, $$$ARGS);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();

            assert!(matcher.match_node_with_env_and_capture(target, &pattern, &mut env));
            assert!(env.has_single_capture("A"));
            assert!(env.has_multi_capture("ARGS"));
            assert_eq!(
                env.get_single_capture("A")
                    .map(|captured| captured.text.as_str()),
                Some("alpha")
            );

            let multi = env
                .get_multi_capture("ARGS")
                .expect("expected $$$ARGS capture");
            assert_eq!(multi.len(), 2);
            assert_eq!(multi[0].text, "beta");
            assert_eq!(multi[1].text, "gamma");
        },
    );
}

#[test]
fn test_repeated_single_meta_var_requires_same_content() {
    with_target_and_pattern(
        "const value = pair(alpha, beta);",
        "const value = pair($A, $A);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();
            assert!(!matcher.match_node_with_env_and_capture(target, &pattern, &mut env));
        },
    );

    with_target_and_pattern(
        "const value = pair(alpha, alpha);",
        "const value = pair($A, $A);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();
            assert!(matcher.match_node_with_env_and_capture(target, &pattern, &mut env));
        },
    );
}

#[test]
fn test_repeated_multi_meta_var_requires_same_sequence() {
    with_target_and_pattern(
        "const value = fn(alpha, beta, alpha, gamma);",
        "const value = fn($$$ARGS, $$$ARGS);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();
            assert!(!matcher.match_node_with_env_and_capture(target, &pattern, &mut env));
        },
    );

    with_target_and_pattern(
        "const value = fn(alpha, beta, alpha, beta);",
        "const value = fn($$$ARGS, $$$ARGS);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();
            assert!(matcher.match_node_with_env_and_capture(target, &pattern, &mut env));

            let captures = env
                .get_multi_capture("ARGS")
                .expect("expected $$$ARGS capture");
            assert_eq!(captures.len(), 2);
            assert_eq!(captures[0].text, "alpha");
            assert_eq!(captures[1].text, "beta");
        },
    );
}

#[test]
fn test_mixed_single_and_multi_meta_var_with_same_name_fails() {
    with_target_and_pattern(
        "const value = fn(alpha, beta);",
        "const value = fn($A, $$$A);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();
            assert!(!matcher.match_node_with_env_and_capture(target, &pattern, &mut env));
        },
    );

    with_target_and_pattern(
        "const value = fn(alpha, beta);",
        "const value = fn($$$A, $A);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let mut env = MatchEnvironment::default();
            assert!(!matcher.match_node_with_env_and_capture(target, &pattern, &mut env));
        },
    );
}
