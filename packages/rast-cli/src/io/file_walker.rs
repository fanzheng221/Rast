use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn collect_files(dir: &Path, exts: &[String], out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {e}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, exts, out)?;
            continue;
        }

        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        if exts.iter().any(|allowed| allowed == ext) {
            out.push(path);
        }
    }

    Ok(())
}

pub fn parse_extensions(extensions: Option<String>) -> Vec<String> {
    extensions
        .map(|s| {
            s.split(',')
                .map(|e| e.trim().trim_start_matches('.').to_string())
                .filter(|e| !e.is_empty())
                .collect()
        })
        .unwrap_or_else(|| {
            vec![
                "js".to_string(),
                "ts".to_string(),
                "jsx".to_string(),
                "tsx".to_string(),
            ]
        })
}
