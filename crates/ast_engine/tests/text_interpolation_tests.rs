use ast_engine::text_interpolation::{generate_replacement, TemplateFix};
use ast_engine::MatchEnvironment;

#[test]
fn test_pure_text_replacement() {
    let template = TemplateFix::from("console.log('hello')");
    let env = MatchEnvironment::default();
    let result = generate_replacement(&template, &env);
    assert_eq!(result, "console.log('hello')");
}

#[test]
fn test_single_meta_var_replacement() {
    let template = TemplateFix::from("console.log($MSG)");
    let mut env = MatchEnvironment::default();
    env.single_captures.insert(
        "MSG".to_string(),
        ast_engine::CapturedNode {
            kind: "StringLiteral".to_string(),
            text: "world".to_string(),
            span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
        },
    );

    let result = generate_replacement(&template, &env);
    assert_eq!(result, "console.log(world)");
}

#[test]
fn test_multi_meta_var_replacement() {
    let template = TemplateFix::from("return $$$ARGS;");
    let mut env = MatchEnvironment::default();
    env.multi_captures.insert(
        "ARGS".to_string(),
        vec![
            ast_engine::CapturedNode {
                kind: "Identifier".to_string(),
                text: "a".to_string(),
                span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
            },
            ast_engine::CapturedNode {
                kind: "Identifier".to_string(),
                text: "b".to_string(),
                span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
            },
            ast_engine::CapturedNode {
                kind: "Identifier".to_string(),
                text: "c".to_string(),
                span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
            },
        ],
    );

    let result = generate_replacement(&template, &env);
    assert_eq!(result, "return abc;");
}

#[test]
fn test_mixed_meta_vars() {
    let template = TemplateFix::from("$FUNC($ARGS)");
    let mut env = MatchEnvironment::default();
    env.single_captures.insert(
        "FUNC".to_string(),
        ast_engine::CapturedNode {
            kind: "Identifier".to_string(),
            text: "console.log".to_string(),
            span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
        },
    );
    env.multi_captures.insert(
        "ARGS".to_string(),
        vec![
            ast_engine::CapturedNode {
                kind: "StringLiteral".to_string(),
                text: "'hello'".to_string(),
                span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
            },
            ast_engine::CapturedNode {
                kind: "StringLiteral".to_string(),
                text: "'world'".to_string(),
                span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
            },
        ],
    );

    let result = generate_replacement(&template, &env);
    assert_eq!(result, "console.log('hello''world')");
}

#[test]
fn test_replacement_ignores_unmatched_meta_var() {
    let template = TemplateFix::from("console.log($UNKNOWN)");
    let mut env = MatchEnvironment::default();
    env.single_captures.insert(
        "KNOWN".to_string(),
        ast_engine::CapturedNode {
            kind: "Identifier".to_string(),
            text: "value".to_string(),
            span: ast_engine::node_trait::NodeSpan { start: 0, end: 0 },
        },
    );

    let result = generate_replacement(&template, &env);
    assert_eq!(result, "console.log($UNKNOWN)");
}
