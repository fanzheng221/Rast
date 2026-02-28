use ast_engine::{apply_span_replacements, NodeSpan, SpanReplacement};

fn span_of(source: &str, needle: &str) -> NodeSpan {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("needle not found: {needle}"));
    NodeSpan {
        start: start as u32,
        end: (start + needle.len()) as u32,
    }
}

#[test]
fn test_delete_absorbs_trailing_newline() {
    let source = "let a = 1;\nlet b = 2;\n";
    let replacements =
        vec![SpanReplacement::new(span_of(source, "let a = 1;"), "").with_trivia_absorption()];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "let b = 2;\n");
}

#[test]
fn test_delete_absorbs_leading_whitespace() {
    let source = "if (x) {    foo();}";
    let replacements =
        vec![SpanReplacement::new(span_of(source, "foo();"), "").with_trivia_absorption()];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "if (x) {}");
}

#[test]
fn test_delete_absorbs_both_sides() {
    let source = "{\n    foo();    \n}\n";
    let replacements =
        vec![SpanReplacement::new(span_of(source, "foo();"), "").with_trivia_absorption()];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "{\n}\n");
}

#[test]
fn test_replacement_does_not_absorb() {
    let source = "let value = 1;\n";
    let replacements =
        vec![SpanReplacement::new(span_of(source, "1"), "10").with_trivia_absorption()];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "let value = 10;\n");
}

#[test]
fn test_multiple_deletes_no_double_absorb() {
    let source = "first(); \nsecond();\n";
    let replacements = vec![
        SpanReplacement::new(span_of(source, "first();"), "").with_trivia_absorption(),
        SpanReplacement::new(span_of(source, "second();"), "").with_trivia_absorption(),
    ];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "");
}

#[test]
fn test_delete_without_absorption_keeps_trivia() {
    let source = "let a = 1;\n";
    let replacements = vec![SpanReplacement::new(span_of(source, "let a = 1;"), "")];

    let result = apply_span_replacements(source, &replacements).unwrap();

    assert_eq!(result, "\n");
}
