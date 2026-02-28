use std::path::Path;

use ast_engine::{
    identify_meta_variables, is_valid_meta_capture_name, to_pattern_ast,
    wildcard_kind_from_identifier, IntoAstNode, PatternNode, PatternNodeKind, WildcardNode,
};
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

fn with_root(source: &str, run: impl for<'a> FnOnce(ast_engine::AstNode<'a>)) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    run(parsed.program.as_node(source));
}

fn count_kind(node: &PatternNode, target: &PatternNodeKind) -> usize {
    let mut count = usize::from(&node.kind == target);
    for child in &node.children {
        count += count_kind(child, target);
    }
    count
}

#[test]
fn test_is_valid_meta_capture_name_accepts_uppercase_or_underscore() {
    assert!(is_valid_meta_capture_name("A"));
    assert!(is_valid_meta_capture_name("_"));
    assert!(is_valid_meta_capture_name("A_1"));

    assert!(!is_valid_meta_capture_name(""));
    assert!(!is_valid_meta_capture_name("a"));
    assert!(!is_valid_meta_capture_name("1A"));
    assert!(!is_valid_meta_capture_name("A-b"));
}

#[test]
fn test_wildcard_kind_from_identifier_maps_supported_forms() {
    assert!(matches!(
        wildcard_kind_from_identifier("$A"),
        Some(PatternNodeKind::MetaVar(WildcardNode { name })) if name == "A"
    ));
    assert!(matches!(
        wildcard_kind_from_identifier("$$$ARGS"),
        Some(PatternNodeKind::MultiMetaVar(WildcardNode { name })) if name == "ARGS"
    ));
    assert!(matches!(
        wildcard_kind_from_identifier("$$$"),
        Some(PatternNodeKind::MultiWildcard)
    ));
    assert!(wildcard_kind_from_identifier("$a").is_none());
}

#[test]
fn test_identify_meta_variables_detects_all_wildcard_kinds() {
    let source = "const value = fn($A, $$$ARGS, $$$);";

    with_root(source, |root| {
        let meta_vars = identify_meta_variables(&root);
        assert_eq!(meta_vars.len(), 3);
        assert!(meta_vars.values().any(|kind| {
            matches!(
                kind,
                PatternNodeKind::MetaVar(WildcardNode { name }) if name == "A"
            )
        }));
        assert!(meta_vars.values().any(|kind| {
            matches!(
                kind,
                PatternNodeKind::MultiMetaVar(WildcardNode { name }) if name == "ARGS"
            )
        }));
        assert!(meta_vars
            .values()
            .any(|kind| matches!(kind, PatternNodeKind::MultiWildcard)));
    });
}

#[test]
fn test_identify_meta_variables_rejects_invalid_capture_names() {
    let source = "const value = fn($a, $$$b, $1, $$$1, $_OK);";

    with_root(source, |root| {
        let meta_vars = identify_meta_variables(&root);
        assert_eq!(meta_vars.len(), 1);
        assert!(meta_vars.values().any(|kind| {
            matches!(
                kind,
                PatternNodeKind::MetaVar(WildcardNode { name }) if name == "_OK"
            )
        }));
    });
}

#[test]
fn test_identify_meta_variables_ignores_non_expression_identifier_contexts() {
    let source = "const $A = 1; const value = obj.$A;";

    with_root(source, |root| {
        let meta_vars = identify_meta_variables(&root);
        assert_eq!(meta_vars.len(), 0);
    });
}

#[test]
fn test_to_pattern_ast_marks_meta_nodes_and_preserves_text() {
    let source = "const value = fn($A, $$$ARGS, $$$);";

    with_root(source, |root| {
        let pattern = to_pattern_ast(root);
        assert_eq!(pattern.text, source);
        assert_eq!(pattern.span.start, 0);
        assert_eq!(pattern.span.end as usize, source.len());

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
                    name: "ARGS".to_string(),
                })
            ),
            1
        );
        assert_eq!(count_kind(&pattern, &PatternNodeKind::MultiWildcard), 1);
    });
}
