use std::path::{Path, PathBuf};

pub mod cli;
pub mod config;
pub mod generator;
pub mod license;
pub mod project;

/// Helper utility to expand `~` to the user's home directory.
pub fn expand_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str == "~" {
        dirs::home_dir().unwrap_or_else(|| path.to_path_buf())
    } else if let Some(stripped) = path_str.strip_prefix("~/") {
        dirs::home_dir()
            .map(|h| h.join(stripped))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path_tilde_only() {
        if let Some(home) = dirs::home_dir() {
            let expanded = expand_path(Path::new("~"));
            assert_eq!(expanded, home);
        }
    }

    #[test]
    fn test_expand_path_with_subfolder() {
        if let Some(home) = dirs::home_dir() {
            let expanded = expand_path(Path::new("~/dev/workspace"));
            assert_eq!(expanded, home.join("dev/workspace"));
        }
    }
}
