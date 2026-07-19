#!/usr/bin/env python3
"""Render a Homebrew formula for a grahambrooks Rust CLI.

Project metadata comes from the environment (loaded from .release.env):
    GITHUB_REPOSITORY   e.g. grahambrooks/forge
    BIN_NAME            installed binary name, e.g. forge
    FORMULA_NAME        formula file/name, e.g. forge
    FORMULA_CLASS       Ruby class name, e.g. Forge
    FORMULA_DESC        one-line description
    FORMULA_LICENSE     SPDX id (default: MIT)

Version + platform SHA256s come from argv:
    render-formula.py VERSION SHA_DARWIN_ARM SHA_LINUX_ARM SHA_LINUX_INTEL

Intel macOS is intentionally NOT distributed as a binary; those users build
from source with `cargo install`. Output is the formula on stdout.
"""
from __future__ import annotations

import os
import sys

TEMPLATE = """\
class {cls} < Formula
  desc "{desc}"
  homepage "https://github.com/{repo}"
  version "{version}"
  license "{license}"

  on_macos do
    on_arm do
      url "https://github.com/{repo}/releases/download/v{version}/{bin}-v{version}-aarch64-apple-darwin.tar.gz"
      sha256 "{sha_darwin_arm}"
    end
    on_intel do
      odie "Intel Mac binaries are not provided. Run `cargo install --git https://github.com/{repo} --locked` to build from source."
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/{repo}/releases/download/v{version}/{bin}-v{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "{sha_linux_arm}"
    end
    on_intel do
      url "https://github.com/{repo}/releases/download/v{version}/{bin}-v{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "{sha_linux_intel}"
    end
  end

  def install
    bin.install "{bin}"
  end

  test do
    assert_path_exists bin/"{bin}"
  end
end
"""


def main(argv: list[str]) -> int:
    if len(argv) != 5:
        print(__doc__, file=sys.stderr)
        return 2
    env = os.environ
    try:
        repo = env["GITHUB_REPOSITORY"]
        binn = env["BIN_NAME"]
        cls = env["FORMULA_CLASS"]
        desc = env["FORMULA_DESC"]
    except KeyError as exc:
        print(f"render-formula: missing required env var {exc}", file=sys.stderr)
        return 2
    license_id = env.get("FORMULA_LICENSE", "MIT")

    sys.stdout.write(
        TEMPLATE.format(
            cls=cls,
            desc=desc.replace('"', '\\"'),
            repo=repo,
            version=argv[1],
            license=license_id,
            bin=binn,
            sha_darwin_arm=argv[2],
            sha_linux_arm=argv[3],
            sha_linux_intel=argv[4],
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
