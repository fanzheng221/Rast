use ast_engine::node_trait::IntoAstNode;
use ast_engine::wildcard_parsing::to_pattern_ast;
use ast_engine::{
    AllMatcher, AnyMatcher, AstNode, CapturedNode, MatchEnvironment, Matcher, NotMatcher,
    PatternMatcher, PatternNode,
};
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::path::Path;

/// Test helper matcher that always rejects
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

/// Test helper matcher that captures the target node with a given name
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

    // AllMatcher succeeds when all children match
    let mut matcher = AllMatcher::new();
    matcher.push(PatternMatcher::default());
    matcher.push(CaptureMatcher { name: "MARK" });
    let env = matcher
        .match_node(target, &pattern)
        .expect("all matcher should match when all children match");

    assert!(env.has_single_capture("A"));
    assert!(env.has_single_capture("MARK"));

    // AllMatcher fails when any child fails
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

    // AnyMatcher accepts first success and keeps environment
    let mut matcher = AnyMatcher::new();
    matcher.push(RejectMatcher);
    matcher.push(PatternMatcher::default());

    let env = matcher
        .match_node(target, &pattern)
        .expect("any matcher should match when one child matches");
    assert!(env.has_single_capture("A"));

    // AnyMatcher isolates failed branch side effects
    let mut baseline = MatchEnvironment::default();
    baseline.single_captures.insert(
        "PRE".to_string(),
        CapturedNode {
            kind: "Preset".to_string(),
            text: "preset".to_string(),
            span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
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

    // CompositeMatcher (AllMatcher) should match and preserve captures
    let mut composite = AllMatcher::new();
    composite.push(PatternMatcher::default());
    composite.push(PatternMatcher::default());

    let env = composite
        .match_node(target, &pattern)
        .expect("composite matcher should match and preserve captures");
    assert_eq!(env.get_single_capture("A").unwrap().text, "answer");
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

    // NotMatcher negates without mutating environment
    let before = MatchEnvironment::default();
    let mut env = before.clone();
    let not_pattern = NotMatcher::new(PatternMatcher::default());
    assert!(!not_pattern.match_node_with_env(target, &pattern, &mut env));
    assert_eq!(env, before);

    // NotMatcher of RejectMatcher should match
    let mut env = MatchEnvironment::default();
    let not_reject = NotMatcher::new(RejectMatcher);
    assert!(not_reject.match_node_with_env(target, &pattern, &mut env));
    assert!(!env.has_single_capture("A"));
}
