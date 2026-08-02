class RefactorDsl < Formula
  desc "A DSL for multi-language code refactoring with Git-aware matching"
  homepage "https://github.com/grahambrooks/refactor-dsl"
  version "2026.7.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/grahambrooks/refactor-dsl/archive/refs/tags/v2026.8.1.tar.gz"
      sha256 "1d9874509954480601a14c047bef5c6bab8130ecbfb19785f2d049244e754867"
    end
    on_intel do
      odie "Intel Mac binaries are not provided. Run `cargo install --git https://github.com/grahambrooks/refactor-dsl --locked` to build from source."
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/grahambrooks/refactor-dsl/releases/download/v2026.7.0/refactor-v2026.7.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "1da9bf07b7cb5ce7d17537509221f918f0ef903d4f4bc6ffa98a68f743f796fb"
    end
    on_intel do
      url "https://github.com/grahambrooks/refactor-dsl/releases/download/v2026.7.0/refactor-v2026.7.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "11a6d5465c058058a5bc2b51665044d4ad685052cada88195023fa177f8b1afb"
    end
  end

  def install
    bin.install "refactor"
  end

  test do
    assert_path_exists bin/"refactor"
  end
end
