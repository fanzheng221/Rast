use std::{fs, path::Path};

use ast_engine::{apply_rule_to_source, parse_rule};
use serde::Serialize;

use crate::io::{
    file_walker::{collect_files, parse_extensions},
    rule_loader::resolve_rule,
};
use crate::output::types::OutputFormat;

pub fn scan(
    dir: &Path,
    rule: &str,
    dry_run: bool,
    _output: OutputFormat,
    extensions: Option<String>,
    _verbose: bool,
) -> Result<String, String> {
    let resolved_rule = resolve_rule(rule)?;
    let parsed_rule = parse_rule(&resolved_rule)?;
    let exts = parse_extensions(extensions);
    let mut files = Vec::new();
    collect_files(dir, &exts, &mut files)?;

    let mut results = Vec::new();
    for path in files {
        let original_source = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file {}: {e}", path.display()))?;
        let (updated_source, match_count, modifications) =
            apply_rule_to_source(&original_source, &parsed_rule)?;

        if !dry_run && modifications > 0 && updated_source != original_source {
            fs::write(&path, updated_source)
                .map_err(|e| format!("Failed to write file {}: {e}", path.display()))?;
        }

        results.push(ScanResult {
            path: path.to_string_lossy().to_string(),
            matches: match_count,
            modifications: if dry_run { None } else { Some(modifications) },
        });
    }

    serde_json::to_string(&results).map_err(|e| format!("Failed to serialize scan result: {e}"))
}

#[derive(Serialize)]
struct ScanResult {
    path: String,
    matches: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    modifications: Option<usize>,
}
