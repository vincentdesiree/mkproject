use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::expand_path;

/// Represents the supported license types for a generated project.
#[derive(Clone, Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum LicenseType {
    #[default]
    None,
    Mit,
    Apache,
    Dual, // MIT OR Apache-2.0
    Unlicense,
    Custom(String, PathBuf),
}

impl TryFrom<String> for LicenseType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl FromStr for LicenseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim();
        match input.to_lowercase().as_str() {
            "none" => Ok(LicenseType::None),
            "mit" => Ok(LicenseType::Mit),
            "apache" | "apache-2.0" => Ok(LicenseType::Apache),
            "dual" | "mit or apache-2.0" | "mit/apache" => Ok(LicenseType::Dual),
            "unlicense" => Ok(LicenseType::Unlicense),
            _ => {
                if let Some((name, path_str)) = input.split_once(':') {
                    Ok(LicenseType::Custom(
                        name.trim().to_string(),
                        PathBuf::from(path_str.trim()),
                    ))
                } else {
                    Err(format!(
                        "Invalid license '{}'. Expected: mit, apache, dual, unlicense, none, or 'name:path/to/file' for custom.",
                        s
                    ))
                }
            }
        }
    }
}

impl LicenseType {
    /// Returns the corresponding SPDX expression or name string.
    pub fn name(&self) -> Option<&str> {
        match self {
            LicenseType::Mit => Some("MIT"),
            LicenseType::Apache => Some("Apache-2.0"),
            LicenseType::Dual => Some("MIT OR Apache-2.0"),
            LicenseType::Unlicense => Some("Unlicense"),
            LicenseType::Custom(name, _) => Some(name.as_str()),
            LicenseType::None => None,
        }
    }

    /// Returns a list of target filenames and their embedded raw content templates.
    pub fn files_to_create(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            LicenseType::Mit => vec![("LICENSE-MIT", include_str!("../assets/LICENSE-MIT"))],
            LicenseType::Apache => {
                vec![("LICENSE-APACHE", include_str!("../assets/LICENSE-APACHE"))]
            }
            LicenseType::Dual => vec![
                ("LICENSE-MIT", include_str!("../assets/LICENSE-MIT")),
                ("LICENSE-APACHE", include_str!("../assets/LICENSE-APACHE")),
            ],
            LicenseType::Unlicense => {
                vec![("LICENSE", include_str!("../assets/UNLICENSE"))]
            }
            LicenseType::Custom(_, _) | LicenseType::None => vec![],
        }
    }

    //// Applies the selected license configuration to the generated project directory.
    pub fn apply(&self, project_dir: &Path, authors: &str, year: u16) -> Result<()> {
        let template_license = project_dir.join("LICENSE");
        if template_license.exists() {
            fs::remove_file(&template_license).with_context(|| {
                format!(
                    "Failed to remove template LICENSE at {:?}",
                    template_license
                )
            })?;
        }

        if let LicenseType::Custom(_, source_path) = self {
            let expanded_source = expand_path(source_path);
            if expanded_source.exists() {
                let raw_content = fs::read_to_string(&expanded_source).with_context(|| {
                    format!(
                        "Failed to read custom license file at {:?}",
                        expanded_source
                    )
                })?;

                let content = raw_content
                    .replace("{year}", &year.to_string())
                    .replace("{authors}", authors);

                let target_path = project_dir.join("LICENSE");
                fs::write(&target_path, content).with_context(|| {
                    format!("Failed to write custom license file at {:?}", target_path)
                })?;
            }
            return Ok(());
        }

        for (filename, raw_content) in self.files_to_create() {
            let target_path = project_dir.join(filename);
            let content = raw_content
                .replace("{year}", &year.to_string())
                .replace("{authors}", authors);

            fs::write(&target_path, content)
                .with_context(|| format!("Failed to write license file at {:?}", target_path))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_license_names() {
        assert_eq!(LicenseType::Mit.name(), Some("MIT"));
        assert_eq!(LicenseType::Apache.name(), Some("Apache-2.0"));
        assert_eq!(LicenseType::Dual.name(), Some("MIT OR Apache-2.0"));
        assert_eq!(LicenseType::Unlicense.name(), Some("Unlicense"));
        assert_eq!(
            LicenseType::Custom("GPL-3.0".into(), PathBuf::from("LICENSE")).name(),
            Some("GPL-3.0")
        );
        assert_eq!(LicenseType::None.name(), None);
    }

    #[test]
    fn test_from_str_parsing() {
        assert_eq!("mit".parse::<LicenseType>(), Ok(LicenseType::Mit));
        assert_eq!("apache-2.0".parse::<LicenseType>(), Ok(LicenseType::Apache));
        assert_eq!("dual".parse::<LicenseType>(), Ok(LicenseType::Dual));
        assert_eq!("none".parse::<LicenseType>(), Ok(LicenseType::None));
        assert_eq!(
            "proprietary:LICENSE.txt".parse::<LicenseType>(),
            Ok(LicenseType::Custom(
                "proprietary".into(),
                PathBuf::from("LICENSE.txt")
            ))
        );
        assert!("invalid_license".parse::<LicenseType>().is_err());
    }

    #[test]
    fn test_apply_custom_license() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let custom_src = dir.path().join("my_custom_license.txt");
        fs::write(&custom_src, "Copyright (c) {year} {authors}. Proprietary.")?;

        let license = LicenseType::Custom("Proprietary".into(), custom_src);
        license.apply(project_dir, "Vincent Désirée", 2026)?;

        let target_license = project_dir.join("LICENSE");
        assert!(target_license.exists());

        let content = fs::read_to_string(target_license)?;
        assert!(content.contains("2026"));
        assert!(content.contains("Vincent Désirée"));

        Ok(())
    }

    #[test]
    fn test_embedded_files_not_empty() {
        for license in [
            LicenseType::Mit,
            LicenseType::Apache,
            LicenseType::Dual,
            LicenseType::Unlicense,
        ] {
            let files = license.files_to_create();
            assert!(!files.is_empty(), "License files list should not be empty");
            for (filename, content) in files {
                assert!(!filename.is_empty());
                assert!(
                    !content.is_empty(),
                    "Embedded license text for {} should not be empty",
                    filename
                );
            }
        }
    }

    #[test]
    fn test_apply_dual_license_and_cleanup() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path();

        let dummy_license = project_dir.join("LICENSE");
        fs::write(&dummy_license, "Old Template License Content")?;

        let license = LicenseType::Dual;
        license.apply(project_dir, "Vincent Désirée", 2026)?;

        assert!(
            !dummy_license.exists(),
            "Old LICENSE should have been removed"
        );

        let mit_path = project_dir.join("LICENSE-MIT");
        let apache_path = project_dir.join("LICENSE-APACHE");

        assert!(mit_path.exists(), "LICENSE-MIT should exist");
        assert!(apache_path.exists(), "LICENSE-APACHE should exist");

        let mit_content = fs::read_to_string(mit_path)?;
        assert!(mit_content.contains("2026"), "Year should be interpolated");
        assert!(
            mit_content.contains("Vincent Désirée"),
            "Author should be interpolated"
        );

        Ok(())
    }
}
