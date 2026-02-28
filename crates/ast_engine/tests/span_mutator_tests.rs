use ast_engine::{
    apply_span_replacements, apply_text_diffs, generate_text_diffs, NodeSpan, SpanMutatorError,
    SpanReplacement, TextDiff,
};

#[test]
fn test_single_span_replacement() {
    let source = "const x = 1;";
    let replacements = vec![SpanReplacement::new(NodeSpan { start: 10, end: 11 }, "2")];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "const x = 2;");
}

#[test]
fn test_multiple_independent_spans() {
    let source = "const a = 1; const b = 2;";
    let replacements = vec![
        SpanReplacement::new(NodeSpan { start: 10, end: 11 }, "10"),
        SpanReplacement::new(NodeSpan { start: 23, end: 24 }, "20"),
    ];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "const a = 10; const b = 20;");
}

#[test]
fn test_reverse_order_application() {
    let source = "abcdef";
    let replacements = vec![
        SpanReplacement::new(NodeSpan { start: 1, end: 3 }, "X"),
        SpanReplacement::new(NodeSpan { start: 4, end: 6 }, "Y"),
    ];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "aXdY");
}

#[test]
fn test_generate_text_diffs_keeps_original_and_replacement() {
    let source = "hello world";
    let replacements = vec![SpanReplacement::new(NodeSpan { start: 6, end: 11 }, "rust")];

    let diffs = generate_text_diffs(source, &replacements).unwrap();

    assert_eq!(
        diffs,
        vec![TextDiff {
            span: NodeSpan { start: 6, end: 11 },
            original: "world".to_string(),
            replacement: "rust".to_string(),
        }]
    );
}

#[test]
fn test_overlapping_spans_should_fail() {
    let source = "abcdef";
    let replacements = vec![
        SpanReplacement::new(NodeSpan { start: 1, end: 4 }, "X"),
        SpanReplacement::new(NodeSpan { start: 3, end: 5 }, "Y"),
    ];

    let err = apply_span_replacements(source, &replacements).unwrap_err();

    assert!(matches!(err, SpanMutatorError::OverlappingSpans { .. }));
}

#[test]
fn test_empty_replacement_delete_text() {
    let source = "const debug = true;";
    let replacements = vec![SpanReplacement::new(NodeSpan { start: 6, end: 12 }, "")];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "const = true;");
}

#[test]
fn test_span_boundaries_and_empty_text() {
    let source = "";
    let replacements = vec![SpanReplacement::new(NodeSpan { start: 0, end: 0 }, "init")];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "init");
}

#[test]
fn test_span_out_of_bounds_should_fail() {
    let source = "abc";
    let replacements = vec![SpanReplacement::new(NodeSpan { start: 0, end: 4 }, "x")];

    let err = apply_span_replacements(source, &replacements).unwrap_err();

    assert!(matches!(err, SpanMutatorError::OutOfBounds { .. }));
}

#[test]
fn test_apply_text_diffs_source_mismatch_should_fail() {
    let source = "hello";
    let diffs = vec![TextDiff {
        span: NodeSpan { start: 0, end: 5 },
        original: "world".to_string(),
        replacement: "hi".to_string(),
    }];

    let err = apply_text_diffs(source, &diffs).unwrap_err();

    assert!(matches!(err, SpanMutatorError::SourceMismatch { .. }));
}
