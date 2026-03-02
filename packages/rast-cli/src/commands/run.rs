use std::{fs, path::Path};

use ast_engine::{apply_rule_to_source, parse_rule};

use crate::io::rule_loader::resolve_rule;
use crate::output::types::OutputFormat;

pub fn run(
    file: &Path,
    rule: &str,
    _output: OutputFormat,
    _verbose: bool,
) -> Result<String, String> {
    let source = fs::read_to_string(file).map_err(|e| format!("Failed to read file: {e}"))?;
    let resolved_rule = resolve_rule(rule)?;
    let parsed_rule = parse_rule(&resolved_rule)?;
    let (updated_source, _, _) = apply_rule_to_source(&source, &parsed_rule)?;
    Ok(updated_source)
}
