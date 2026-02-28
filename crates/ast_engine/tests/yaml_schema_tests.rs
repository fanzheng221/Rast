use ast_engine::yaml_schema::{
    KindAtomicRule, PatternAtomicRule, RegexAtomicRule, RuleCore, RuleKind, RuleLanguage,
};

#[test]
fn test_parse_pattern_rule() {
    let yaml = r#"
id: test-pattern
language: js
rule:
  pattern: console.log($A)
"#;
    let rule_core = RuleCore::from_yaml(yaml).expect("Failed to parse pattern rule");
    assert_eq!(rule_core.id, "test-pattern");
    assert_eq!(rule_core.language, RuleLanguage::Js);

    match rule_core.rule.core {
        RuleKind::Pattern(PatternAtomicRule { pattern }) => {
            assert_eq!(pattern, "console.log($A)");
        }
        _ => panic!("Expected Pattern rule"),
    }
}

#[test]
fn test_parse_regex_rule() {
    let yaml = r#"
id: test-regex
language: ts
rule:
  regex: "foo.*bar"
"#;
    let rule_core = RuleCore::from_yaml(yaml).expect("Failed to parse regex rule");
    assert_eq!(rule_core.id, "test-regex");
    assert_eq!(rule_core.language, RuleLanguage::Ts);

    match rule_core.rule.core {
        RuleKind::Regex(RegexAtomicRule { regex }) => {
            assert_eq!(regex, "foo.*bar");
        }
        _ => panic!("Expected Regex rule"),
    }
}

#[test]
fn test_parse_kind_rule() {
    let yaml = r#"
id: test-kind
language: tsx
rule:
  kind: class_declaration
"#;
    let rule_core = RuleCore::from_yaml(yaml).expect("Failed to parse kind rule");
    assert_eq!(rule_core.id, "test-kind");
    assert_eq!(rule_core.language, RuleLanguage::Tsx);

    match rule_core.rule.core {
        RuleKind::Kind(KindAtomicRule { kind }) => {
            assert_eq!(kind, "class_declaration");
        }
        _ => panic!("Expected Kind rule"),
    }
}

#[test]
fn test_parse_all_composite_rule() {
    let yaml = r#"
id: test-all
language: javascript
rule:
  all:
    - pattern: foo()
    - pattern: bar()
"#;
    let rule_core = RuleCore::from_yaml(yaml).expect("Failed to parse all rule");
    assert_eq!(rule_core.id, "test-all");
    assert_eq!(rule_core.language, RuleLanguage::Javascript);

    match rule_core.rule.core {
        RuleKind::All(all_rule) => {
            assert_eq!(all_rule.all.len(), 2);
            match &all_rule.all[0].core {
                RuleKind::Pattern(p) => assert_eq!(p.pattern, "foo()"),
                _ => panic!("Expected pattern"),
            }
            match &all_rule.all[1].core {
                RuleKind::Pattern(p) => assert_eq!(p.pattern, "bar()"),
                _ => panic!("Expected pattern"),
            }
        }
        _ => panic!("Expected All rule"),
    }
}

#[test]
fn test_parse_any_composite_rule() {
    let yaml = r#"
id: test-any
language: typescript
rule:
  any:
    - kind: identifier
    - regex: "^foo"
"#;
    let rule_core = RuleCore::from_yaml(yaml).expect("Failed to parse any rule");
    assert_eq!(rule_core.id, "test-any");
    assert_eq!(rule_core.language, RuleLanguage::Typescript);

    match rule_core.rule.core {
        RuleKind::Any(any_rule) => {
            assert_eq!(any_rule.any.len(), 2);
            match &any_rule.any[0].core {
                RuleKind::Kind(k) => assert_eq!(k.kind, "identifier"),
                _ => panic!("Expected kind"),
            }
            match &any_rule.any[1].core {
                RuleKind::Regex(r) => assert_eq!(r.regex, "^foo"),
                _ => panic!("Expected regex"),
            }
        }
        _ => panic!("Expected Any rule"),
    }
}

#[test]
fn test_parse_not_composite_rule() {
    let yaml = r#"
id: test-not
language: jsx
rule:
  not:
    pattern: console.log($A)
"#;
    let rule_core = RuleCore::from_yaml(yaml).expect("Failed to parse not rule");
    assert_eq!(rule_core.id, "test-not");
    assert_eq!(rule_core.language, RuleLanguage::Jsx);

    match rule_core.rule.core {
        RuleKind::Not(not_rule) => match &not_rule.not.core {
            RuleKind::Pattern(p) => assert_eq!(p.pattern, "console.log($A)"),
            _ => panic!("Expected pattern"),
        },
        _ => panic!("Expected Not rule"),
    }
}

#[test]
fn test_invalid_mixed_rule() {
    let yaml = r#"
id: test-invalid
language: js
rule:
  pattern: foo()
  regex: bar
"#;
    let result = RuleCore::from_yaml(yaml);
    assert!(result.is_err(), "Should fail when mixing pattern and regex");
}

#[test]
fn test_invalid_language() {
    let yaml = r#"
id: test-invalid-lang
language: python
rule:
  pattern: foo()
"#;
    let result = RuleCore::from_yaml(yaml);
    assert!(result.is_err(), "Should fail with invalid language");
}
