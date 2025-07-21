use std::path::Path;
use std::fs;

pub fn get_files_from_config(config_path: &Path) -> Result<Vec<String>, String> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read .gwtconfig: {e}"))?;
    Ok(content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}
