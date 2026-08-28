use crate::license::LicenseType;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "mkproject",
    version,
    about = "A light CLI wrapper to bootstrap Rust projects via Git templates"
)]
pub struct Cli {
    /// Project name (e.g., 'my_app') or combined 'type/name' (e.g., 'app/my_app')
    pub name: String,

    /// Optional project type defined in config (e.g., app, lib, tool)
    pub project_type: Option<String>,

    /// Override template repository URL
    #[arg(short = 't', long = "template")]
    pub template: Option<String>,

    /// Override target destination path
    #[arg(short = 'p', long = "path")]
    pub path: Option<PathBuf>,

    /// Author(s) name(s)
    #[arg(short = 'a', long = "authors")]
    pub authors: Option<String>,

    /// License SPDX or custom (ex: "MIT", "Apache-2.0", "MIT OR Apache-2.0", "Unlicense", "license_name:path_to_license_file")
    #[arg(short = 'l', long = "license")]
    pub license: Option<LicenseType>,

    /// Skip Git repository initialization
    #[arg(long = "no-git")]
    pub no_git: bool,
}

impl Cli {
    pub fn normalize(mut self) -> Self {
        if self.project_type.is_none() && self.name.contains('/') {
            if let Some((t, n)) = self.name.split_once('/') {
                self.project_type = Some(t.to_string());
                self.name = n.to_string();
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_normalize_type_and_name() {
        let cli = Cli {
            name: "tool/my_cli".to_string(),
            project_type: None,
            template: None,
            path: None,
            authors: None,
            license: None,
            no_git: false,
        }
        .normalize();

        assert_eq!(cli.project_type, Some("tool".to_string()));
        assert_eq!(cli.name, "my_cli");
    }
}
