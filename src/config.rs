use crate::{expand_path, license::LicenseType};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub default_template: String,
    pub workspace_root: PathBuf,
    pub default_authors: Option<String>,
    pub default_license: Option<LicenseType>,
    pub types: HashMap<String, ProjectType>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectType {
    pub path: String,
    pub template: Option<String>,
}

impl Config {
    /// Loads the configuration from disk, creating a default one if missing.
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if !config_path.exists() {
            Self::create_default_config(&config_path);
        }

        let content = fs::read_to_string(&config_path).unwrap_or_else(|err| {
            eprintln!(
                "❌ Failed to read config file {}: {err}",
                config_path.display()
            );
            exit(1);
        });

        toml::from_str(&content).unwrap_or_else(|err| {
            eprintln!("❌ Invalid TOML in {}: {err}", config_path.display());
            exit(1);
        })
    }

    /// Resolves the absolute workspace path, expanding the home directory if needed.
    pub fn workspace_path(&self) -> PathBuf {
        expand_path(&self.workspace_root)
    }

    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| {
            eprintln!("❌ Failed to locate system config directory.");
            exit(1);
        });

        config_dir.join("mkproject").join("config.toml")
    }

    fn create_default_config(path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let default_content = r#"workspace_root = "~/dev/workspace"
default_template = "https://github.com/vincentdesiree/rust_project_template.git"

default_authors = "Your name"
default_license = "None"

[types.project]
path = "projects"
#template = "template_repo_url"
"#;

        if let Err(err) = fs::write(path, default_content) {
            eprintln!(
                "❌ Failed to create config file at {}: {err}",
                path.display()
            );
            exit(1);
        }

        println!("⚙️ Default config created at: {}", path.display());
        println!("👉 Please update it with your settings and re-run.");
        exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_path_expansion() {
        let config = Config {
            default_template: "default".to_string(),
            workspace_root: PathBuf::from("~/dev/workspace"),
            default_authors: None,
            default_license: None,
            types: HashMap::new(),
        };

        let resolved_path = config.workspace_path();

        assert!(!resolved_path.starts_with("~"));
        assert!(resolved_path.ends_with("dev/workspace"));
    }
}
