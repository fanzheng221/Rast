use std::{fs, path::Path};

pub fn resolve_rule(rule: &str) -> Result<String, String> {
    let path = Path::new(rule);
    if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("Failed to read rule file: {e}"))
    } else {
        Ok(rule.to_string())
    }
}
