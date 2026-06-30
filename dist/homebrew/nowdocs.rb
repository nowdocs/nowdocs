# typed: false
# frozen_string_literal: true

# Homebrew formula for nowdocs — local MCP server, single self-contained binary.
# Unsigned by design (D9): avoids F-1/OPT visa risk associated with commercial
# code-signing certificates; matches community norms for CLI tools (ripgrep, fd,
# uv, ruff). Users verify authenticity via the sha256 pinned in this formula
# and the GitHub release tag.
#
# Release asset layout (per Task 5a binstall matrix + OQ-3 unified format):
#   URL:  https://github.com/nowdocs/nowdocs/releases/download/v{version}/
#         nowdocs-v{version}-{target}.tar.gz
#   Archive internal layout:
#         nowdocs-v{version}-{target}/
#         └── nowdocs       (the binary)
#
# Update flow on each release:
#   1. CI builds 5 targets, uploads assets to GitHub Release.
#   2. After release, CI (or maintainer) computes sha256 for the two macOS
#      assets and opens a PR against the tap repo (nowdocs-registry/homebrew-nowdocs
#      — see TAP_SETUP.md) that bumps `version` and the two `sha256` lines.
#   3. `brew upgrade nowdocs` then picks up the new version.

class Nowdocs < Formula
  desc "Local MCP server: latest third-party dev docs, hybrid search, zero API cost"
  homepage "https://nowdocs.rs"
  license "MIT OR Apache-2.0"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/nowdocs/nowdocs/releases/download/v#{version}/nowdocs-v#{version}-aarch64-apple-darwin.tar.gz"
      # TODO_FILL_AFTER_RELEASE: replace with `shasum -a 256` of the asset after
      # the matching GitHub Release is published. Bumped by release CI.
      sha256 "TODO_FILL_AFTER_RELEASE"
    end

    on_intel do
      url "https://github.com/nowdocs/nowdocs/releases/download/v#{version}/nowdocs-v#{version}-x86_64-apple-darwin.tar.gz"
      # TODO_FILL_AFTER_RELEASE: same workflow as the arm64 asset above.
      sha256 "TODO_FILL_AFTER_RELEASE"
    end
  end

  # NOTE: Linux + Windows targets are intentionally NOT covered by this Homebrew
  # formula. Homebrew is a macOS-first distribution channel. Linux users get
  # the same binaries via cargo-binstall (Task 5a) and Windows users via the
  # standalone tgz on the GitHub Release page.

  def install
    # Archive root is nowdocs-v{version}-{target}/, so CWD in def install is
    # that directory and the binary is right at ./nowdocs.
    bin.install "nowdocs"
  end

  test do
    # Smoke test: binary should respond to --version and exit 0. No external
    # network or filesystem state required.
    assert_match version.to_s, shell_output("#{bin}/nowdocs --version")
  end
end
