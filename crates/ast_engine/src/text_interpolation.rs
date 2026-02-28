//! Text interpolation for fix template patterns.

use crate::MatchEnvironment;

/// Represents a text template that may contain metavariable placeholders.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateFix {
    /// Pure text replacement without metavariables
    Text(String),
    /// Template with metavariable placeholders
    Template {
        fragments: Vec<TemplateFragment>,
    },
}

/// A fragment of a template - either literal text or a metavariable reference.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateFragment {
    /// Literal text string
    Literal(String),
    /// Single metavariable reference ($A)
    SingleMetaVar(String),
    /// Multiple metavariable reference ($$$A)
    MultiMetaVar(String),
}

impl TemplateFix {
    /// Parse a template string into a TemplateFix.
    pub fn from(template: &str) -> Self {
        if !template.contains('$') {
            return TemplateFix::Text(template.to_string());
        }

        let mut fragments = Vec::new();
        let mut current_literal_start = 0;
        let mut pos = 0;

        while pos < template.len() {
            // Check for $$$ first (multi metavariable)
            if template[pos..].starts_with("$$$") && pos + 3 < template.len() {
                let remaining = &template[pos + 3..];
                if !remaining.is_empty() {
                    let first = remaining.chars().next().unwrap();
                    if first.is_ascii_uppercase() || first == '_' {
                        let name_end = remaining
                            .chars()
                            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        if name_end > 0 {
                            if current_literal_start < pos {
                                fragments.push(TemplateFragment::Literal(template[current_literal_start..pos].to_string()));
                            }
                            fragments.push(TemplateFragment::MultiMetaVar(remaining[..name_end].to_string()));
                            pos = pos + 3 + name_end;
                            current_literal_start = pos;
                            continue;
                        }
                    }
                }
            }
            
            // Check for $ (single metavariable)
            if template[pos..].starts_with('$') && pos + 1 < template.len() {
                let remaining = &template[pos + 1..];
                if !remaining.is_empty() {
                    let first = remaining.chars().next().unwrap();
                    if first.is_ascii_uppercase() || first == '_' {
                        let name_end = remaining
                            .chars()
                            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        if name_end > 0 {
                            if current_literal_start < pos {
                                fragments.push(TemplateFragment::Literal(template[current_literal_start..pos].to_string()));
                            }
                            fragments.push(TemplateFragment::SingleMetaVar(remaining[..name_end].to_string()));
                            pos = pos + 1 + name_end;
                            current_literal_start = pos;
                            continue;
                        }
                    }
                }
            }
            
            // If no metavariable was found at current pos, treat current char as literal
            pos += 1;
        }

        // Add any remaining literal text
        if current_literal_start < template.len() {
            fragments.push(TemplateFragment::Literal(template[current_literal_start..].to_string()));
        }

        if fragments.is_empty() {
            TemplateFix::Text(template.to_string())
        } else {
            TemplateFix::Template { fragments }
        }
    }
}

/// Generate replacement text by substituting metavariables in the template.
pub fn generate_replacement(template: &TemplateFix, env: &MatchEnvironment) -> String {
    match template {
        TemplateFix::Text(text) => text.clone(),
        TemplateFix::Template { fragments } => {
            let mut result = String::new();

            for fragment in fragments {
                match fragment {
                    TemplateFragment::Literal(text) => {
                        result.push_str(text);
                    }
                    TemplateFragment::SingleMetaVar(name) => {
                        if let Some(captured) = env.get_single_capture(name) {
                            result.push_str(&captured.text);
                        } else if let Some(captured_nodes) = env.get_multi_capture(name) {
                            let texts: Vec<&str> = captured_nodes.iter().map(|n| n.text.as_str()).collect();
                            result.push_str(&texts.join(""));
                        }
                        else {
                            // Unmatched metavariable - leave as-is
                            result.push_str(&format!("${}", name));
                        }
                    }
                    TemplateFragment::MultiMetaVar(name) => {
                        if let Some(captured_nodes) = env.get_multi_capture(name) {
                            let texts: Vec<&str> = captured_nodes.iter().map(|n| n.text.as_str()).collect();
                            result.push_str(&texts.join(""));
                        }
                        else {
                            // Unmatched metavariable - leave as-is
                            result.push_str(&format!("$$${}", name));
                        }
                    }
                }
            }

            result
        }
    }
}
