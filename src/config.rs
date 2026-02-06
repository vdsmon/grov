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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_config_missing_file_returns_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = read_config(tmp.path());
        assert_eq!(config.worktree.prefix, "");
    }

    #[test]
    fn read_config_invalid_toml_returns_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".grov.toml"), "not valid {{ toml").unwrap();
        let config = read_config(tmp.path());
        assert_eq!(config.worktree.prefix, "");
    }

    #[test]
    fn write_then_read_roundtrip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = GrovConfig {
            worktree: WorktreeConfig {
                prefix: "mp".to_string(),
            },
        };
        write_config(tmp.path(), &config).unwrap();
        let loaded = read_config(tmp.path());
        assert_eq!(loaded.worktree.prefix, "mp");
    }

    #[test]
    fn write_then_read_empty_prefix() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = GrovConfig {
            worktree: WorktreeConfig {
                prefix: "".to_string(),
            },
        };
        write_config(tmp.path(), &config).unwrap();
        let loaded = read_config(tmp.path());
        assert_eq!(loaded.worktree.prefix, "");
    }
}
