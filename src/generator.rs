use crate::{license::LicenseType, project::ResolvedProject};
use anyhow::Result;
use chrono::Datelike;
use std::{
    fs,
    path::Path,
    process::{Command, exit},
};

/// Generates a new project using `cargo-generate` and applies custom license configuration.
pub fn generate_project(resolved: &ResolvedProject) {
    println!("🚀 Creating '{}' using cargo-generate...", resolved.name);

    let destination_dir = &resolved.target_path;

    if let Err(err) = fs::create_dir_all(destination_dir) {
        eprintln!(
            "❌ Failed to create destination directory {}: {err}",
            destination_dir.display()
        );
        exit(1);
    }

    let license_str = resolved.license.name().unwrap_or("");
    let current_year = chrono::Utc::now().year() as u16;

    let status = Command::new("cargo")
        .arg("generate")
        .arg("--git")
        .arg(&resolved.template_url)
        .arg("--name")
        .arg(&resolved.name)
        .arg("--destination")
        .arg(destination_dir)
        .arg("-d")
        .arg(format!("project_authors={}", resolved.authors))
        .arg("-d")
        .arg(format!("license={license_str}"))
        .arg("-d")
        .arg(format!("year={current_year}"))
        .status();

    match status {
        Ok(s) if s.success() => {
            let project_path = resolved.target_path.join(&resolved.name);
            if let Err(err) = resolved
                .license
                .apply(&project_path, &resolved.authors, current_year)
            {
                eprintln!("⚠️ Warning: Failed to apply license files: {err}");
            }

            if matches!(resolved.license, LicenseType::None) {
                if let Err(err) = remove_license_from_cargo_toml(&project_path) {
                    eprintln!("⚠️ Warning: Failed to clean up Cargo.toml: {err}");
                }
            }
        }
        Ok(s) => {
            eprintln!("❌ cargo-generate failed with exit code: {:?}", s.code());
            exit(1);
        }
        Err(err) => {
            eprintln!("❌ Failed to execute cargo-generate: {err}");
            eprintln!("👉 Please ensure it is installed: `cargo install cargo-generate`");
            exit(1);
        }
    }
}

/// Removes any `license = ...` entry from `Cargo.toml` to comply with Cargo standards when no license is selected.
fn remove_license_from_cargo_toml(project_dir: &Path) -> Result<()> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&cargo_toml_path)?;
    let cleaned_content: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim_start().starts_with("license ="))
        .collect();

    fs::write(&cargo_toml_path, cleaned_content.join("\n") + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_remove_license_from_cargo_toml() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let cargo_path = dir.path().join("Cargo.toml");

        let initial_toml = r#"[package]
name = "test_project"
version = "0.1.0"
authors = ["Vincent"]
license = "MIT"
edition = "2021"

[dependencies]
"#;

        fs::write(&cargo_path, initial_toml)?;

        remove_license_from_cargo_toml(dir.path())?;

        let cleaned_toml = fs::read_to_string(&cargo_path)?;

        assert!(!cleaned_toml.contains("license ="));
        assert!(cleaned_toml.contains("name = \"test_project\""));
        assert!(cleaned_toml.contains("authors = [\"Vincent\"]"));

        Ok(())
    }

    #[test]
    fn test_remove_license_when_file_missing() {
        let dir = tempdir().unwrap();
        assert!(remove_license_from_cargo_toml(dir.path()).is_ok());
    }
}
