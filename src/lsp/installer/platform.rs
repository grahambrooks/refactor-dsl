//! Platform detection for LSP server installation.

use std::env;

/// Represents the current platform (OS + architecture).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    /// Operating system.
    pub os: Os,
    /// CPU architecture.
    pub arch: Arch,
}

/// Operating system types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    MacOS,
    Windows,
}

/// CPU architecture types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
    X86,
}

impl Platform {
    /// Detects the current platform.
    pub fn detect() -> Self {
        Self {
            os: Self::detect_os(),
            arch: Self::detect_arch(),
        }
    }

    /// Creates a platform with explicit OS and architecture.
    pub fn new(os: Os, arch: Arch) -> Self {
        Self { os, arch }
    }

    /// Detects the operating system.
    fn detect_os() -> Os {
        match env::consts::OS {
            "linux" => Os::Linux,
            "macos" => Os::MacOS,
            "windows" => Os::Windows,
            _ => {
                // Default to Linux for unknown Unix-like systems
                if cfg!(unix) {
                    Os::Linux
                } else {
                    Os::Windows
                }
            }
        }
    }

    /// Detects the CPU architecture.
    fn detect_arch() -> Arch {
        match env::consts::ARCH {
            "x86_64" | "amd64" => Arch::X64,
            "aarch64" | "arm64" => Arch::Arm64,
            "x86" | "i686" | "i386" => Arch::X86,
            _ => Arch::X64, // Default to x64
        }
    }

    /// Returns target patterns for matching assets.
    pub fn get_target_patterns(&self) -> Vec<String> {
        let os_patterns: Vec<&str> = match self.os {
            Os::Linux => vec!["linux", "unknown-linux"],
            Os::MacOS => vec!["darwin", "macos", "apple-darwin", "apple"],
            Os::Windows => vec!["windows", "win", "pc-windows"],
        };

        let arch_patterns: Vec<&str> = match self.arch {
            Arch::X64 => vec!["x86_64", "x64", "amd64"],
            Arch::Arm64 => vec!["aarch64", "arm64"],
            Arch::X86 => vec!["i686", "x86", "i386"],
        };

        let mut patterns = Vec::new();

        // Add combined patterns (most specific first)
        for os in &os_patterns {
            for arch in &arch_patterns {
                patterns.push(format!("{}_{}", os, arch));
                patterns.push(format!("{}-{}", arch, os));
                patterns.push(format!("{}-unknown-{}", arch, os));
            }
        }

        // Add OS-only patterns as fallback
        for os in &os_patterns {
            patterns.push(os.to_string());
        }

        patterns
    }

    /// Returns the Rust target triple for this platform.
    pub fn target_triple(&self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Linux, Arch::X64) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Arm64) => "aarch64-unknown-linux-gnu",
            (Os::Linux, Arch::X86) => "i686-unknown-linux-gnu",
            (Os::MacOS, Arch::X64) => "x86_64-apple-darwin",
            (Os::MacOS, Arch::Arm64) => "aarch64-apple-darwin",
            (Os::MacOS, Arch::X86) => "i686-apple-darwin",
            (Os::Windows, Arch::X64) => "x86_64-pc-windows-msvc",
            (Os::Windows, Arch::Arm64) => "aarch64-pc-windows-msvc",
            (Os::Windows, Arch::X86) => "i686-pc-windows-msvc",
        }
    }

    /// Returns the file extension for executables on this platform.
    pub fn exe_extension(&self) -> &'static str {
        match self.os {
            Os::Windows => ".exe",
            _ => "",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}/{:?}", self.os, self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        // Should detect valid OS
        assert!(matches!(platform.os, Os::Linux | Os::MacOS | Os::Windows));
        // Should detect valid arch
        assert!(matches!(platform.arch, Arch::X64 | Arch::Arm64 | Arch::X86));
    }

    #[test]
    fn test_target_patterns() {
        let linux_x64 = Platform::new(Os::Linux, Arch::X64);
        let patterns = linux_x64.get_target_patterns();

        assert!(patterns.iter().any(|p| p.contains("linux")));
        assert!(patterns.iter().any(|p| p.contains("x86_64") || p.contains("x64")));
    }

    #[test]
    fn test_target_triple() {
        assert_eq!(
            Platform::new(Os::Linux, Arch::X64).target_triple(),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            Platform::new(Os::MacOS, Arch::Arm64).target_triple(),
            "aarch64-apple-darwin"
        );
        assert_eq!(
            Platform::new(Os::Windows, Arch::X64).target_triple(),
            "x86_64-pc-windows-msvc"
        );
    }

    #[test]
    fn test_exe_extension() {
        assert_eq!(Platform::new(Os::Windows, Arch::X64).exe_extension(), ".exe");
        assert_eq!(Platform::new(Os::Linux, Arch::X64).exe_extension(), "");
        assert_eq!(Platform::new(Os::MacOS, Arch::Arm64).exe_extension(), "");
    }

    #[test]
    fn test_display() {
        let platform = Platform::new(Os::Linux, Arch::X64);
        assert_eq!(format!("{}", platform), "Linux/X64");
    }
}
