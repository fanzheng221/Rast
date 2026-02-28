//! Rast AST Engine
//!
//! Core AST analysis engine for JavaScript/TypeScript parsing (oxc-based).

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use oxc::{
    allocator::Allocator,
    ast::{
        ast::{
            CallExpression, Class, ExportDefaultDeclarationKind, ExportNamedDeclaration,
            Expression, ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind,
            MethodDefinition, ModuleExportName, Statement, TSInterfaceDeclaration,
            TSTypeAliasDeclaration, VariableDeclaration, VariableDeclarationKind,
        },
        visit::walk,
        Visit,
    },
    parser::Parser,
    semantic::SemanticBuilder,
    span::{GetSpan, SourceType},
    syntax::scope::ScopeFlags,
};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternNode {
    pub kind: PatternNodeKind,
    pub text: String,
    pub span: NodeSpan,
    pub children: Vec<PatternNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternNodeKind {
    Node { kind: String },
    MetaVar(WildcardNode),
    MultiMetaVar(WildcardNode),
    MultiWildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildcardNode {
    pub name: String,
}

/// `sgconfig.yml` root schema for a single Rast rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCore {
    pub id: String,
    pub language: RuleLanguage,
    pub rule: Rule,
}

/// Alias for the canonical `sgconfig.yml` payload.
pub type SgConfig = RuleCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleLanguage {
    Js,
    Jsx,
    Ts,
    Tsx,
    Javascript,
    Typescript,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rule {
    pub core: RuleKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleKind {
    Pattern(PatternAtomicRule),
    Regex(RegexAtomicRule),
    Kind(KindAtomicRule),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternAtomicRule {
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegexAtomicRule {
    pub regex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindAtomicRule {
    pub kind: String,
}

impl RuleCore {
    pub fn from_yaml(input: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchStrictness {
    Ast,
    Relaxed,
    Cst,
    Signature,
    Template,
}

impl Default for MatchStrictness {
    fn default() -> Self {
        Self::Ast
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedNode {
    pub kind: String,
    pub text: String,
    pub span: NodeSpan,
}

impl<'a> From<AstNode<'a>> for CapturedNode {
    fn from(node: AstNode<'a>) -> Self {
        Self {
            kind: node.kind().to_string(),
            text: node.text(),
            span: node.span(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEnvironment {
    pub single_captures: HashMap<String, CapturedNode>,
    pub multi_captures: HashMap<String, Vec<CapturedNode>>,
}

impl MatchEnvironment {
    pub fn get_single_capture(&self, name: &str) -> Option<&CapturedNode> {
        self.single_captures.get(name)
    }

    pub fn get_multi_capture(&self, name: &str) -> Option<&[CapturedNode]> {
        self.multi_captures.get(name).map(Vec::as_slice)
    }

    pub fn has_single_capture(&self, name: &str) -> bool {
        self.single_captures.contains_key(name)
    }

    pub fn has_multi_capture(&self, name: &str) -> bool {
        self.multi_captures.contains_key(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchOutcome {
    pub matched: bool,
    pub environment: MatchEnvironment,
}

impl MatchOutcome {
    pub fn is_match(&self) -> bool {
        self.matched
    }

    pub fn environment(&self) -> &MatchEnvironment {
        &self.environment
    }

    pub fn into_environment(self) -> MatchEnvironment {
        self.environment
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    PreferOuter,
    PreferInner,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        Self::PreferOuter
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub span: NodeSpan,
    pub environment: MatchEnvironment,
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

    pub fn from_declaration(declaration: &'a oxc::ast::ast::Declaration<'a>, source: &'a str) -> Self {
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

    pub fn from_import_declaration(import_declaration: &'a ImportDeclaration<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::ImportDeclaration(import_declaration),
        }
    }

    pub fn from_variable_declaration(variable_declaration: &'a VariableDeclaration<'a>, source: &'a str) -> Self {
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

    pub fn from_method_definition(method_definition: &'a MethodDefinition<'a>, source: &'a str) -> Self {
        Self {
            source,
            kind: AstNodeKind::MethodDefinition(method_definition),
        }
    }

    pub fn from_ts_interface(interface_declaration: &'a TSInterfaceDeclaration<'a>, source: &'a str) -> Self {
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
            Statement::ImportDeclaration(node) => vec![AstNode::from_import_declaration(node, source)],
            Statement::VariableDeclaration(node) => vec![AstNode::from_variable_declaration(node, source)],
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

    fn declaration_children(declaration: &'a oxc::ast::ast::Declaration<'a>, source: &'a str) -> Vec<AstNode<'a>> {
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

fn is_valid_meta_capture_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_uppercase() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn wildcard_kind_from_identifier(name: &str) -> Option<PatternNodeKind> {
    if name == "$$$" {
        return Some(PatternNodeKind::MultiWildcard);
    }

    if let Some(rest) = name.strip_prefix("$$$") {
        if is_valid_meta_capture_name(rest) {
            return Some(PatternNodeKind::MultiMetaVar(WildcardNode {
                name: rest.to_string(),
            }));
        }
        return None;
    }

    if let Some(rest) = name.strip_prefix('$') {
        if is_valid_meta_capture_name(rest) {
            return Some(PatternNodeKind::MetaVar(WildcardNode {
                name: rest.to_string(),
            }));
        }
    }

    None
}

pub fn identify_meta_variables(root: &AstNode<'_>) -> HashMap<(u32, u32), PatternNodeKind> {
    fn walk(node: AstNode<'_>, output: &mut HashMap<(u32, u32), PatternNodeKind>) {
        let span = node.span();
        if let Some(Expression::Identifier(identifier)) = node.as_expression() {
            if let Some(kind) = wildcard_kind_from_identifier(identifier.name.as_str()) {
                output.insert((span.start, span.end), kind);
            }
        }

        for child in node.children() {
            walk(child, output);
        }
    }

    let mut result = HashMap::new();
    walk(*root, &mut result);
    result
}

fn ast_node_to_pattern(node: AstNode<'_>, meta_variables: &HashMap<(u32, u32), PatternNodeKind>) -> PatternNode {
    let span = node.span();
    let key = (span.start, span.end);
    let kind = meta_variables
        .get(&key)
        .cloned()
        .unwrap_or_else(|| PatternNodeKind::Node {
            kind: node.kind().to_string(),
        });

    let children = node
        .children()
        .into_iter()
        .map(|child| ast_node_to_pattern(child, meta_variables))
        .collect();

    PatternNode {
        kind,
        text: node.text(),
        span,
        children,
    }
}

pub fn to_pattern_ast(root: AstNode<'_>) -> PatternNode {
    let meta_variables = identify_meta_variables(&root);
    ast_node_to_pattern(root, &meta_variables)
}

pub trait Matcher {
    fn match_node_with_env<'a>(
        &self,
        target: AstNode<'a>,
        pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool;

    fn match_result<'a>(&self, target: AstNode<'a>, pattern: &PatternNode) -> MatchOutcome {
        let mut env = MatchEnvironment::default();
        let matched = self.match_node_with_env(target, pattern, &mut env);
        MatchOutcome {
            matched,
            environment: env,
        }
    }

    fn match_node<'a>(&self, target: AstNode<'a>, pattern: &PatternNode) -> Option<MatchEnvironment> {
        let result = self.match_result(target, pattern);
        if result.matched {
            Some(result.environment)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatchCandidate {
    result: MatchResult,
    depth: usize,
    visit_index: usize,
}

fn spans_overlap(left: NodeSpan, right: NodeSpan) -> bool {
    left.start < right.end && right.start < left.end
}

fn span_len(span: NodeSpan) -> u32 {
    span.end.saturating_sub(span.start)
}

fn sort_candidates(candidates: &mut [MatchCandidate], conflict_resolution: ConflictResolution) {
    candidates.sort_by(|left, right| {
        let left_len = span_len(left.result.span);
        let right_len = span_len(right.result.span);

        let length_order = match conflict_resolution {
            ConflictResolution::PreferOuter => right_len.cmp(&left_len),
            ConflictResolution::PreferInner => left_len.cmp(&right_len),
        };

        length_order
            .then_with(|| left.result.span.start.cmp(&right.result.span.start))
            .then_with(|| left.result.span.end.cmp(&right.result.span.end))
            .then_with(|| left.depth.cmp(&right.depth))
            .then_with(|| left.visit_index.cmp(&right.visit_index))
    });
}

pub fn find_all_matches<'a, M: Matcher + ?Sized>(
    matcher: &M,
    target: AstNode<'a>,
    pattern: &PatternNode,
    conflict_resolution: ConflictResolution,
) -> Vec<MatchResult> {
    fn collect_matches<'a, M: Matcher + ?Sized>(
        matcher: &M,
        node: AstNode<'a>,
        pattern: &PatternNode,
        depth: usize,
        visit_index: &mut usize,
        candidates: &mut Vec<MatchCandidate>,
    ) {
        if let Some(environment) = matcher.match_node(node, pattern) {
            candidates.push(MatchCandidate {
                result: MatchResult {
                    span: node.span(),
                    environment,
                },
                depth,
                visit_index: *visit_index,
            });
            *visit_index += 1;
        }

        for child in node.children() {
            collect_matches(
                matcher,
                child,
                pattern,
                depth + 1,
                visit_index,
                candidates,
            );
        }
    }

    let mut candidates = Vec::new();
    let mut visit_index = 0;
    collect_matches(
        matcher,
        target,
        pattern,
        0,
        &mut visit_index,
        &mut candidates,
    );

    sort_candidates(&mut candidates, conflict_resolution);

    let mut selected = Vec::new();
    for candidate in candidates {
        if selected.iter().all(|existing: &MatchCandidate| {
            !spans_overlap(existing.result.span, candidate.result.span)
        }) {
            selected.push(candidate);
        }
    }

    selected.sort_by(|left, right| {
        left.result
            .span
            .start
            .cmp(&right.result.span.start)
            .then_with(|| left.result.span.end.cmp(&right.result.span.end))
            .then_with(|| left.depth.cmp(&right.depth))
            .then_with(|| left.visit_index.cmp(&right.visit_index))
    });
    selected.dedup_by(|left, right| left.result == right.result);

    selected.into_iter().map(|candidate| candidate.result).collect()
}

#[allow(non_snake_case)]
pub fn FindAllMatches<'a, M: Matcher + ?Sized>(
    matcher: &M,
    target: AstNode<'a>,
    pattern: &PatternNode,
    conflict_resolution: ConflictResolution,
) -> Vec<MatchResult> {
    find_all_matches(matcher, target, pattern, conflict_resolution)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternMatcher {
    strictness: MatchStrictness,
}

impl PatternMatcher {
    pub fn new(strictness: MatchStrictness) -> Self {
        Self { strictness }
    }

    pub fn strictness(&self) -> MatchStrictness {
        self.strictness
    }

    fn should_skip_trivia(&self) -> bool {
        !matches!(self.strictness, MatchStrictness::Cst)
    }

    fn should_require_exact_text(&self) -> bool {
        matches!(self.strictness, MatchStrictness::Cst)
    }

    fn node_text_equal(&self, left: &str, right: &str) -> bool {
        match self.strictness {
            MatchStrictness::Cst => left == right,
            MatchStrictness::Signature => normalize_signature_text(left) == normalize_signature_text(right),
            MatchStrictness::Ast | MatchStrictness::Relaxed | MatchStrictness::Template => {
                normalize_text(left) == normalize_text(right)
            }
        }
    }

    fn capture_single(&self, env: &mut MatchEnvironment, name: &str, node: AstNode<'_>) -> bool {
        if env.has_multi_capture(name) {
            return false;
        }

        let captured = CapturedNode::from(node);
        if let Some(existing) = env.get_single_capture(name) {
            self.captured_node_equal(existing, &captured)
        } else {
            env.single_captures.insert(name.to_string(), captured);
            true
        }
    }

    fn capture_multi(
        &self,
        env: &mut MatchEnvironment,
        name: &str,
        nodes: &[AstNode<'_>],
    ) -> bool {
        if env.has_single_capture(name) {
            return false;
        }

        let captured = nodes
            .iter()
            .copied()
            .map(CapturedNode::from)
            .collect::<Vec<_>>();

        if let Some(existing) = env.get_multi_capture(name) {
            existing.len() == captured.len()
                && existing
                    .iter()
                    .zip(captured.iter())
                    .all(|(left, right)| self.captured_node_equal(left, right))
        } else {
            env.multi_captures.insert(name.to_string(), captured);
            true
        }
    }

    fn captured_node_equal(&self, left: &CapturedNode, right: &CapturedNode) -> bool {
        left.kind == right.kind && self.node_text_equal(&left.text, &right.text)
    }

    fn is_ignorable_text(&self, text: &str) -> bool {
        normalize_text(text).is_empty()
    }

    fn should_skip_pattern_child(&self, node: &PatternNode) -> bool {
        if !self.should_skip_trivia() {
            return false;
        }

        matches!(node.kind, PatternNodeKind::Node { .. }) && self.is_ignorable_text(&node.text)
    }

    fn match_regular_node<'a>(
        &self,
        target: AstNode<'a>,
        kind: &str,
        pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool {
        if kind != target.kind() {
            return false;
        }

        if self.should_require_exact_text() && pattern.text != target.text() {
            return false;
        }

        let target_children = target
            .children()
            .into_iter()
            .filter(|node| !self.should_skip_trivia() || !self.is_ignorable_text(&node.text()))
            .collect::<Vec<_>>();

        let pattern_children = pattern
            .children
            .iter()
            .filter(|node| !self.should_skip_pattern_child(node))
            .collect::<Vec<_>>();

        if target_children.is_empty() && pattern_children.is_empty() {
            return self.node_text_equal(&pattern.text, &target.text());
        }

        self.match_sequence(&target_children, &pattern_children, 0, 0, env)
    }

    fn match_sequence<'a>(
        &self,
        target_nodes: &[AstNode<'a>],
        pattern_nodes: &[&PatternNode],
        target_index: usize,
        pattern_index: usize,
        env: &mut MatchEnvironment,
    ) -> bool {
        if pattern_index == pattern_nodes.len() {
            return target_index == target_nodes.len();
        }

        let current_pattern = pattern_nodes[pattern_index];

        match &current_pattern.kind {
            PatternNodeKind::MultiWildcard => {
                for next_target in target_index..=target_nodes.len() {
                    let mut next_env = env.clone();
                    if self.match_sequence(
                        target_nodes,
                        pattern_nodes,
                        next_target,
                        pattern_index + 1,
                        &mut next_env,
                    ) {
                        *env = next_env;
                        return true;
                    }
                }
                false
            }
            PatternNodeKind::MultiMetaVar(meta) => {
                for next_target in target_index..=target_nodes.len() {
                    let mut next_env = env.clone();
                    let slice = &target_nodes[target_index..next_target];
                    if self.capture_multi(&mut next_env, &meta.name, slice)
                        && self.match_sequence(
                            target_nodes,
                            pattern_nodes,
                            next_target,
                            pattern_index + 1,
                            &mut next_env,
                        )
                    {
                        *env = next_env;
                        return true;
                    }
                }
                false
            }
            _ => {
                if target_index >= target_nodes.len() {
                    return false;
                }

                let mut next_env = env.clone();
                if self.match_node_with_env(
                    target_nodes[target_index],
                    current_pattern,
                    &mut next_env,
                ) && self.match_sequence(
                    target_nodes,
                    pattern_nodes,
                    target_index + 1,
                    pattern_index + 1,
                    &mut next_env,
                ) {
                    *env = next_env;
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self {
            strictness: MatchStrictness::default(),
        }
    }
}

impl Matcher for PatternMatcher {
    fn match_node_with_env<'a>(
        &self,
        target: AstNode<'a>,
        pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool {
        match &pattern.kind {
            PatternNodeKind::Node { kind } => self.match_regular_node(target, kind, pattern, env),
            PatternNodeKind::MetaVar(meta) => self.capture_single(env, &meta.name, target),
            PatternNodeKind::MultiMetaVar(_) | PatternNodeKind::MultiWildcard => false,
        }
    }
}

#[derive(Default)]
pub struct CompositeMatcher {
    matchers: Vec<Box<dyn Matcher>>,
}

impl CompositeMatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<M: Matcher + 'static>(&mut self, matcher: M) {
        self.matchers.push(Box::new(matcher));
    }
}

impl Matcher for CompositeMatcher {
    fn match_node_with_env<'a>(
        &self,
        target: AstNode<'a>,
        pattern: &PatternNode,
        env: &mut MatchEnvironment,
    ) -> bool {
        let mut current = env.clone();
        for matcher in &self.matchers {
            if !matcher.match_node_with_env(target, pattern, &mut current) {
                return false;
            }
        }
        *env = current;
        true
    }
}

fn normalize_text(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '/' {
            match chars.peek().copied() {
                Some('/') => {
                    let _ = chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' {
                            break;
                        }
                    }
                }
                Some('*') => {
                    let _ = chars.next();
                    let mut prev = '\0';
                    for next in chars.by_ref() {
                        if prev == '*' && next == '/' {
                            break;
                        }
                        prev = next;
                    }
                }
                _ => output.push(ch),
            }
            continue;
        }

        if !ch.is_whitespace() {
            output.push(ch);
        }
    }

    output
}

fn normalize_signature_text(text: &str) -> String {
    let normalized = normalize_text(text);
    if let Some(idx) = normalized.find('{') {
        normalized[..idx].to_string()
    } else {
        normalized
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

/// Represents an exported identifier from a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    /// Name of exported item
    pub name: String,
    /// Type of export: "function", "variable", "class", "type", "interface"
    pub kind: String,
    /// Source location (line, column)
    pub location: Option<(usize, usize)>,
}

/// Represents a linting issue found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    /// Category of lint issue
    pub category: String,
    /// Severity level
    pub severity: String,
    /// Description of the issue
    pub message: String,
    /// Source location (line, column)
    pub location: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSignature {
    pub name: String,
    pub kind: String,
    pub params: Vec<String>,
    pub return_type: Option<String>,
    pub type_params: Vec<String>,
    pub exported: bool,
    pub location: Option<(usize, usize)>,
    pub jsdoc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub source: String,
    pub kind: String,
    pub specifiers: Vec<String>,
    pub is_type_only: bool,
    pub location: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: String,
    pub callee: String,
    pub location: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CallGraph {
    pub edges: Vec<CallEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileStructure {
    pub language: String,
    pub imports: Vec<DependencyInfo>,
    pub exports: Vec<SymbolSignature>,
    pub symbols: Vec<SymbolSignature>,
    pub classes: Vec<SymbolSignature>,
    pub interfaces: Vec<SymbolSignature>,
    pub type_aliases: Vec<SymbolSignature>,
    pub comments: Vec<String>,
    pub jsdoc: Vec<String>,
    pub call_graph: CallGraph,
}

/// Result of AST analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// List of exports found in module
    pub exports: Vec<ExportInfo>,
    /// List of linting issues found
    pub issues: Vec<LintIssue>,
    pub file_structure: FileStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueSfcScriptKind {
    Script,
    ScriptSetup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VueSfcBlockPresence {
    pub has_script: bool,
    pub has_script_setup: bool,
    pub has_template: bool,
    pub has_style: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueSfcOffsetMap {
    script_span: NodeSpan,
    relative_to_absolute_offsets: Vec<u32>,
    line_starts: Vec<usize>,
}

impl VueSfcOffsetMap {
    fn new(script_span: NodeSpan, source: &str) -> Self {
        let start = script_span.start;
        let end = script_span.end;
        let relative_to_absolute_offsets = (start..=end).collect::<Vec<_>>();
        Self {
            script_span,
            relative_to_absolute_offsets,
            line_starts: compute_line_starts(source),
        }
    }

    pub fn script_span(&self) -> NodeSpan {
        self.script_span
    }

    pub fn relative_to_absolute_offset(&self, relative_offset: u32) -> Option<u32> {
        self.relative_to_absolute_offsets
            .get(relative_offset as usize)
            .copied()
    }

    pub fn absolute_to_relative_offset(&self, absolute_offset: u32) -> Option<u32> {
        if absolute_offset < self.script_span.start || absolute_offset > self.script_span.end {
            return None;
        }
        Some(absolute_offset - self.script_span.start)
    }

    pub fn relative_to_absolute_span(&self, relative_span: NodeSpan) -> Option<NodeSpan> {
        let start = self.relative_to_absolute_offset(relative_span.start)?;
        let end = self.relative_to_absolute_offset(relative_span.end)?;
        Some(NodeSpan { start, end })
    }

    pub fn absolute_to_relative_span(&self, absolute_span: NodeSpan) -> Option<NodeSpan> {
        let start = self.absolute_to_relative_offset(absolute_span.start)?;
        let end = self.absolute_to_relative_offset(absolute_span.end)?;
        Some(NodeSpan { start, end })
    }

    pub fn absolute_offset_to_line_col(&self, absolute_offset: u32) -> (usize, usize) {
        offset_to_line_col(absolute_offset as usize, &self.line_starts)
    }

    pub fn relative_offset_to_line_col(&self, relative_offset: u32) -> Option<(usize, usize)> {
        let absolute = self.relative_to_absolute_offset(relative_offset)?;
        Some(self.absolute_offset_to_line_col(absolute))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedScriptBlock<'a> {
    pub content: &'a str,
    pub kind: VueSfcScriptKind,
    pub offset_map: VueSfcOffsetMap,
    pub block_presence: VueSfcBlockPresence,
}

#[derive(Debug, Clone)]
pub struct VueSfcExtractor<'a> {
    source: &'a str,
    block_presence: VueSfcBlockPresence,
}

impl<'a> VueSfcExtractor<'a> {
    pub fn new(source: &'a str) -> Self {
        let script_blocks = collect_sfc_script_blocks(source);
        let has_script = script_blocks
            .iter()
            .any(|block| block.kind == VueSfcScriptKind::Script);
        let has_script_setup = script_blocks
            .iter()
            .any(|block| block.kind == VueSfcScriptKind::ScriptSetup);
        let has_template = find_sfc_block(source, "template", 0).is_some();
        let has_style = find_sfc_block(source, "style", 0).is_some();

        Self {
            source,
            block_presence: VueSfcBlockPresence {
                has_script,
                has_script_setup,
                has_template,
                has_style,
            },
        }
    }

    pub fn block_presence(&self) -> VueSfcBlockPresence {
        self.block_presence
    }

    pub fn extract_script_block(&self) -> Option<ExtractedScriptBlock<'a>> {
        let block = collect_sfc_script_blocks(self.source).into_iter().next()?;
        let script_span = NodeSpan {
            start: block.content_start as u32,
            end: block.content_end as u32,
        };
        let content = &self.source[block.content_start..block.content_end];

        Some(ExtractedScriptBlock {
            content,
            kind: block.kind,
            offset_map: VueSfcOffsetMap::new(script_span, self.source),
            block_presence: self.block_presence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SfcBlockMatch {
    content_start: usize,
    content_end: usize,
    kind: VueSfcScriptKind,
}

fn collect_sfc_script_blocks(source: &str) -> Vec<SfcBlockMatch> {
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    while let Some(block) = find_sfc_block(source, "script", cursor) {
        let next_cursor = block.content_end.saturating_add(1);
        blocks.push(block);
        if next_cursor > source.len() {
            break;
        }
        cursor = next_cursor;
    }
    blocks
}

fn find_sfc_block(source: &str, tag_name: &str, from_offset: usize) -> Option<SfcBlockMatch> {
    let lower_source = source.to_ascii_lowercase();
    let lower_tag_name = tag_name.to_ascii_lowercase();
    let open_tag_prefix = format!("<{}", lower_tag_name);
    let close_tag = format!("</{}>", lower_tag_name);

    let mut cursor = from_offset;
    while cursor < lower_source.len() {
        let open_start = lower_source[cursor..].find(&open_tag_prefix)? + cursor;
        let boundary_idx = open_start + open_tag_prefix.len();
        if !is_sfc_tag_boundary(lower_source.as_bytes().get(boundary_idx).copied()) {
            cursor = boundary_idx;
            continue;
        }

        let open_end = lower_source[boundary_idx..].find('>')? + boundary_idx;
        let content_start = open_end + 1;
        let close_start = lower_source[content_start..].find(&close_tag)? + content_start;
        let open_tag_content = &source[open_start..=open_end];

        let kind = if lower_tag_name == "script" && tag_has_attribute(open_tag_content, "setup") {
            VueSfcScriptKind::ScriptSetup
        } else {
            VueSfcScriptKind::Script
        };

        return Some(SfcBlockMatch {
            content_start,
            content_end: close_start,
            kind,
        });
    }
    None
}

fn is_sfc_tag_boundary(ch: Option<u8>) -> bool {
    matches!(ch, None | Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b'\t'))
}

fn tag_has_attribute(open_tag: &str, attr_name: &str) -> bool {
    let lower = open_tag.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut idx = 1usize;

    while idx < bytes.len() && bytes[idx].is_ascii_alphabetic() {
        idx += 1;
    }

    while idx < bytes.len() {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] == b'>' || bytes[idx] == b'/' {
            break;
        }

        let attr_start = idx;
        while idx < bytes.len()
            && !bytes[idx].is_ascii_whitespace()
            && bytes[idx] != b'='
            && bytes[idx] != b'>'
            && bytes[idx] != b'/'
        {
            idx += 1;
        }

        if &lower[attr_start..idx] == attr_name {
            return true;
        }

        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }

        if idx < bytes.len() && bytes[idx] == b'=' {
            idx += 1;
            while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }

            if idx < bytes.len() && (bytes[idx] == b'"' || bytes[idx] == b'\'') {
                let quote = bytes[idx];
                idx += 1;
                while idx < bytes.len() && bytes[idx] != quote {
                    idx += 1;
                }
                if idx < bytes.len() {
                    idx += 1;
                }
            } else {
                while idx < bytes.len()
                    && !bytes[idx].is_ascii_whitespace()
                    && bytes[idx] != b'>'
                {
                    idx += 1;
                }
            }
        }
    }

    false
}

#[derive(Debug, Default)]
struct ProjectGraphState {
    files: HashMap<String, FileStructure>,
    dependency_graph: HashMap<String, Vec<String>>,
    parse_cache: HashMap<String, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectGraph {
    inner: Arc<RwLock<ProjectGraphState>>,
}

impl ProjectGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&self, path: &str, code: &str) -> Result<(), String> {
        let normalized_path = normalize_project_path(path);
        let code_hash = hash_code(code);

        {
            let state = self
                .inner
                .read()
                .map_err(|err| format!("Failed to acquire graph read lock: {err}"))?;
            if state
                .parse_cache
                .get(&normalized_path)
                .is_some_and(|cached| *cached == code_hash)
            {
                return Ok(());
            }
        }

        let result = analyze_ast_internal(code);
        let dependencies = result
            .file_structure
            .imports
            .iter()
            .filter_map(|dep| {
                resolve_import_candidates(&normalized_path, &dep.source)
                    .into_iter()
                    .next()
            })
            .collect::<Vec<_>>();

        let mut state = self
            .inner
            .write()
            .map_err(|err| format!("Failed to acquire graph write lock: {err}"))?;
        state
            .files
            .insert(normalized_path.clone(), result.file_structure);
        state
            .dependency_graph
            .insert(normalized_path.clone(), dependencies);
        state.parse_cache.insert(normalized_path, code_hash);
        Ok(())
    }

    pub fn get_file_structure(&self, path: &str) -> Option<FileStructure> {
        let normalized_path = normalize_project_path(path);
        let state = self.inner.read().ok()?;
        state.files.get(&normalized_path).cloned()
    }

    pub fn get_all_files(&self) -> Vec<String> {
        let state = self.inner.read().ok();
        let mut files = state
            .map(|graph| graph.files.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        files.sort();
        files
    }

    pub fn resolve_dependencies(&self, path: &str) -> Vec<DependencyInfo> {
        let normalized_path = normalize_project_path(path);
        let state = match self.inner.read() {
            Ok(state) => state,
            Err(_) => return Vec::new(),
        };

        let Some(file_structure) = state.files.get(&normalized_path) else {
            return Vec::new();
        };

        file_structure
            .imports
            .iter()
            .filter_map(|dep| {
                let resolved = resolve_import_candidates(&normalized_path, &dep.source)
                    .into_iter()
                    .find(|candidate| state.files.contains_key(candidate))?;
                if state.files.contains_key(&resolved) {
                    let mut info = dep.clone();
                    info.source = resolved;
                    Some(info)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn find_symbol(&self, name: &str) -> Vec<SymbolSignature> {
        let state = match self.inner.read() {
            Ok(state) => state,
            Err(_) => return Vec::new(),
        };

        let mut symbols = Vec::new();
        for structure in state.files.values() {
            let mut local = structure
                .symbols
                .iter()
                .filter(|symbol| symbol.name == name)
                .cloned()
                .collect::<Vec<_>>();

            for export in structure
                .exports
                .iter()
                .filter(|symbol| symbol.name == name)
            {
                if !local
                    .iter()
                    .any(|symbol| symbol.name == export.name && symbol.kind == export.kind)
                {
                    local.push(export.clone());
                }
            }

            symbols.extend(local);
        }

        symbols
    }

    pub fn clear(&self) {
        if let Ok(mut state) = self.inner.write() {
            state.files.clear();
            state.dependency_graph.clear();
            state.parse_cache.clear();
        }
    }
}

/// Analyzes JavaScript/TypeScript source code and extracts AST information
///
/// # Arguments
/// * `source` - The source code to analyze
///
/// # Returns
/// * `String` - JSON string containing analysis result
pub fn analyze_ast(source: &str) -> String {
    let result = analyze_ast_internal(source);
    serde_json::to_string(&result)
        .unwrap_or_else(|_| String::from("{\"error\":\"Failed to serialize result\"}"))
}

#[derive(Debug)]
struct Analyzer<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
    symbols: Vec<SymbolSignature>,
    classes: Vec<SymbolSignature>,
    interfaces: Vec<SymbolSignature>,
    type_aliases: Vec<SymbolSignature>,
    call_edges: Vec<CallEdge>,
    lint_issues: Vec<LintIssue>,
    function_stack: Vec<String>,
    class_stack: Vec<String>,
}

impl<'a> Analyzer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            line_starts: compute_line_starts(source),
            symbols: Vec::new(),
            classes: Vec::new(),
            interfaces: Vec::new(),
            type_aliases: Vec::new(),
            call_edges: Vec::new(),
            lint_issues: Vec::new(),
            function_stack: vec!["<module>".to_string()],
            class_stack: Vec::new(),
        }
    }

    fn location(&self, span: oxc::span::Span) -> Option<(usize, usize)> {
        Some(offset_to_line_col(span.start as usize, &self.line_starts))
    }

    fn span_text(&self, span: oxc::span::Span) -> String {
        span.source_text(self.source).trim().to_string()
    }

    fn current_scope_name(&self) -> String {
        self.function_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "<module>".to_string())
    }
}

impl<'a> Visit<'a> for Analyzer<'a> {
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if it.kind == VariableDeclarationKind::Var {
            self.lint_issues.push(LintIssue {
                category: "best-practices".to_string(),
                severity: "warning".to_string(),
                message: "Avoid using 'var'. Use 'const' or 'let' instead.".to_string(),
                location: self.location(it.span),
            });
        }
        walk::walk_variable_declaration(self, it);
    }

    fn visit_function(&mut self, it: &oxc::ast::ast::Function<'a>, flags: ScopeFlags) {
        let name = it.name().map(|name| name.to_string()).unwrap_or_else(|| {
            format!("<anonymous@{}>", self.location(it.span).map_or(0, |v| v.0))
        });

        let params = it
            .params
            .items
            .iter()
            .filter_map(|param| param.pattern.get_identifier())
            .map(|name| name.to_string())
            .collect::<Vec<_>>();

        let return_type = it.return_type.as_ref().map(|ret| self.span_text(ret.span));
        let type_params = it
            .type_parameters
            .as_ref()
            .map(|params| {
                params
                    .params
                    .iter()
                    .map(|p| p.name.name.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        self.symbols.push(SymbolSignature {
            name: name.clone(),
            kind: "function".to_string(),
            params,
            return_type,
            type_params,
            exported: false,
            location: self.location(it.span),
            jsdoc: None,
        });

        self.function_stack.push(name);
        walk::walk_function(self, it, flags);
        self.function_stack.pop();
    }

    fn visit_class(&mut self, it: &Class<'a>) {
        let class_name = it
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .unwrap_or_else(|| "<anonymous-class>".to_string());

        let class_signature = SymbolSignature {
            name: class_name.clone(),
            kind: "class".to_string(),
            params: Vec::new(),
            return_type: None,
            type_params: Vec::new(),
            exported: false,
            location: self.location(it.span),
            jsdoc: None,
        };

        self.classes.push(class_signature.clone());
        self.symbols.push(class_signature);
        self.class_stack.push(class_name);
        walk::walk_class(self, it);
        self.class_stack.pop();
    }

    fn visit_method_definition(&mut self, it: &MethodDefinition<'a>) {
        let class_name = self
            .class_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "<class>".to_string());
        let method_name = it
            .key
            .name()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<computed>".to_string());
        let qualified = format!("{class_name}.{method_name}");

        self.symbols.push(SymbolSignature {
            name: qualified.clone(),
            kind: "method".to_string(),
            params: it
                .value
                .params
                .items
                .iter()
                .filter_map(|param| param.pattern.get_identifier())
                .map(|name| name.to_string())
                .collect(),
            return_type: it
                .value
                .return_type
                .as_ref()
                .map(|return_type| self.span_text(return_type.span)),
            type_params: Vec::new(),
            exported: false,
            location: self.location(it.span),
            jsdoc: None,
        });

        self.function_stack.push(qualified);
        walk::walk_method_definition(self, it);
        self.function_stack.pop();
    }

    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        let signature = SymbolSignature {
            name: it.id.name.to_string(),
            kind: "interface".to_string(),
            params: Vec::new(),
            return_type: None,
            type_params: it
                .type_parameters
                .as_ref()
                .map(|params| {
                    params
                        .params
                        .iter()
                        .map(|p| p.name.name.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            exported: false,
            location: self.location(it.span),
            jsdoc: None,
        };

        self.interfaces.push(signature.clone());
        self.symbols.push(signature);
        walk::walk_ts_interface_declaration(self, it);
    }

    fn visit_ts_type_alias_declaration(&mut self, it: &TSTypeAliasDeclaration<'a>) {
        let signature = SymbolSignature {
            name: it.id.name.to_string(),
            kind: "type".to_string(),
            params: Vec::new(),
            return_type: Some(self.span_text(it.type_annotation.span())),
            type_params: it
                .type_parameters
                .as_ref()
                .map(|params| {
                    params
                        .params
                        .iter()
                        .map(|p| p.name.name.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            exported: false,
            location: self.location(it.span),
            jsdoc: None,
        };

        self.type_aliases.push(signature.clone());
        self.symbols.push(signature);
        walk::walk_ts_type_alias_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(callee) = expression_name(&it.callee) {
            self.call_edges.push(CallEdge {
                caller: self.current_scope_name(),
                callee,
                location: self.location(it.span),
            });
        }

        if let Some(member) = it.callee.as_member_expression() {
            if member.object().is_specific_id("console") {
                self.lint_issues.push(LintIssue {
                    category: "dev-code".to_string(),
                    severity: "warning".to_string(),
                    message: "Console statement detected. Remove in production code.".to_string(),
                    location: self.location(it.span),
                });
            }
        }

        walk::walk_call_expression(self, it);
    }
}

fn analyze_ast_internal(source: &str) -> AnalysisResult {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("inline.tsx")).unwrap_or_else(|_| {
        SourceType::unambiguous()
            .with_typescript(true)
            .with_jsx(true)
    });

    let parser_return = Parser::new(&allocator, source, source_type).parse();
    let mut issues = parser_return
        .errors
        .iter()
        .map(|err| LintIssue {
            category: "parse".to_string(),
            severity: "error".to_string(),
            message: err.to_string(),
            location: None,
        })
        .collect::<Vec<_>>();

    let semantic_return = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_jsdoc(true)
        .build(&parser_return.program);

    issues.extend(semantic_return.errors.iter().map(|err| LintIssue {
        category: "semantic".to_string(),
        severity: "error".to_string(),
        message: err.to_string(),
        location: None,
    }));

    let semantic = semantic_return.semantic;
    let mut analyzer = Analyzer::new(source);
    analyzer.visit_program(&parser_return.program);
    issues.extend(analyzer.lint_issues);

    let imports = parser_return
        .program
        .body
        .iter()
        .filter_map(|stmt| {
            if let Statement::ImportDeclaration(import_decl) = stmt {
                Some(extract_import(import_decl, &analyzer.line_starts))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut exports = Vec::new();
    let mut export_signatures = Vec::new();
    for stmt in &parser_return.program.body {
        extract_exports(
            stmt,
            &analyzer.line_starts,
            &mut exports,
            &mut export_signatures,
        );
    }

    let comments = parser_return
        .program
        .comments
        .iter()
        .map(|comment| comment.span.source_text(source).to_string())
        .collect::<Vec<_>>();

    let jsdoc = semantic
        .jsdoc()
        .iter_all()
        .map(|doc| doc.span.source_text(source).to_string())
        .collect::<Vec<_>>();

    let file_structure = FileStructure {
        language: language_name(parser_return.program.source_type),
        imports,
        exports: export_signatures,
        symbols: analyzer.symbols,
        classes: analyzer.classes,
        interfaces: analyzer.interfaces,
        type_aliases: analyzer.type_aliases,
        comments,
        jsdoc,
        call_graph: CallGraph {
            edges: analyzer.call_edges,
        },
    };

    AnalysisResult {
        exports,
        issues,
        file_structure,
    }
}

fn extract_import(import_decl: &ImportDeclaration<'_>, line_starts: &[usize]) -> DependencyInfo {
    let specifiers = import_decl
        .specifiers
        .as_ref()
        .map(|specifiers| {
            specifiers
                .iter()
                .map(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        spec.local.name.to_string()
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        spec.local.name.to_string()
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        format!("* as {}", spec.local.name)
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    DependencyInfo {
        source: import_decl.source.value.to_string(),
        kind: "import".to_string(),
        specifiers,
        is_type_only: import_decl.import_kind == ImportOrExportKind::Type,
        location: Some(offset_to_line_col(
            import_decl.span.start as usize,
            line_starts,
        )),
    }
}

fn extract_exports(
    stmt: &Statement<'_>,
    line_starts: &[usize],
    exports: &mut Vec<ExportInfo>,
    signatures: &mut Vec<SymbolSignature>,
) {
    match stmt {
        Statement::ExportNamedDeclaration(decl) => {
            extract_named_export(decl, line_starts, exports, signatures);
        }
        Statement::ExportDefaultDeclaration(decl) => {
            let (name, kind) = match &decl.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(func) => (
                    func.name()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "default".to_string()),
                    "function".to_string(),
                ),
                ExportDefaultDeclarationKind::ClassDeclaration(class) => (
                    class
                        .id
                        .as_ref()
                        .map(|id| id.name.to_string())
                        .unwrap_or_else(|| "default".to_string()),
                    "class".to_string(),
                ),
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface_decl) => {
                    (interface_decl.id.name.to_string(), "interface".to_string())
                }
                _ => ("default".to_string(), "value".to_string()),
            };

            let location = Some(offset_to_line_col(decl.span.start as usize, line_starts));
            exports.push(ExportInfo {
                name: name.clone(),
                kind: kind.clone(),
                location,
            });
            signatures.push(SymbolSignature {
                name,
                kind,
                params: Vec::new(),
                return_type: None,
                type_params: Vec::new(),
                exported: true,
                location,
                jsdoc: None,
            });
        }
        Statement::ExportAllDeclaration(decl) => {
            let name = decl
                .exported
                .as_ref()
                .map(export_name)
                .unwrap_or_else(|| "*".to_string());
            let location = Some(offset_to_line_col(decl.span.start as usize, line_starts));
            exports.push(ExportInfo {
                name: name.clone(),
                kind: if decl.export_kind == ImportOrExportKind::Type {
                    "type".to_string()
                } else {
                    "value".to_string()
                },
                location,
            });
            signatures.push(SymbolSignature {
                name,
                kind: "re-export".to_string(),
                params: Vec::new(),
                return_type: None,
                type_params: Vec::new(),
                exported: true,
                location,
                jsdoc: None,
            });
        }
        _ => {}
    }
}

fn extract_named_export(
    decl: &ExportNamedDeclaration<'_>,
    line_starts: &[usize],
    exports: &mut Vec<ExportInfo>,
    signatures: &mut Vec<SymbolSignature>,
) {
    let before_len = exports.len();
    let location = Some(offset_to_line_col(decl.span.start as usize, line_starts));
    if let Some(declaration) = &decl.declaration {
        match declaration {
            oxc::ast::ast::Declaration::FunctionDeclaration(function) => {
                if let Some(name) = function.name() {
                    exports.push(ExportInfo {
                        name: name.to_string(),
                        kind: "function".to_string(),
                        location,
                    });
                }
            }
            oxc::ast::ast::Declaration::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    exports.push(ExportInfo {
                        name: id.name.to_string(),
                        kind: "class".to_string(),
                        location,
                    });
                }
            }
            oxc::ast::ast::Declaration::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(name) = declarator.id.get_identifier() {
                        exports.push(ExportInfo {
                            name: name.to_string(),
                            kind: "variable".to_string(),
                            location,
                        });
                    }
                }
            }
            oxc::ast::ast::Declaration::TSInterfaceDeclaration(interface_decl) => {
                exports.push(ExportInfo {
                    name: interface_decl.id.name.to_string(),
                    kind: "interface".to_string(),
                    location,
                });
            }
            oxc::ast::ast::Declaration::TSTypeAliasDeclaration(type_alias) => {
                exports.push(ExportInfo {
                    name: type_alias.id.name.to_string(),
                    kind: "type".to_string(),
                    location,
                });
            }
            _ => {}
        }
    }

    for specifier in &decl.specifiers {
        let kind = if specifier.export_kind == ImportOrExportKind::Type {
            "type"
        } else {
            "value"
        };
        exports.push(ExportInfo {
            name: export_name(&specifier.exported),
            kind: kind.to_string(),
            location,
        });
    }

    signatures.extend(exports[before_len..].iter().map(|export| SymbolSignature {
        name: export.name.clone(),
        kind: export.kind.clone(),
        params: Vec::new(),
        return_type: None,
        type_params: Vec::new(),
        exported: true,
        location: export.location,
        jsdoc: None,
    }));
}

fn export_name(export: &ModuleExportName<'_>) -> String {
    export.name().to_string()
}

fn expression_name(expression: &Expression<'_>) -> Option<String> {
    if let Some(name) = expression.get_identifier_reference() {
        return Some(name.name.to_string());
    }
    expression.as_member_expression().map(|member| {
        let object = expression_name(member.object()).unwrap_or_else(|| "<expr>".to_string());
        let property = member
            .static_property_name()
            .map(|property| property.to_string())
            .unwrap_or_else(|| "<computed>".to_string());
        format!("{object}.{property}")
    })
}

fn language_name(source_type: SourceType) -> String {
    if source_type.is_typescript() {
        if source_type.is_jsx() {
            "tsx".to_string()
        } else {
            "ts".to_string()
        }
    } else if source_type.is_jsx() {
        "jsx".to_string()
    } else {
        "js".to_string()
    }
}

fn hash_code(code: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

fn normalize_project_path(path: &str) -> String {
    normalize_path_buf(PathBuf::from(path))
}

fn normalize_path_buf(path: PathBuf) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized.to_string_lossy().replace('\\', "/")
}

fn resolve_import_candidates(file_path: &str, import_source: &str) -> Vec<String> {
    if !import_source.starts_with('.') {
        return Vec::new();
    }

    let parent = Path::new(file_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let candidate = normalize_path_buf(parent.join(import_source));
    [
        candidate.clone(),
        format!("{candidate}.ts"),
        format!("{candidate}.tsx"),
        format!("{candidate}.js"),
        format!("{candidate}.jsx"),
        format!("{candidate}/index.ts"),
        format!("{candidate}/index.tsx"),
        format!("{candidate}/index.js"),
        format!("{candidate}/index.jsx"),
    ]
    .to_vec()
}

fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (index, byte) in source.as_bytes().iter().enumerate() {
        if *byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (usize, usize) {
    let line_idx = line_starts
        .partition_point(|start| *start <= offset)
        .saturating_sub(1);
    let column = offset.saturating_sub(line_starts[line_idx]);
    (line_idx + 1, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty_code() {
        let result = analyze_ast("");
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.exports.len(), 0);
        assert_eq!(parsed.issues.len(), 0);
    }

    #[test]
    fn test_analyze_function_export() {
        let code = r#"
export function testFunction() {
    return 42;
}
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.exports.len(), 1);
        assert_eq!(parsed.exports[0].name, "testFunction");
        assert_eq!(parsed.exports[0].kind, "function");
    }

    #[test]
    fn test_analyze_multiple_exports() {
        let code = r#"
export const myVar = 123;
export function myFunc() {}
export class MyClass {}
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.exports.len(), 3);
    }

    #[test]
    fn test_detect_var_declaration() {
        let code = r#"
var x = 1;
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(parsed.issues[0].category, "best-practices");
        assert!(parsed.issues[0].message.contains("var"));
    }

    #[test]
    fn test_detect_console_log() {
        let code = r#"
function test() {
    console.log("debug");
}
        "#;
        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(parsed.issues[0].category, "dev-code");
        // Check that the message is about Console (capital C)
        assert!(parsed.issues[0].message.contains("Console"));
    }

    #[test]
    fn test_serialization() {
        let code = "export const x = 1;";
        let result = analyze_ast(code);
        // Should be valid JSON
        serde_json::from_str::<serde_json::Value>(&result).unwrap();
    }

    #[test]
    fn test_rule_core_yaml_pattern_rule() {
        let yaml = r#"
id: no-console-log
language: ts
rule:
  pattern: console.log($$$ARGS)
"#;

        let parsed = RuleCore::from_yaml(yaml).unwrap();
        assert_eq!(parsed.id, "no-console-log");
        assert_eq!(parsed.language, RuleLanguage::Ts);

        match parsed.rule.core {
            RuleKind::Pattern(rule) => {
                assert_eq!(rule.pattern, "console.log($$$ARGS)");
            }
            _ => panic!("expected pattern rule"),
        }
    }

    #[test]
    fn test_rule_core_yaml_regex_rule() {
        let yaml = r#"
id: no-debugger
language: javascript
rule:
  regex: "\\bdebugger\\b"
"#;

        let parsed = RuleCore::from_yaml(yaml).unwrap();
        assert_eq!(parsed.language, RuleLanguage::Javascript);

        match parsed.rule.core {
            RuleKind::Regex(rule) => {
                assert_eq!(rule.regex, "\\bdebugger\\b");
            }
            _ => panic!("expected regex rule"),
        }
    }

    #[test]
    fn test_rule_core_yaml_kind_rule() {
        let yaml = r#"
id: avoid-with
language: js
rule:
  kind: WithStatement
"#;

        let parsed = RuleCore::from_yaml(yaml).unwrap();
        assert_eq!(parsed.language, RuleLanguage::Js);

        match parsed.rule.core {
            RuleKind::Kind(rule) => {
                assert_eq!(rule.kind, "WithStatement");
            }
            _ => panic!("expected kind rule"),
        }
    }

    #[test]
    fn test_rule_core_yaml_rejects_multiple_atomic_keys() {
        let yaml = r#"
id: invalid-rule
language: tsx
rule:
  pattern: foo($A)
  kind: CallExpression
"#;

        let parsed = RuleCore::from_yaml(yaml);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_complex_typescript_structure() {
        let code = r#"
import type { User } from "./types";
import React, { useMemo } from "react";

/** User service */
export interface Service<T> {
  get(id: string): Promise<T>;
}

export type Maybe<T> = T | null;

export class UserService implements Service<User> {
  get(id: string): Promise<User> {
    return fetchUser(id);
  }
}

export function buildName(user: User): string {
  console.log(user.id);
  return user.name;
}
"#;

        let result = analyze_ast(code);
        let parsed: AnalysisResult = serde_json::from_str(&result).unwrap();
        assert!(parsed.file_structure.imports.len() >= 2);
        assert!(parsed
            .file_structure
            .call_graph
            .edges
            .iter()
            .any(|e| e.callee.contains("console")));
        assert!(parsed
            .exports
            .iter()
            .any(|export| export.name == "buildName" && export.kind == "function"));
    }

    #[test]
    fn test_project_graph_add_and_query_files() {
        let graph = ProjectGraph::new();
        let utils = r#"export function helper(): string { return \"ok\"; }"#;
        let app = r#"
import { helper } from "./utils";
export function run() {
  return helper();
}
"#;

        graph.add_file("src/utils.ts", utils).unwrap();
        graph.add_file("src/app.ts", app).unwrap();

        let files = graph.get_all_files();
        assert_eq!(
            files,
            vec!["src/app.ts".to_string(), "src/utils.ts".to_string()]
        );
        assert!(graph.get_file_structure("src/app.ts").is_some());
        assert!(graph.get_file_structure("src/missing.ts").is_none());
    }

    #[test]
    fn test_project_graph_resolve_dependencies_cross_file() {
        let graph = ProjectGraph::new();
        graph
            .add_file("src/utils.ts", "export const helper = () => 1;")
            .unwrap();
        graph
            .add_file(
                "src/app.ts",
                "import { helper } from './utils'; export const run = () => helper();",
            )
            .unwrap();

        let deps = graph.resolve_dependencies("src/app.ts");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source, "src/utils.ts");
        assert_eq!(deps[0].kind, "import");
    }

    #[test]
    fn test_project_graph_find_symbol_and_clear() {
        let graph = ProjectGraph::new();
        graph
            .add_file(
                "src/a.ts",
                "export interface User { id: string } export const userName = 'A';",
            )
            .unwrap();
        graph
            .add_file("src/b.ts", "export const userName = 'B';")
            .unwrap();

        let user_name_symbols = graph.find_symbol("userName");
        assert_eq!(user_name_symbols.len(), 2);

        let user_symbols = graph.find_symbol("User");
        assert_eq!(user_symbols.len(), 1);
        assert_eq!(user_symbols[0].kind, "interface");

        graph.clear();
        assert!(graph.get_all_files().is_empty());
        assert!(graph.find_symbol("userName").is_empty());
    }

    #[test]
    fn test_project_graph_repeated_add_uses_cache() {
        let graph = ProjectGraph::new();
        let code = "export const cached = 1;";
        graph.add_file("src/cache.ts", code).unwrap();
        graph.add_file("src/cache.ts", code).unwrap();

        let files = graph.get_all_files();
        assert_eq!(files, vec!["src/cache.ts".to_string()]);
        let found = graph.find_symbol("cached");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_node_trait_unified_api() {
        let allocator = Allocator::default();
        let source = "const value = foo(1, bar);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let root = parsed.program.as_node(source);
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
    fn test_node_trait_upcast_and_downcast() {
        let allocator = Allocator::default();
        let source = "export function greet(name) { return name; }";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let root = parsed.program.as_node(source);
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
    fn test_identify_meta_variables_detects_single_and_multi_capture() {
        let allocator = Allocator::default();
        let source = "const value = fn($A, $$$B, $$$);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let root = parsed.program.as_node(source);
        let meta_vars = identify_meta_variables(&root);
        assert_eq!(meta_vars.len(), 3);

        let mut single = false;
        let mut multi = false;
        let mut wildcard = false;

        for kind in meta_vars.values() {
            match kind {
                PatternNodeKind::MetaVar(node) => {
                    if node.name == "A" {
                        single = true;
                    }
                }
                PatternNodeKind::MultiMetaVar(node) => {
                    if node.name == "B" {
                        multi = true;
                    }
                }
                PatternNodeKind::MultiWildcard => wildcard = true,
                PatternNodeKind::Node { .. } => {}
            }
        }

        assert!(single);
        assert!(multi);
        assert!(wildcard);
    }

    #[test]
    fn test_to_pattern_ast_marks_meta_variable_nodes() {
        let allocator = Allocator::default();
        let source = "const value = fn($A, $$$B);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let root = parsed.program.as_node(source);
        let pattern = to_pattern_ast(root);

        fn count_kind(node: &PatternNode, target: &PatternNodeKind) -> usize {
            let mut count = usize::from(&node.kind == target);
            for child in &node.children {
                count += count_kind(child, target);
            }
            count
        }

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
                    name: "B".to_string(),
                })
            ),
            1
        );
        assert_eq!(pattern.text, source);
    }

    #[test]
    fn test_pattern_matcher_captures_single_and_multi_meta_var() {
        let allocator = Allocator::default();
        let target_source = "const value = fn(answer, foo, bar);";
        let pattern_source = "const value = fn($A, $$$B);";

        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::default();
        let env = matcher
            .match_node(target, &pattern)
            .expect("pattern should match target");

        assert_eq!(env.single_captures.get("A").unwrap().text, "answer");
        assert_eq!(env.multi_captures.get("B").unwrap().len(), 2);
        assert_eq!(env.multi_captures.get("B").unwrap()[0].text, "foo");
        assert_eq!(env.multi_captures.get("B").unwrap()[1].text, "bar");
    }

    #[test]
    fn test_pattern_matcher_exposes_match_result_and_capture_queries() {
        let allocator = Allocator::default();
        let target_source = "const value = fn(answer, foo, bar);";
        let pattern_source = "const value = fn($A, $$$B);";

        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::default();
        let result = matcher.match_result(target, &pattern);

        assert!(result.is_match());
        assert!(result.environment().has_single_capture("A"));
        assert!(result.environment().has_multi_capture("B"));
        assert_eq!(
            result.environment().get_single_capture("A").unwrap().text,
            "answer"
        );
        assert_eq!(result.environment().get_multi_capture("B").unwrap().len(), 2);
    }

    #[test]
    fn test_pattern_matcher_requires_consistent_meta_var_binding() {
        let allocator = Allocator::default();
        let pattern_source = "const pair = fn($A, $A);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);
        let matcher = PatternMatcher::default();

        let matched_source = "const pair = fn(x, x);";
        let parsed_matched = Parser::new(&allocator, matched_source, source_type).parse();
        let matched_target = parsed_matched.program.as_node(matched_source);
        assert!(matcher.match_node(matched_target, &pattern).is_some());

        let unmatched_source = "const pair = fn(x, y);";
        let parsed_unmatched = Parser::new(&allocator, unmatched_source, source_type).parse();
        let unmatched_target = parsed_unmatched.program.as_node(unmatched_source);
        assert!(matcher.match_node(unmatched_target, &pattern).is_none());
    }

    #[test]
    fn test_pattern_matcher_requires_consistent_multi_meta_var_binding() {
        let allocator = Allocator::default();
        let pattern_source = "const value = fn($$$A, $$$A);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);
        let matcher = PatternMatcher::default();

        let matched_source = "const value = fn(x, y, x, y);";
        let parsed_matched = Parser::new(&allocator, matched_source, source_type).parse();
        let matched_target = parsed_matched.program.as_node(matched_source);
        assert!(matcher.match_node(matched_target, &pattern).is_some());

        let unmatched_source = "const value = fn(x, y, x, z);";
        let parsed_unmatched = Parser::new(&allocator, unmatched_source, source_type).parse();
        let unmatched_target = parsed_unmatched.program.as_node(unmatched_source);
        assert!(matcher.match_node(unmatched_target, &pattern).is_none());
    }

    #[test]
    fn test_pattern_matcher_rejects_mixed_single_and_multi_capture_name() {
        let allocator = Allocator::default();
        let target_source = "const value = fn(a, b);";
        let pattern_source = "const value = fn($A, $$$A);";

        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::default();
        assert!(matcher.match_node(target, &pattern).is_none());
    }

    #[test]
    fn test_pattern_matcher_relaxed_skips_whitespace_and_comments() {
        let allocator = Allocator::default();
        let target_source = "const /*comment*/ value = 1;";
        let pattern_source = "const value = 1;";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::new(MatchStrictness::Relaxed);
        assert!(matcher.match_node(target, &pattern).is_some());
    }

    #[test]
    fn test_pattern_matcher_ast_ignores_whitespace_and_comments() {
        let allocator = Allocator::default();
        let target_source = "const /*comment*/ value = 1;";
        let pattern_source = "const value = 1;";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::new(MatchStrictness::Ast);
        assert!(matcher.match_node(target, &pattern).is_some());
    }

    #[test]
    fn test_pattern_matcher_cst_requires_exact_text() {
        let allocator = Allocator::default();
        let target_source = "const /*comment*/ value = 1;";
        let pattern_source = "const value = 1;";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::new(MatchStrictness::Cst);
        assert!(matcher.match_node(target, &pattern).is_none());
    }

    #[test]
    fn test_pattern_matcher_signature_ignores_function_body() {
        let allocator = Allocator::default();
        let target_source = "function foo(a) { return a + 1; }";
        let pattern_source = "function foo(a) { return a + 2; }";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::new(MatchStrictness::Signature);
        assert!(matcher.match_node(target, &pattern).is_some());
    }

    #[test]
    fn test_pattern_matcher_template_supports_multi_wildcard() {
        let allocator = Allocator::default();
        let target_source = "const value = fn(a, b, c);";
        let pattern_source = "const value = fn($$$);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_root = parsed_pattern.program.as_node(pattern_source);
        let pattern = to_pattern_ast(pattern_root);

        let matcher = PatternMatcher::new(MatchStrictness::Template);
        assert!(matcher.match_node(target, &pattern).is_some());
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

        let mut composite = CompositeMatcher::new();
        composite.push(PatternMatcher::default());
        composite.push(PatternMatcher::default());

        let env = composite
            .match_node(target, &pattern)
            .expect("composite matcher should match and preserve captures");
        assert_eq!(env.single_captures.get("A").unwrap().text, "answer");
    }

    #[test]
    fn test_find_all_matches_prefers_outer_overlap() {
        let allocator = Allocator::default();
        let target_source = "const value = fn(fn(a));";
        let pattern_source = "const value = fn($$$A);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_program = parsed_pattern.program.as_node(pattern_source);
        let pattern_call = pattern_program.children()[0].children()[0].children()[0];
        let pattern = to_pattern_ast(pattern_call);

        let matcher = PatternMatcher::default();
        let results = find_all_matches(&matcher, target, &pattern, ConflictResolution::PreferOuter);

        assert_eq!(results.len(), 1);
        assert_eq!(&target_source[results[0].span.start as usize..results[0].span.end as usize], "fn(fn(a))");
        assert_eq!(results[0].environment.get_multi_capture("A").unwrap().len(), 1);
    }

    #[test]
    fn test_find_all_matches_prefers_inner_overlap() {
        let allocator = Allocator::default();
        let target_source = "const value = fn(fn(a));";
        let pattern_source = "const value = fn($$$A);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_program = parsed_pattern.program.as_node(pattern_source);
        let pattern_call = pattern_program.children()[0].children()[0].children()[0];
        let pattern = to_pattern_ast(pattern_call);

        let matcher = PatternMatcher::default();
        let results = find_all_matches(&matcher, target, &pattern, ConflictResolution::PreferInner);

        assert_eq!(results.len(), 1);
        assert_eq!(&target_source[results[0].span.start as usize..results[0].span.end as usize], "fn(a)");
    }

    #[test]
    fn test_find_all_matches_sorts_and_dedups_non_overlapping_matches() {
        let allocator = Allocator::default();
        let target_source = "const a = fn(1); const b = fn(2); const c = fn(3);";
        let pattern_source = "const value = fn($A);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();

        let parsed_target = Parser::new(&allocator, target_source, source_type).parse();
        let parsed_pattern = Parser::new(&allocator, pattern_source, source_type).parse();

        let target = parsed_target.program.as_node(target_source);
        let pattern_program = parsed_pattern.program.as_node(pattern_source);
        let pattern_call = pattern_program.children()[0].children()[0].children()[0];
        let pattern = to_pattern_ast(pattern_call);

        let matcher = PatternMatcher::default();
        let results = FindAllMatches(&matcher, target, &pattern, ConflictResolution::PreferOuter);

        assert_eq!(results.len(), 3);
        assert_eq!(&target_source[results[0].span.start as usize..results[0].span.end as usize], "fn(1)");
        assert_eq!(&target_source[results[1].span.start as usize..results[1].span.end as usize], "fn(2)");
        assert_eq!(&target_source[results[2].span.start as usize..results[2].span.end as usize], "fn(3)");
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
    }

    #[test]
    fn test_identify_meta_variables_rejects_non_conforming_names() {
        let allocator = Allocator::default();
        let source = "const value = fn($a, $$$b, $1, $_OK);";
        let source_type = SourceType::from_path(Path::new("inline.ts")).unwrap();
        let parsed = Parser::new(&allocator, source, source_type).parse();

        let root = parsed.program.as_node(source);
        let meta_vars = identify_meta_variables(&root);

        assert_eq!(meta_vars.len(), 1);
        assert!(meta_vars.values().any(|kind| {
            matches!(
                kind,
                PatternNodeKind::MetaVar(WildcardNode { name }) if name == "_OK"
            )
        }));
    }

    #[test]
    fn test_vue_sfc_extract_script_setup_and_offset_map() {
        let source = r#"<template>
  <div>{{ msg }}</div>
</template>
<script setup lang="ts">
const answer = 42;
</script>
<style scoped>
.app { color: red; }
</style>
"#;

        let extractor = VueSfcExtractor::new(source);
        let block = extractor.extract_script_block().unwrap();
        assert_eq!(block.kind, VueSfcScriptKind::ScriptSetup);
        assert!(block.content.contains("const answer = 42;"));
        assert!(block.block_presence.has_template);
        assert!(block.block_presence.has_style);
        assert!(block.block_presence.has_script_setup);

        let rel = block.content.find("answer").unwrap() as u32;
        let abs = block.offset_map.relative_to_absolute_offset(rel).unwrap();
        assert_eq!(&source[abs as usize..abs as usize + 6], "answer");
        assert_eq!(block.offset_map.absolute_to_relative_offset(abs), Some(rel));

        let rel_span = NodeSpan {
            start: rel,
            end: rel + 6,
        };
        let abs_span = block.offset_map.relative_to_absolute_span(rel_span).unwrap();
        assert_eq!(&source[abs_span.start as usize..abs_span.end as usize], "answer");
        assert_eq!(
            block.offset_map.absolute_to_relative_span(abs_span),
            Some(rel_span)
        );

        let expected = offset_to_line_col(abs as usize, &compute_line_starts(source));
        assert_eq!(block.offset_map.relative_offset_to_line_col(rel), Some(expected));
    }

    #[test]
    fn test_vue_sfc_extract_normal_script_and_presence() {
        let source = r#"<script lang="ts">
export const value = 1;
</script>
"#;

        let extractor = VueSfcExtractor::new(source);
        let block = extractor.extract_script_block().unwrap();
        assert_eq!(block.kind, VueSfcScriptKind::Script);
        assert!(block.content.contains("export const value = 1;"));

        let presence = extractor.block_presence();
        assert!(presence.has_script);
        assert!(!presence.has_script_setup);
        assert!(!presence.has_template);
        assert!(!presence.has_style);
    }

    #[test]
    fn test_vue_sfc_identify_template_style_without_script() {
        let source = r#"<template><div>Only template</div></template>
<style>.only { color: blue; }</style>
"#;

        let extractor = VueSfcExtractor::new(source);
        assert!(extractor.extract_script_block().is_none());

        let presence = extractor.block_presence();
        assert!(!presence.has_script);
        assert!(!presence.has_script_setup);
        assert!(presence.has_template);
        assert!(presence.has_style);
    }
}
