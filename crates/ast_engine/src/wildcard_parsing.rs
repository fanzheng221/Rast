use std::collections::HashMap;

use oxc::ast::ast::Expression;
use serde::{Deserialize, Serialize};

use crate::{AstNode, NodeSpan, NodeTrait};

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

pub fn is_valid_meta_capture_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_uppercase() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

pub fn wildcard_kind_from_identifier(name: &str) -> Option<PatternNodeKind> {
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

fn ast_node_to_pattern(
    node: AstNode<'_>,
    meta_variables: &HashMap<(u32, u32), PatternNodeKind>,
) -> PatternNode {
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
