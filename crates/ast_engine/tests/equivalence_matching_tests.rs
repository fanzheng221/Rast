use ast_engine::{
    matcher::{
        CompositeMatcher, MatchEnvironment, MatchResult, MatchStrictness, Matcher, PatternMatcher,
    },
    to_pattern_ast, CapturedNode, IntoAstNode, PatternNode,
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
fn test_single_and_multi_capture_are_recorded() {
    with_target_and_pattern(
        "const value = fn(alpha, beta, gamma);",
        "const value = fn($A, $$$ARGS);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let result: MatchResult = matcher.match_result(target, &pattern);

            assert!(result.matched);
            assert_eq!(
                result
                    .environment
                    .get_single_capture("A")
                    .map(|node| node.text.as_str()),
                Some("alpha")
            );

            let multi = result
                .environment
                .get_multi_capture("ARGS")
                .expect("expected $$$ARGS capture");
            assert_eq!(multi.len(), 2);
            assert_eq!(multi[0].text, "beta");
            assert_eq!(multi[1].text, "gamma");
        },
    );
}

#[test]
fn test_repeated_meta_var_requires_consistent_binding() {
    with_target_and_pattern(
        "const value = pair(alpha, beta);",
        "const value = pair($A, $A);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let result = matcher.match_result(target, &pattern);
            assert!(!result.matched);
        },
    );

    with_target_and_pattern(
        "const value = pair(alpha, alpha);",
        "const value = pair($A, $A);",
        |target, pattern| {
            let matcher = PatternMatcher::new(MatchStrictness::Template);
            let result = matcher.match_result(target, &pattern);
            assert!(result.matched);
        },
    );
}

#[test]
fn test_relaxed_and_signature_modes_ignore_trivia_or_body() {
    with_target_and_pattern(
        "function sum(a, b) { /* keep */ return a + b; }",
        "function sum(a,b){return a+b;}",
        |target, pattern| {
            let relaxed = PatternMatcher::new(MatchStrictness::Relaxed);
            let cst = PatternMatcher::new(MatchStrictness::Cst);

            assert!(relaxed.match_result(target, &pattern).matched);
            assert!(!cst.match_result(target, &pattern).matched);
        },
    );

    with_target_and_pattern(
        "function sum(a, b) { return a + b; }",
        "function sum(a,b){throw new Error('x');}",
        |target, pattern| {
            let signature = PatternMatcher::new(MatchStrictness::Signature);
            assert!(signature.match_result(target, &pattern).matched);
        },
    );
}

struct RequireCaptureText {
    name: &'static str,
    expected_text: &'static str,
}

impl Matcher for RequireCaptureText {
    fn match_node_with_env<'a>(
        &self,
        _target: ast_engine::AstNode<'a>,
        _pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool {
        env.get_single_capture(self.name)
            .map(|captured| captured.text == self.expected_text)
            .unwrap_or(false)
    }
}

struct MutatingFailureMatcher;

impl Matcher for MutatingFailureMatcher {
    fn match_node_with_env<'a>(
        &self,
        target: ast_engine::AstNode<'a>,
        _pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool {
        env.single_captures
            .insert("BROKEN".to_string(), CapturedNode::from(target));
        false
    }
}

#[test]
fn test_composite_matcher_keeps_environment_consistent() {
    with_target_and_pattern(
        "const value = fn(alpha);",
        "const value = fn($A);",
        |target, pattern| {
            let mut composite = CompositeMatcher::new();
            composite.push(PatternMatcher::new(MatchStrictness::Template));
            composite.push(RequireCaptureText {
                name: "A",
                expected_text: "alpha",
            });

            let matched = composite.match_result(target, &pattern);
            assert!(matched.matched);
            assert_eq!(
                matched
                    .environment
                    .get_single_capture("A")
                    .map(|node| node.text.as_str()),
                Some("alpha")
            );
        },
    );

    with_target_and_pattern(
        "const value = fn(alpha);",
        "const value = fn($A);",
        |target, pattern| {
            let mut composite = CompositeMatcher::new();
            composite.push(PatternMatcher::new(MatchStrictness::Template));
            composite.push(MutatingFailureMatcher);

            let mut env = MatchEnvironment::default();
            let matched = composite.match_node_with_env(target, &pattern, &mut env);

            assert!(!matched);
            assert!(env.single_captures.is_empty());
        },
    );
}
