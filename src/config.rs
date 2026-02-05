use std::path::Path;

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = ".grov.toml";

#[derive(Default, Serialize, Deserialize)]
pub struct GrovConfig {
    #[serde(default)]
    pub worktree: WorktreeConfig,
}

#[derive(Default, Serialize, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default)]
    pub prefix: String,
}

/// Read `.grov.toml` from the bare repo directory.
/// Returns `Default` if the file doesn't exist.
pub fn read_config(bare_repo: &Path) -> GrovConfig {
    let path = bare_repo.join(CONFIG_FILE);
    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => GrovConfig::default(),
    }
}

/// Write `.grov.toml` into the bare repo directory.
pub fn write_config(bare_repo: &Path, config: &GrovConfig) -> anyhow::Result<()> {
    let path = bare_repo.join(CONFIG_FILE);
    let contents = toml::to_string_pretty(config)?;
    std::fs::write(path, contents)?;
    Ok(())
}
