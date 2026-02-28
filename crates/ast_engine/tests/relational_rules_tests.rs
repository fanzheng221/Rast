use ast_engine::{
    evaluate_relational_rule, AstNode, IntoAstNode, NodeTrait, RelationalRule, RelationalRuleKind, Rule,
    RuleKind, KindAtomicRule,
};
use oxc::{
    allocator::Allocator,
    parser::Parser,
    span::SourceType,
};

fn parse_js<'a>(allocator: &'a Allocator, source: &'a str) -> oxc::ast::ast::Program<'a> {
    let source_type = SourceType::default();
    let ret = Parser::new(allocator, source, source_type).parse();
    ret.program
}

#[test]
fn test_evaluate_relational_rule_inside() {
    let allocator = Allocator::default();
    let source = "let a = console.log(1);";
    let program = parse_js(&allocator, source);
    let root = program.as_node(source);


    let mut ancestors = vec![root];
    
    let var_decl_node = root.children().into_iter().next().unwrap();
    ancestors.push(var_decl_node);
    
    fn find_call_expr<'a>(
        node: AstNode<'a>,
        current_ancestors: &mut Vec<AstNode<'a>>,
        target: &mut Option<(AstNode<'a>, Vec<AstNode<'a>>)>,
    ) {
        if node.kind() == "CallExpression" {
            *target = Some((node, current_ancestors.clone()));
            return;
        }
        current_ancestors.push(node);
        for child in node.children() {
            find_call_expr(child, current_ancestors, target);
        }
        current_ancestors.pop();
    }
    
    let mut target_info = None;
    find_call_expr(root, &mut vec![], &mut target_info);
    
    let (target, ancestors) = target_info.unwrap();
    
    let rule = RelationalRule::new(
        RelationalRuleKind::Inside,
        Rule {
            core: RuleKind::Kind(KindAtomicRule {
                kind: "VariableDeclaration".to_string(),
            }),
        },
    );
    
    let evaluate = |node: AstNode<'_>, rule: &Rule| -> bool {
        if let RuleKind::Kind(kind_rule) = &rule.core {
            node.kind() == kind_rule.kind
        } else {
            false
        }
    };
    
    let result = evaluate_relational_rule(target, &ancestors, &rule, evaluate);
    assert!(result, "CallExpression should be inside VariableDeclaration");
    
    let rule_fail = RelationalRule::new(
        RelationalRuleKind::Inside,
        Rule {
            core: RuleKind::Kind(KindAtomicRule {
                kind: "ClassDeclaration".to_string(),
            }),
        },
    );
    
    let result_fail = evaluate_relational_rule(target, &ancestors, &rule_fail, evaluate);
    assert!(!result_fail, "CallExpression should not be inside ClassDeclaration");
}

#[test]
fn test_evaluate_relational_rule_has() {
    let allocator = Allocator::default();
    let source = "let a = console.log(1);";
    let program = parse_js(&allocator, source);
    let root = program.as_node(source);

    let mut var_decl_node = None;
    for child in root.children() {
        if child.kind() == "VariableDeclaration" {
            var_decl_node = Some(child);
            break;
        }
    }
    let target = var_decl_node.unwrap();
    
    let rule = RelationalRule::new(
        RelationalRuleKind::Has,
        Rule {
            core: RuleKind::Kind(KindAtomicRule {
                kind: "CallExpression".to_string(),
            }),
        },
    );
    
    let evaluate = |node: AstNode<'_>, rule: &Rule| -> bool {
        if let RuleKind::Kind(kind_rule) = &rule.core {
            node.kind() == kind_rule.kind
        } else {
            false
        }
    };
    
    let result = evaluate_relational_rule(target, &[], &rule, evaluate);
    assert!(result, "VariableDeclaration should have CallExpression");
    
    let rule_fail = RelationalRule::new(
        RelationalRuleKind::Has,
        Rule {
            core: RuleKind::Kind(KindAtomicRule {
                kind: "ClassDeclaration".to_string(),
            }),
        },
    );
    
    let result_fail = evaluate_relational_rule(target, &[], &rule_fail, evaluate);
    assert!(!result_fail, "VariableDeclaration should not have ClassDeclaration");
}
