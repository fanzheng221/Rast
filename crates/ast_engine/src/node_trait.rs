use oxc::ast::ast::{
    CallExpression, Class, ExportDefaultDeclarationKind, Expression, ImportDeclaration,
    MethodDefinition, Statement, TSInterfaceDeclaration, TSTypeAliasDeclaration,
    VariableDeclaration,
};
use oxc::span::GetSpan;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSpan {
    pub start: u32,
    pub end: u32,
}

impl From<oxc::span::Span> for NodeSpan {
    fn from(value: oxc::span::Span) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

/// Unified AST node API for strongly-typed oxc nodes.
pub trait NodeTrait<'a> {
    fn kind(&self) -> &'static str;
    fn text(&self) -> String;
    fn span(&self) -> NodeSpan;
    fn children(&self) -> Vec<AstNode<'a>>;
}

#[derive(Debug, Clone, Copy)]
pub enum AstNodeKind<'a> {
    Program(&'a oxc::ast::ast::Program<'a>),
    Statement(&'a Statement<'a>),
    Declaration(&'a oxc::ast::ast::Declaration<'a>),
    Expression(&'a Expression<'a>),
    ImportDeclaration(&'a ImportDeclaration<'a>),
    VariableDeclaration(&'a VariableDeclaration<'a>),
    Function(&'a oxc::ast::ast::Function<'a>),
    Class(&'a Class<'a>),
    MethodDefinition(&'a MethodDefinition<'a>),
    TSInterfaceDeclaration(&'a TSInterfaceDeclaration<'a>),
    TSTypeAliasDeclaration(&'a TSTypeAliasDeclaration<'a>),
    CallExpression(&'a CallExpression<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct AstNode<'a> {
    source: &'a str,
    kind: AstNodeKind<'a>,
}

impl<'a> AstNode<'a> {
    pub fn from_program(program: &'a oxc::ast::ast::Program<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::Program(program),
        }
    }

    pub fn from_statement(statement: &'a Statement<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::Statement(statement),
        }
    }

    pub fn from_declaration(
        declaration: &'a oxc::ast::ast::Declaration<'a>,
        source: &'a str,
    ) -> Self {
        Self {
            source,
            kind: AstNodeKind::Declaration(declaration),
        }
    }

    pub fn from_expression(expression: &'a Expression<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::Expression(expression),
        }
    }

    pub fn from_import_declaration(
        import_declaration: &'a ImportDeclaration<'a>,
        source: &'a str,
    ) -> Self {
        Self {
            source,
            kind: AstNodeKind::ImportDeclaration(import_declaration),
        }
    }

    pub fn from_variable_declaration(
        variable_declaration: &'a VariableDeclaration<'a>,
        source: &'a str,
    ) -> Self {
        Self {
            source,
            kind: AstNodeKind::VariableDeclaration(variable_declaration),
        }
    }

    pub fn from_function(function: &'a oxc::ast::ast::Function<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::Function(function),
        }
    }

    pub fn from_class(class: &'a Class<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::Class(class),
        }
    }

    pub fn from_method_definition(
        method_definition: &'a MethodDefinition<'a>,
        source: &'a str,
    ) -> Self {
        Self {
            source,
            kind: AstNodeKind::MethodDefinition(method_definition),
        }
    }

    pub fn from_ts_interface(
        interface_declaration: &'a TSInterfaceDeclaration<'a>,
        source: &'a str,
    ) -> Self {
        Self {
            source,
            kind: AstNodeKind::TSInterfaceDeclaration(interface_declaration),
        }
    }

    pub fn from_ts_type_alias(type_alias: &'a TSTypeAliasDeclaration<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::TSTypeAliasDeclaration(type_alias),
        }
    }

    pub fn from_call_expression(call_expression: &'a CallExpression<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::CallExpression(call_expression),
        }
    }

    pub fn as_program(&self) -> Option<&'a oxc::ast::ast::Program<'a>> {
        match self.kind {
            AstNodeKind::Program(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_statement(&self) -> Option<&'a Statement<'a>> {
        match self.kind {
            AstNodeKind::Statement(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_declaration(&self) -> Option<&'a oxc::ast::ast::Declaration<'a>> {
        match self.kind {
            AstNodeKind::Declaration(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_expression(&self) -> Option<&'a Expression<'a>> {
        match self.kind {
            AstNodeKind::Expression(node) => Some(node),
            _ => None,
        }
    }

    fn raw_span(&self) -> oxc::span::Span {
        match self.kind {
            AstNodeKind::Program(node) => node.span,
            AstNodeKind::Statement(node) => node.span(),
            AstNodeKind::Declaration(node) => node.span(),
            AstNodeKind::Expression(node) => node.span(),
            AstNodeKind::ImportDeclaration(node) => node.span,
            AstNodeKind::VariableDeclaration(node) => node.span,
            AstNodeKind::Function(node) => node.span,
            AstNodeKind::Class(node) => node.span,
            AstNodeKind::MethodDefinition(node) => node.span,
            AstNodeKind::TSInterfaceDeclaration(node) => node.span,
            AstNodeKind::TSTypeAliasDeclaration(node) => node.span,
            AstNodeKind::CallExpression(node) => node.span,
        }
    }

    fn statement_kind(statement: &Statement<'_>) -> &'static str {
        match statement {
            Statement::ImportDeclaration(_) => "ImportDeclaration",
            Statement::VariableDeclaration(_) => "VariableDeclaration",
            Statement::FunctionDeclaration(_) => "FunctionDeclaration",
            Statement::ClassDeclaration(_) => "ClassDeclaration",
            Statement::ExpressionStatement(_) => "ExpressionStatement",
            Statement::ExportNamedDeclaration(_) => "ExportNamedDeclaration",
            Statement::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
            Statement::ExportAllDeclaration(_) => "ExportAllDeclaration",
            Statement::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
            Statement::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
            _ => "Statement",
        }
    }

    fn declaration_kind(declaration: &oxc::ast::ast::Declaration<'_>) -> &'static str {
        match declaration {
            oxc::ast::ast::Declaration::FunctionDeclaration(_) => "FunctionDeclaration",
            oxc::ast::ast::Declaration::ClassDeclaration(_) => "ClassDeclaration",
            oxc::ast::ast::Declaration::VariableDeclaration(_) => "VariableDeclaration",
            oxc::ast::ast::Declaration::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
            oxc::ast::ast::Declaration::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
            _ => "Declaration",
        }
    }

    fn expression_kind(expression: &Expression<'_>) -> &'static str {
        match expression {
            Expression::Identifier(_) => "Identifier",
            Expression::CallExpression(_) => "CallExpression",
            _ if expression.as_member_expression().is_some() => "MemberExpression",
            _ => "Expression",
        }
    }

    fn statement_children(statement: &'a Statement<'a>, source: &'a str) -> Vec<AstNode<'a>> {
        match statement {
            Statement::ImportDeclaration(node) => {
                vec![AstNode::from_import_declaration(node, source)]
            }
            Statement::VariableDeclaration(node) => {
                vec![AstNode::from_variable_declaration(node, source)]
            }
            Statement::FunctionDeclaration(node) => vec![AstNode::from_function(node, source)],
            Statement::ClassDeclaration(node) => vec![AstNode::from_class(node, source)],
            Statement::ExpressionStatement(node) => {
                vec![AstNode::from_expression(&node.expression, source)]
            }
            Statement::ExportNamedDeclaration(node) => {
                let mut children = Vec::new();
                if let Some(declaration) = &node.declaration {
                    children.push(AstNode::from_declaration(declaration, source));
                }
                children
            }
            Statement::ExportDefaultDeclaration(node) => match &node.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    vec![AstNode::from_function(function, source)]
                }
                ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    vec![AstNode::from_class(class, source)]
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface_decl) => {
                    vec![AstNode::from_ts_interface(interface_decl, source)]
                }
                _ => Vec::new(),
            },
            Statement::TSInterfaceDeclaration(interface_decl) => {
                vec![AstNode::from_ts_interface(interface_decl, source)]
            }
            Statement::TSTypeAliasDeclaration(type_alias) => {
                vec![AstNode::from_ts_type_alias(type_alias, source)]
            }
            _ => Vec::new(),
        }
    }

    fn declaration_children(
        declaration: &'a oxc::ast::ast::Declaration<'a>,
        source: &'a str,
    ) -> Vec<AstNode<'a>> {
        match declaration {
            oxc::ast::ast::Declaration::VariableDeclaration(node) => {
                vec![AstNode::from_variable_declaration(node, source)]
            }
            oxc::ast::ast::Declaration::FunctionDeclaration(node) => {
                vec![AstNode::from_function(node, source)]
            }
            oxc::ast::ast::Declaration::ClassDeclaration(node) => {
                vec![AstNode::from_class(node, source)]
            }
            oxc::ast::ast::Declaration::TSInterfaceDeclaration(node) => {
                vec![AstNode::from_ts_interface(node, source)]
            }
            oxc::ast::ast::Declaration::TSTypeAliasDeclaration(node) => {
                vec![AstNode::from_ts_type_alias(node, source)]
            }
            _ => Vec::new(),
        }
    }

    fn expression_children(expression: &'a Expression<'a>, source: &'a str) -> Vec<AstNode<'a>> {
        match expression {
            Expression::CallExpression(node) => vec![AstNode::from_call_expression(node, source)],
            _ => Vec::new(),
        }
    }
}

impl<'a> NodeTrait<'a> for AstNode<'a> {
    fn kind(&self) -> &'static str {
        match self.kind {
            AstNodeKind::Program(_) => "Program",
            AstNodeKind::Statement(node) => AstNode::statement_kind(node),
            AstNodeKind::Declaration(node) => AstNode::declaration_kind(node),
            AstNodeKind::Expression(node) => AstNode::expression_kind(node),
            AstNodeKind::ImportDeclaration(_) => "ImportDeclaration",
            AstNodeKind::VariableDeclaration(_) => "VariableDeclaration",
            AstNodeKind::Function(_) => "Function",
            AstNodeKind::Class(_) => "Class",
            AstNodeKind::MethodDefinition(_) => "MethodDefinition",
            AstNodeKind::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
            AstNodeKind::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
            AstNodeKind::CallExpression(_) => "CallExpression",
        }
    }

    fn text(&self) -> String {
        self.raw_span().source_text(self.source).to_string()
    }

    fn span(&self) -> NodeSpan {
        self.raw_span().into()
    }

    fn children(&self) -> Vec<AstNode<'a>> {
        match self.kind {
            AstNodeKind::Program(node) => node
                .body
                .iter()
                .map(|statement| AstNode::from_statement(statement, self.source))
                .collect(),
            AstNodeKind::Statement(node) => AstNode::statement_children(node, self.source),
            AstNodeKind::Declaration(node) => AstNode::declaration_children(node, self.source),
            AstNodeKind::Expression(node) => AstNode::expression_children(node, self.source),
            AstNodeKind::ImportDeclaration(_) => Vec::new(),
            AstNodeKind::VariableDeclaration(node) => node
                .declarations
                .iter()
                .filter_map(|declarator| declarator.init.as_ref())
                .map(|expression| AstNode::from_expression(expression, self.source))
                .collect(),
            AstNodeKind::Function(_) => Vec::new(),
            AstNodeKind::Class(node) => node
                .body
                .body
                .iter()
                .filter_map(|element| match element {
                    oxc::ast::ast::ClassElement::MethodDefinition(method) => {
                        Some(AstNode::from_method_definition(method, self.source))
                    }
                    _ => None,
                })
                .collect(),
            AstNodeKind::MethodDefinition(node) => {
                vec![AstNode::from_function(&node.value, self.source)]
            }
            AstNodeKind::TSInterfaceDeclaration(_) => Vec::new(),
            AstNodeKind::TSTypeAliasDeclaration(_) => Vec::new(),
            AstNodeKind::CallExpression(node) => {
                let mut children = Vec::new();
                children.push(AstNode::from_expression(&node.callee, self.source));
                children.extend(
                    node.arguments
                        .iter()
                        .filter_map(|argument| argument.as_expression())
                        .map(|expression| AstNode::from_expression(expression, self.source)),
                );
                children
            }
        }
    }
}

pub trait IntoAstNode<'a> {
    fn as_node(&'a self, source: &'a str) -> AstNode<'a>;
}

impl<'a> IntoAstNode<'a> for oxc::ast::ast::Program<'a> {
    fn as_node(&'a self, source: &'a str) -> AstNode<'a> {
        AstNode::from_program(self, source)
    }
}

impl<'a> IntoAstNode<'a> for Statement<'a> {
    fn as_node(&'a self, source: &'a str) -> AstNode<'a> {
        AstNode::from_statement(self, source)
    }
}

impl<'a> IntoAstNode<'a> for oxc::ast::ast::Declaration<'a> {
    fn as_node(&'a self, source: &'a str) -> AstNode<'a> {
        AstNode::from_declaration(self, source)
    }
}

impl<'a> IntoAstNode<'a> for Expression<'a> {
    fn as_node(&'a self, source: &'a str) -> AstNode<'a> {
        AstNode::from_expression(self, source)
    }
}
