use std::path::Path;

use ast_engine::{IntoAstNode, NodeTrait};
use oxc::{allocator::Allocator, ast::ast::Statement, parser::Parser, span::SourceType};

fn parse_program<'a>(allocator: &'a Allocator, source: &'a str) -> oxc::ast::ast::Program<'a> {
    let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
    Parser::new(allocator, source, source_type).parse().program
}

#[test]
fn node_trait_unified_api_works_for_program_to_call_chain() {
    let allocator = Allocator::default();
    let source = "const value = foo(1, bar);";
    let program = parse_program(&allocator, source);

    let root = program.as_node(source);
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
fn node_trait_upcast_and_downcast_work() {
    let allocator = Allocator::default();
    let source = "export function greet(name) { return name; }";
    let program = parse_program(&allocator, source);

    let root = program.as_node(source);
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
fn into_ast_node_supports_statement_declaration_expression() {
    let allocator = Allocator::default();
    let source = "export const value = foo(bar);";
    let program = parse_program(&allocator, source);

    let statement = &program.body[0];
    let statement_node = statement.as_node(source);
    assert_eq!(statement_node.kind(), "ExportNamedDeclaration");
    assert!(statement_node.as_statement().is_some());

    let declaration = match statement {
        Statement::ExportNamedDeclaration(named_export) => {
            named_export.declaration.as_ref().expect("expected declaration")
        }
        _ => panic!("expected export named declaration statement"),
    };

    let declaration_node = declaration.as_node(source);
    assert_eq!(declaration_node.kind(), "VariableDeclaration");
    assert!(declaration_node.as_declaration().is_some());

    let init = declaration_node
        .children()
        .first()
        .expect("expected variable declaration child")
        .children()
        .first()
        .copied()
        .expect("expected init expression");
    assert_eq!(init.kind(), "CallExpression");
    assert!(init.as_expression().is_some());
}

#[test]
fn children_cover_class_to_method_path() {
    let allocator = Allocator::default();
    let source = "class A { m() { return 1; } }";
    let program = parse_program(&allocator, source);

    let root = program.as_node(source);
    let class_statement = &root.children()[0];
    assert_eq!(class_statement.kind(), "ClassDeclaration");

    let class_nodes = class_statement.children();
    assert_eq!(class_nodes.len(), 1);
    assert_eq!(class_nodes[0].kind(), "Class");

    let method_nodes = class_nodes[0].children();
    assert_eq!(method_nodes.len(), 1);
    assert_eq!(method_nodes[0].kind(), "MethodDefinition");

    let function_nodes = method_nodes[0].children();
    assert_eq!(function_nodes.len(), 1);
    assert_eq!(function_nodes[0].kind(), "Function");
}
