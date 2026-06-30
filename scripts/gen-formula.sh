#!/bin/sh
# Generate the Homebrew formula for a published release.
#
# Usage:
#   scripts/gen-formula.sh v0.1.1 > Formula/lazyglab.rb
#
# Downloads the release tarballs from GitHub, computes their SHA-256, and
# prints a multi-platform binary formula to stdout. Used both locally and by
# the release workflow to keep the tap up to date.

set -eu

VERSION="${1:?usage: gen-formula.sh <vX.Y.Z>}"
ver="${VERSION#v}"
repo="ragamo/lazyglab"
base="https://github.com/$repo/releases/download/$VERSION"

sha() {
  curl -fsSL "$base/lazyglab-$1.tar.gz" \
    | { sha256sum 2>/dev/null || shasum -a 256; } \
    | awk '{print $1}'
}

MAC_ARM="$(sha aarch64-apple-darwin)"
MAC_X86="$(sha x86_64-apple-darwin)"
LIN_ARM="$(sha aarch64-unknown-linux-gnu)"
LIN_X86="$(sha x86_64-unknown-linux-gnu)"

cat <<EOF
class Lazyglab < Formula
  desc "TUI for GitLab and GitHub — merge requests and pipelines in your terminal"
  homepage "https://github.com/$repo"
  version "$ver"
  license "MIT"

  on_macos do
    on_arm do
      url "$base/lazyglab-aarch64-apple-darwin.tar.gz"
      sha256 "$MAC_ARM"
    end
    on_intel do
      url "$base/lazyglab-x86_64-apple-darwin.tar.gz"
      sha256 "$MAC_X86"
    end
  end

  on_linux do
    on_arm do
      url "$base/lazyglab-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "$LIN_ARM"
    end
    on_intel do
      url "$base/lazyglab-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "$LIN_X86"
    end
  end

  def install
    bin.install "lazyglab"
  end

  test do
    assert_match "lazyglab #{version}", shell_output("#{bin}/lazyglab --version")
  end
end
EOF
