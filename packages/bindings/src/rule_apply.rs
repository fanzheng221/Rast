use std::fs;

use ast_engine::{apply_rule_to_source, parse_rule};
use napi_derive::napi;
use walkdir::WalkDir;

use crate::serde_payload::{to_json_string, to_napi_error, ScanResult};

#[napi(js_name = "apply_rule")]
pub fn apply_rule(source: String, yaml_rule: String) -> napi::Result<String> {
    let parsed_rule =
        parse_rule(&yaml_rule).map_err(|err| to_napi_error("Invalid YAML rule", err))?;

    if parsed_rule.fix_template.is_none() {
        return Err(napi::Error::from_reason(
            "Invalid YAML rule: missing top-level fix template",
        ));
    }

    let (updated_source, _matches, _modifications) = apply_rule_to_source(&source, &parsed_rule)
        .map_err(|err| to_napi_error("Failed to apply rule", err))?;
    Ok(updated_source)
}

#[napi(js_name = "scan_directory")]
pub fn scan_directory(root_path: String, yaml_rule: String, dry_run: bool) -> napi::Result<String> {
    let parsed_rule =
        parse_rule(&yaml_rule).map_err(|err| to_napi_error("Invalid YAML rule", err))?;
    if !dry_run && parsed_rule.fix_template.is_none() {
        return Err(napi::Error::from_reason(
            "Invalid YAML rule: missing top-level fix template",
        ));
    }

    let mut results = Vec::new();
    for entry in WalkDir::new(&root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let extension = entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        if !matches!(extension, "js" | "ts" | "jsx" | "tsx") {
            continue;
        }

        let path = entry.path();
        let original_source =
            fs::read_to_string(path).map_err(|err| to_napi_error("Failed to read file", err))?;
        let (updated_source, match_count, modifications) =
            apply_rule_to_source(&original_source, &parsed_rule)
                .map_err(|err| to_napi_error("Failed to apply rule", err))?;

        if !dry_run && modifications > 0 && updated_source != original_source {
            fs::write(path, updated_source)
                .map_err(|err| to_napi_error("Failed to write file", err))?;
        }

        results.push(ScanResult {
            path: path.to_string_lossy().to_string(),
            matches: match_count,
            modifications: if dry_run { None } else { Some(modifications) },
        });
    }

    Ok(to_json_string(&results))
}
