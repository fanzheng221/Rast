use ast_engine::{
    find_all_matches, to_pattern_ast, ConflictResolution, IntoAstNode, MatchStrictness, Matcher,
    PatternMatcher,
};
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

fn with_target_and_pattern(
    target_source: &str,
    pattern_source: &str,
    run: impl for<'a> FnOnce(ast_engine::AstNode<'a>, ast_engine::PatternNode),
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
fn prefer_outer_skips_overlapping_children() {
    with_target_and_pattern(
        "const value = fn(fn(a));",
        "const value = fn($$$A);",
        |target, pattern| {
            let pattern = pattern.children[0].children[0].children[0].clone();
            let matcher = PatternMatcher::new(MatchStrictness::Template);

            let results = find_all_matches(&matcher, target, &pattern, ConflictResolution::PreferOuter);

            assert_eq!(results.len(), 1);
            assert_eq!(
                &"const value = fn(fn(a));"[results[0].span.start as usize..results[0].span.end as usize],
                "fn(fn(a))"
            );
            assert_eq!(results[0].environment.get_multi_capture("A").unwrap().len(), 1);
        },
    );
}

#[test]
fn prefer_inner_selects_smallest_overlapping_match() {
    with_target_and_pattern(
        "const value = fn(fn(a));",
        "const value = fn($$$A);",
        |target, pattern| {
            let pattern = pattern.children[0].children[0].children[0].clone();
            let matcher = PatternMatcher::new(MatchStrictness::Template);

            let results = matcher.find_all_matches(target, &pattern, ConflictResolution::PreferInner);

            assert_eq!(results.len(), 1);
            assert_eq!(
                &"const value = fn(fn(a));"[results[0].span.start as usize..results[0].span.end as usize],
                "fn(a)"
            );
        },
    );
}

#[test]
fn non_overlapping_matches_are_sorted_by_source_position() {
    with_target_and_pattern(
        "const a = fn(1); const b = fn(2); const c = fn(3);",
        "const value = fn($A);",
        |target, pattern| {
            let pattern = pattern.children[0].children[0].children[0].clone();
            let matcher = PatternMatcher::default();

            let results = find_all_matches(&matcher, target, &pattern, ConflictResolution::PreferOuter);

            assert_eq!(results.len(), 3);
            assert!(results[0].span.start < results[1].span.start);
            assert!(results[1].span.start < results[2].span.start);
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
        },
    );
}
