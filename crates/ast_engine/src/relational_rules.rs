use crate::node_trait::{AstNode, NodeTrait};
use crate::yaml_schema::Rule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRuleKind {
    Inside,
    Has,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalRule {
    pub kind: RelationalRuleKind,
    pub rule: Rule,
}

impl RelationalRule {
    pub fn new(kind: RelationalRuleKind, rule: Rule) -> Self {
        Self { kind, rule }
    }
}

pub fn evaluate_relational_rule<'a, F>(
    target: AstNode<'a>,
    ancestors: &[AstNode<'a>],
    rule: &RelationalRule,
    evaluate: F,
) -> bool
where
    F: Fn(AstNode<'a>, &Rule) -> bool,
{
    match rule.kind {
        RelationalRuleKind::Inside => {
            for ancestor in ancestors.iter().rev() {
                if evaluate(*ancestor, &rule.rule) {
                    return true;
                }
            }
            false
        }
        RelationalRuleKind::Has => {
            let mut stack = target.children();
            while let Some(node) = stack.pop() {
                if evaluate(node, &rule.rule) {
                    return true;
                }
                stack.extend(node.children());
            }
            false
        }
    }
}
