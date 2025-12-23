//! LSP server installer using the Mason registry.
//!
//! This module provides functionality to automatically download and install
//! LSP servers from the Mason registry.

mod package;
mod platform;

pub use package::{Package, PackageSource, SourceAsset};
pub use platform::{Arch, Os, Platform};

use crate::error::{RefactorError, Result};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const REGISTRY_BASE_URL: &str = "https://raw.githubusercontent.com/mason-org/mason-registry/main";
const DEFAULT_INSTALL_DIR: &str = "refactor-dsl/lsp-servers";

/// LSP server installer that downloads servers from the Mason registry.
pub struct LspInstaller {
    install_dir: PathBuf,
    cache_dir: PathBuf,
    platform: Platform,
}

impl LspInstaller {
    /// Creates a new installer with default settings.
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir().ok_or_else(|| {
            RefactorError::InvalidConfig("Cannot determine data directory".into())
        })?;

        let install_dir = data_dir.join(DEFAULT_INSTALL_DIR);
        let cache_dir = data_dir.join("refactor-dsl/cache");

        Ok(Self {
            install_dir,
            cache_dir,
            platform: Platform::detect(),
        })
    }

    /// Sets a custom installation directory.
    pub fn install_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.install_dir = path.into();
        self
    }

    /// Sets a custom cache directory.
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = path.into();
        self
    }

    /// Returns the installation directory.
    pub fn get_install_dir(&self) -> &Path {
        &self.install_dir
    }

    /// Checks if a server is already installed.
    pub fn is_installed(&self, server_name: &str) -> bool {
        let server_dir = self.install_dir.join(server_name);
        if !server_dir.exists() {
            return false;
        }

        // Check if the binary exists
        let bin_path = self.get_binary_path(server_name);
        bin_path.map(|p| p.exists()).unwrap_or(false)
    }

    /// Gets the path to an installed server's binary.
    pub fn get_binary_path(&self, server_name: &str) -> Option<PathBuf> {
        let server_dir = self.install_dir.join(server_name);
        let info_path = server_dir.join("info.json");

        if !info_path.exists() {
            return None;
        }

        let info: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&info_path).ok()?).ok()?;

        let bin_name = info.get("binary")?.as_str()?;
        Some(server_dir.join(bin_name))
    }

    /// Ensures a server is installed, downloading if necessary.
    pub fn ensure_installed(&self, server_name: &str) -> Result<PathBuf> {
        if self.is_installed(server_name) {
            return self.get_binary_path(server_name).ok_or_else(|| {
                RefactorError::TransformFailed {
                    message: format!("Server {} is installed but binary not found", server_name),
                }
            });
        }

        self.install(server_name)
    }

    /// Installs a server from the Mason registry.
    pub fn install(&self, server_name: &str) -> Result<PathBuf> {
        // Fetch package metadata
        let package = self.fetch_package(server_name)?;

        // Get the appropriate asset for this platform
        let asset = package
            .get_asset_for_platform(&self.platform)
            .ok_or_else(|| RefactorError::TransformFailed {
                message: format!(
                    "No binary available for {} on {:?}/{:?}",
                    server_name, self.platform.os, self.platform.arch
                ),
            })?;

        // Create installation directory
        let server_dir = self.install_dir.join(server_name);
        fs::create_dir_all(&server_dir)?;

        // Download and extract
        let download_url = package.get_download_url(&asset)?;
        let archive_path = self.download_file(&download_url, server_name, &asset.file)?;

        // Extract based on file type
        let bin_name = self.extract_archive(&archive_path, &server_dir, &asset)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let bin_path = server_dir.join(&bin_name);
            let mut perms = fs::metadata(&bin_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bin_path, perms)?;
        }

        // Save installation info
        let info = serde_json::json!({
            "name": server_name,
            "version": package.source.id,
            "binary": bin_name,
            "platform": format!("{:?}/{:?}", self.platform.os, self.platform.arch),
        });
        fs::write(
            server_dir.join("info.json"),
            serde_json::to_string_pretty(&info)?,
        )?;

        Ok(server_dir.join(bin_name))
    }

    /// Fetches package metadata from the registry.
    fn fetch_package(&self, server_name: &str) -> Result<Package> {
        let url = format!(
            "{}/packages/{}/package.yaml",
            REGISTRY_BASE_URL, server_name
        );

        let response =
            reqwest::blocking::get(&url).map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to fetch package {}: {}", server_name, e),
            })?;

        if !response.status().is_success() {
            return Err(RefactorError::TransformFailed {
                message: format!(
                    "Package {} not found in registry (status: {})",
                    server_name,
                    response.status()
                ),
            });
        }

        let yaml_content = response
            .text()
            .map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to read package data: {}", e),
            })?;

        serde_yaml::from_str(&yaml_content).map_err(|e| RefactorError::TransformFailed {
            message: format!("Failed to parse package YAML: {}", e),
        })
    }

    /// Downloads a file to the cache directory.
    fn download_file(&self, url: &str, server_name: &str, filename: &str) -> Result<PathBuf> {
        fs::create_dir_all(&self.cache_dir)?;

        let cache_path = self.cache_dir.join(format!("{}_{}", server_name, filename));

        // Skip download if already cached
        if cache_path.exists() {
            return Ok(cache_path);
        }

        let response = reqwest::blocking::get(url).map_err(|e| RefactorError::TransformFailed {
            message: format!("Failed to download {}: {}", url, e),
        })?;

        if !response.status().is_success() {
            return Err(RefactorError::TransformFailed {
                message: format!("Download failed with status: {}", response.status()),
            });
        }

        let bytes = response
            .bytes()
            .map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to read download: {}", e),
            })?;

        let mut file = File::create(&cache_path)?;
        file.write_all(&bytes)?;

        Ok(cache_path)
    }

    /// Extracts an archive to the target directory.
    fn extract_archive(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        asset: &SourceAsset,
    ) -> Result<String> {
        let filename = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if filename.ends_with(".gz") && !filename.ends_with(".tar.gz") {
            // Single gzipped file
            self.extract_gzip(archive_path, target_dir, asset)
        } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            // Tar gzipped archive
            self.extract_tar_gz(archive_path, target_dir, asset)
        } else if filename.ends_with(".zip") {
            // Zip archive
            self.extract_zip(archive_path, target_dir, asset)
        } else {
            // Assume it's a plain binary
            let bin_name = asset
                .bin
                .as_ref()
                .map(|b| b.to_string())
                .unwrap_or_else(|| filename.to_string());
            fs::copy(archive_path, target_dir.join(&bin_name))?;
            Ok(bin_name)
        }
    }

    /// Extracts a gzip-compressed single file.
    fn extract_gzip(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        asset: &SourceAsset,
    ) -> Result<String> {
        use flate2::read::GzDecoder;

        let file = File::open(archive_path)?;
        let mut decoder = GzDecoder::new(file);

        let bin_name = asset
            .bin
            .as_ref()
            .map(|b| b.to_string())
            .unwrap_or_else(|| {
                archive_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("server")
                    .to_string()
            });

        let output_path = target_dir.join(&bin_name);
        let mut output_file = File::create(&output_path)?;
        io::copy(&mut decoder, &mut output_file)?;

        Ok(bin_name)
    }

    /// Extracts a tar.gz archive.
    fn extract_tar_gz(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        asset: &SourceAsset,
    ) -> Result<String> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        archive
            .unpack(target_dir)
            .map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to extract tar.gz: {}", e),
            })?;

        let bin_name = asset
            .bin
            .as_ref()
            .map(|b| b.to_string())
            .unwrap_or_else(|| "server".to_string());

        Ok(bin_name)
    }

    /// Extracts a zip archive.
    fn extract_zip(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        asset: &SourceAsset,
    ) -> Result<String> {
        let file = File::open(archive_path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to open zip: {}", e),
            })?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| RefactorError::TransformFailed {
                    message: format!("Failed to read zip entry: {}", e),
                })?;

            let outpath = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }

        let bin_name = asset
            .bin
            .as_ref()
            .map(|b| b.to_string())
            .unwrap_or_else(|| "server".to_string());

        Ok(bin_name)
    }

    /// Lists all installed servers.
    pub fn list_installed(&self) -> Result<Vec<InstalledServer>> {
        let mut servers = Vec::new();

        if !self.install_dir.exists() {
            return Ok(servers);
        }

        for entry in fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let info_path = entry.path().join("info.json");
                if info_path.exists() {
                    if let Ok(info_str) = fs::read_to_string(&info_path) {
                        if let Ok(info) = serde_json::from_str::<serde_json::Value>(&info_str) {
                            servers.push(InstalledServer {
                                name: info
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                version: info
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                binary: entry.path().join(
                                    info.get("binary")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("server"),
                                ),
                            });
                        }
                    }
                }
            }
        }

        Ok(servers)
    }

    /// Uninstalls a server.
    pub fn uninstall(&self, server_name: &str) -> Result<()> {
        let server_dir = self.install_dir.join(server_name);
        if server_dir.exists() {
            fs::remove_dir_all(&server_dir)?;
        }
        Ok(())
    }
}

impl Default for LspInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create default installer")
    }
}

/// Information about an installed server.
#[derive(Debug, Clone)]
pub struct InstalledServer {
    /// Server name.
    pub name: String,
    /// Installed version.
    pub version: String,
    /// Path to the binary.
    pub binary: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation() {
        let installer = LspInstaller::new();
        assert!(installer.is_ok());
    }

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        // Should detect something
        assert!(matches!(platform.os, Os::Linux | Os::MacOS | Os::Windows));
    }

    #[test]
    fn test_custom_install_dir() {
        let installer = LspInstaller::new().unwrap().install_dir("/tmp/test-lsp");

        assert_eq!(installer.get_install_dir(), Path::new("/tmp/test-lsp"));
    }
}
