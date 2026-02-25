//! Rast AST Engine
//!
//! Core AST analysis engine for JavaScript/TypeScript parsing (oxc-based).

use std::path::Path;

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
}
