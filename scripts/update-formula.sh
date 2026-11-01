#!/usr/bin/env bash
#
# Points the Homebrew formula at a released version. Waits for the macOS
# binaries of the GitHub Release, then asks before pushing to the tap.
#
#   ./scripts/update-formula.sh 0.3.1
#
# Safe to run again: it does nothing if the formula is already up to date.
#
# TIMEOUT_SECONDS and POLL_SECONDS change how long it waits for the binaries.

set -euo pipefail

RELEASES="https://github.com/webcyou-org/open-texture-packer/releases/download"
TAP_REPO="git@github.com:webcyou-org/homebrew-tap.git"
FORMULA="Formula/open-texture-packer.rb"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-1800}"
POLL_SECONDS="${POLL_SECONDS:-30}"

die() {
    echo "error: $*" >&2
    exit 1
}

[ $# -eq 1 ] || die "usage: $0 <version>   (example: $0 0.3.1)"
VERSION="$1"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "version must look like 1.2.3: $VERSION"
TAG="v$VERSION"

cd "$(dirname "$0")/.."
GIT_NAME=$(git config user.name) || die "git user.name is not set"
GIT_EMAIL=$(git config user.email) || die "git user.email is not set"

WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT

asset_url() {
    echo "$RELEASES/$TAG/otp-$TAG-$1.tar.gz"
}

# Prints the sha256 of a release binary, waiting until it has been uploaded.
asset_sha256() {
    local target="$1"
    local url code
    local file="$WORK_DIR/$target.tar.gz"
    local waited=0
    url=$(asset_url "$target")

    while :; do
        code=$(curl -sSL -o "$file" -w '%{http_code}' "$url") || die "could not download $url"
        case "$code" in
            200) break ;;
            404) ;;
            *) die "unexpected response for $url: HTTP $code" ;;
        esac

        if [ "$waited" -ge "$TIMEOUT_SECONDS" ]; then
            die "$url did not appear within ${TIMEOUT_SECONDS}s. Check the Release workflow."
        fi
        echo "Waiting for otp-$TAG-$target.tar.gz ..." >&2
        sleep "$POLL_SECONDS"
        waited=$((waited + POLL_SECONDS))
    done

    tar -tzf "$file" | grep -q '/otp$' || die "$url does not contain the otp binary"
    shasum -a 256 "$file" | cut -d ' ' -f 1
}

ARM_SHA256=$(asset_sha256 aarch64-apple-darwin)
INTEL_SHA256=$(asset_sha256 x86_64-apple-darwin)

git clone --quiet "$TAP_REPO" "$WORK_DIR/tap"
cd "$WORK_DIR/tap"

cat > "$FORMULA" <<EOF
class OpenTexturePacker < Formula
  desc "Open source and free Texture Packer tool written in rust."
  homepage "https://github.com/webcyou-org/open-texture-packer"
  version "$VERSION"
  license "MIT"

  on_arm do
    url "$(asset_url aarch64-apple-darwin)"
    sha256 "$ARM_SHA256"
  end

  on_intel do
    url "$(asset_url x86_64-apple-darwin)"
    sha256 "$INTEL_SHA256"
  end

  def install
    bin.install "otp"
  end

  test do
    system "#{bin}/otp", "--help"
  end
end
EOF

ruby -c "$FORMULA" >/dev/null || die "the generated formula has a syntax error"

if [ -z "$(git status --porcelain)" ]; then
    echo "The formula is already at $VERSION."
    exit 0
fi

git --no-pager diff

answer=""
read -r -p "Push this formula to $TAP_REPO? [y/N] " answer || true
[ "$answer" = "y" ] || die "aborted"

git add "$FORMULA"
git -c user.name="$GIT_NAME" -c user.email="$GIT_EMAIL" \
    commit --quiet -m "feat: 🎸 open-texture-packer $TAG"
git push --quiet origin HEAD

echo
echo "Done. To install it: brew update && brew upgrade open-texture-packer"
