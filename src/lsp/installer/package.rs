//! Package definition types for the Mason registry.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::platform::{Platform, Os, Arch};
use crate::error::{RefactorError, Result};

/// A package definition from the Mason registry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    /// Package name.
    pub name: String,
    /// Package description.
    #[serde(default)]
    pub description: String,
    /// Homepage URL.
    #[serde(default)]
    pub homepage: String,
    /// License identifiers.
    #[serde(default)]
    pub licenses: Vec<String>,
    /// Languages this server supports.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Categories (e.g., "LSP", "DAP", "Linter").
    #[serde(default)]
    pub categories: Vec<String>,
    /// Source information for downloading.
    pub source: PackageSource,
    /// Binary mappings.
    #[serde(default)]
    pub bin: HashMap<String, String>,
}

/// Source information for a package.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageSource {
    /// Source identifier (e.g., "pkg:github/rust-lang/rust-analyzer@2024-01-01").
    pub id: String,
    /// Platform-specific assets.
    #[serde(default)]
    pub asset: Vec<SourceAsset>,
}

/// A downloadable asset for a specific platform.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceAsset {
    /// Target platform (e.g., "linux_x64", "darwin_arm64").
    #[serde(default)]
    pub target: String,
    /// Filename to download.
    #[serde(default)]
    pub file: String,
    /// Binary name after extraction (if different from archive name).
    #[serde(default)]
    pub bin: Option<String>,
}

impl Package {
    /// Gets the asset matching the given platform.
    pub fn get_asset_for_platform(&self, platform: &Platform) -> Option<&SourceAsset> {
        let target_patterns = platform.get_target_patterns();

        for pattern in target_patterns {
            for asset in &self.source.asset {
                if asset.target.contains(&pattern) || pattern.contains(&asset.target) {
                    return Some(asset);
                }
            }
        }

        // Try more flexible matching
        for asset in &self.source.asset {
            if self.asset_matches_platform(&asset.target, platform) {
                return Some(asset);
            }
        }

        None
    }

    /// Checks if an asset target matches the platform.
    fn asset_matches_platform(&self, target: &str, platform: &Platform) -> bool {
        let target_lower = target.to_lowercase();

        let os_match = match platform.os {
            Os::Linux => target_lower.contains("linux"),
            Os::MacOS => target_lower.contains("darwin") || target_lower.contains("macos") || target_lower.contains("apple"),
            Os::Windows => target_lower.contains("windows") || target_lower.contains("win"),
        };

        let arch_match = match platform.arch {
            Arch::X64 => target_lower.contains("x86_64") || target_lower.contains("x64") || target_lower.contains("amd64"),
            Arch::Arm64 => target_lower.contains("aarch64") || target_lower.contains("arm64"),
            Arch::X86 => target_lower.contains("i686") || target_lower.contains("x86") && !target_lower.contains("x86_64"),
        };

        os_match && arch_match
    }

    /// Constructs the download URL for an asset.
    pub fn get_download_url(&self, asset: &SourceAsset) -> Result<String> {
        // Parse the source ID to determine the download URL
        let id = &self.source.id;

        if id.starts_with("pkg:github/") {
            self.github_download_url(id, asset)
        } else if id.starts_with("pkg:npm/") {
            Err(RefactorError::TransformFailed {
                message: format!("npm packages require npm to install: {}", id),
            })
        } else if id.starts_with("pkg:pypi/") {
            Err(RefactorError::TransformFailed {
                message: format!("PyPI packages require pip to install: {}", id),
            })
        } else if id.starts_with("pkg:cargo/") {
            Err(RefactorError::TransformFailed {
                message: format!("Cargo packages require cargo to install: {}", id),
            })
        } else {
            Err(RefactorError::TransformFailed {
                message: format!("Unknown package source type: {}", id),
            })
        }
    }

    /// Constructs a GitHub release download URL.
    fn github_download_url(&self, id: &str, asset: &SourceAsset) -> Result<String> {
        // Format: pkg:github/owner/repo@version
        let without_prefix = id.strip_prefix("pkg:github/")
            .ok_or_else(|| RefactorError::TransformFailed {
                message: "Invalid GitHub package ID".into(),
            })?;

        let parts: Vec<&str> = without_prefix.split('@').collect();
        if parts.len() != 2 {
            return Err(RefactorError::TransformFailed {
                message: format!("Invalid GitHub package ID format: {}", id),
            });
        }

        let repo_path = parts[0]; // e.g., "rust-lang/rust-analyzer"
        let version = parts[1];   // e.g., "2024-01-01"

        // GitHub release URL format
        Ok(format!(
            "https://github.com/{}/releases/download/{}/{}",
            repo_path, version, asset.file
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_yaml() {
        let yaml = r#"
name: rust-analyzer
description: A Rust compiler front-end for IDEs
homepage: https://github.com/rust-lang/rust-analyzer
licenses:
  - Apache-2.0
  - MIT
languages:
  - Rust
categories:
  - LSP
source:
  id: pkg:github/rust-lang/rust-analyzer@2024-01-01
  asset:
    - target: linux_x64_gnu
      file: rust-analyzer-x86_64-unknown-linux-gnu.gz
      bin: rust-analyzer
    - target: darwin_x64
      file: rust-analyzer-x86_64-apple-darwin.gz
      bin: rust-analyzer
    - target: darwin_arm64
      file: rust-analyzer-aarch64-apple-darwin.gz
      bin: rust-analyzer
    - target: win_x64
      file: rust-analyzer-x86_64-pc-windows-msvc.zip
      bin: rust-analyzer.exe
bin:
  rust-analyzer: rust-analyzer
"#;

        let package: Package = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(package.name, "rust-analyzer");
        assert_eq!(package.source.asset.len(), 4);
    }

    #[test]
    fn test_github_download_url() {
        let package = Package {
            name: "test".into(),
            description: "".into(),
            homepage: "".into(),
            licenses: vec![],
            languages: vec![],
            categories: vec![],
            source: PackageSource {
                id: "pkg:github/rust-lang/rust-analyzer@2024-01-01".into(),
                asset: vec![],
            },
            bin: HashMap::new(),
        };

        let asset = SourceAsset {
            target: "linux_x64".into(),
            file: "rust-analyzer-x86_64-unknown-linux-gnu.gz".into(),
            bin: Some("rust-analyzer".into()),
        };

        let url = package.get_download_url(&asset).unwrap();
        assert_eq!(
            url,
            "https://github.com/rust-lang/rust-analyzer/releases/download/2024-01-01/rust-analyzer-x86_64-unknown-linux-gnu.gz"
        );
    }

    #[test]
    fn test_asset_platform_matching() {
        let package = Package {
            name: "test".into(),
            description: "".into(),
            homepage: "".into(),
            licenses: vec![],
            languages: vec![],
            categories: vec![],
            source: PackageSource {
                id: "pkg:github/test/test@v1".into(),
                asset: vec![
                    SourceAsset {
                        target: "linux_x64_gnu".into(),
                        file: "linux.gz".into(),
                        bin: None,
                    },
                    SourceAsset {
                        target: "darwin_arm64".into(),
                        file: "macos-arm.gz".into(),
                        bin: None,
                    },
                ],
            },
            bin: HashMap::new(),
        };

        let linux_x64 = Platform { os: Os::Linux, arch: Arch::X64 };
        let macos_arm = Platform { os: Os::MacOS, arch: Arch::Arm64 };

        assert!(package.get_asset_for_platform(&linux_x64).is_some());
        assert!(package.get_asset_for_platform(&macos_arm).is_some());
    }
}
