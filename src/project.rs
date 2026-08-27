use crate::cli::Cli;
use crate::config::Config;
use crate::expand_path;
use crate::license::LicenseType;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedProject {
    pub name: String,
    pub target_path: PathBuf,
    pub template_url: String,
    pub authors: String,
    pub license: LicenseType,
}

impl ResolvedProject {
    /// Resolves project parameters by merging CLI arguments, configuration settings, and fallbacks.
    pub fn resolve(cli: Cli, config: Config) -> Self {
        let cli = cli.normalize();

        let template_url = cli
            .template
            .or_else(|| {
                cli.project_type
                    .as_ref()
                    .and_then(|t| config.types.get(t))
                    .and_then(|pt| pt.template.clone())
            })
            .unwrap_or(config.default_template.clone());

        let target_path = if let Some(custom_path) = cli.path {
            expand_path(&custom_path)
        } else {
            let base = config.workspace_path();

            if let Some(pt) = cli.project_type.as_ref().and_then(|t| config.types.get(t)) {
                base.join(&pt.path)
            } else {
                base
            }
        };

        let authors = cli
            .authors
            .or(config.default_authors)
            .unwrap_or_else(|| get_git_user_name().unwrap_or_else(|| "Unknown".to_string()));

        let license = cli
            .license
            .or(config.default_license)
            .unwrap_or(LicenseType::None);

        Self {
            name: cli.name,
            target_path,
            template_url,
            authors,
            license,
        }
    }
}

/// Retrieves the global or local Git user name if configured.
fn get_git_user_name() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::config::{Config, ProjectType};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn mock_config() -> Config {
        let mut types = HashMap::new();
        types.insert(
            "app".to_string(),
            ProjectType {
                path: "apps".to_string(),
                template: Some("https://github.com/user/app-template.git".to_string()),
            },
        );

        Config {
            default_template: "https://github.com/user/default-template.git".to_string(),
            workspace_root: PathBuf::from("/tmp/workspace"),
            default_authors: Some("Default Author".to_string()),
            default_license: Some(LicenseType::Mit),
            types,
        }
    }

    fn helper_cli(name: &str) -> Cli {
        Cli {
            name: name.to_string(),
            project_type: None,
            template: None,
            path: None,
            authors: None,
            license: None,
        }
    }

    #[test]
    fn test_resolve_combined_type_and_name() {
        let cli = helper_cli("app/my_app");
        let resolved = ResolvedProject::resolve(cli, mock_config());

        assert_eq!(resolved.name, "my_app");
        assert_eq!(
            resolved.target_path.join(&resolved.name),
            PathBuf::from("/tmp/workspace/apps/my_app")
        );
        assert_eq!(
            resolved.template_url,
            "https://github.com/user/app-template.git"
        );
        assert_eq!(resolved.authors, "Default Author");
        assert_eq!(resolved.license, LicenseType::Mit);
    }

    #[test]
    fn test_resolve_explicit_path_override() {
        let cli = Cli {
            path: Some(PathBuf::from("/custom/path/custom_proj")),
            ..helper_cli("custom_proj")
        };

        let resolved = ResolvedProject::resolve(cli, mock_config());

        assert_eq!(
            resolved.target_path,
            PathBuf::from("/custom/path/custom_proj")
        );
        assert_eq!(
            resolved.template_url,
            "https://github.com/user/default-template.git"
        );
    }

    #[test]
    fn test_resolve_explicit_path_tilde_expansion() {
        let cli = Cli {
            path: Some(PathBuf::from("~/custom_proj")),
            ..helper_cli("custom_proj")
        };

        let resolved = ResolvedProject::resolve(cli, mock_config());

        assert!(!resolved.target_path.starts_with("~"));
        assert!(resolved.target_path.ends_with("custom_proj"));
    }

    #[test]
    fn test_resolve_cli_overrides_config() {
        let cli = Cli {
            authors: Some("Vincent".to_string()),
            license: Some(LicenseType::Unlicense),
            ..helper_cli("custom_proj")
        };

        let resolved = ResolvedProject::resolve(cli, mock_config());

        assert_eq!(resolved.authors, "Vincent");
        assert_eq!(resolved.license, LicenseType::Unlicense);
    }
}
