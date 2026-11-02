#!/usr/bin/env bash
#
# Bumps the version, publishes to crates.io and pushes the release tag.
#
#   ./scripts/release.sh 0.3.1
#
# Nothing leaves this machine before the confirmation prompt. If a later step
# fails, run the same command again: steps that are already done are skipped.
# When the Release build has finished, run scripts/update-formula.sh.

set -euo pipefail

CRATE="open_texture_packer"
READMES=(README.md README.ja.md)
VERSION_FILES=(Cargo.toml Cargo.lock "${READMES[@]}")

die() {
    echo "error: $*" >&2
    exit 1
}

confirm() {
    local answer=""
    read -r -p "$1 [y/N] " answer || true
    [ "$answer" = "y" ] || die "aborted"
}

is_published() {
    local code
    code=$(curl -sS -o /dev/null -w '%{http_code}' -A "$CRATE release script" \
        "https://crates.io/api/v1/crates/$CRATE/$VERSION") || die "could not reach crates.io"
    case "$code" in
        200) return 0 ;;
        404) return 1 ;;
        *) die "unexpected response from crates.io: HTTP $code" ;;
    esac
}

push_tag() {
    if ! git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
        git tag "$TAG"
    fi
    git push origin "refs/tags/$TAG"
}

# Puts the version files back when the script stops before committing them.
bump_pending=0
restore_version_files() {
    if [ "$bump_pending" = 1 ]; then
        git checkout -- "${VERSION_FILES[@]}"
        echo "Restored ${VERSION_FILES[*]}." >&2
    fi
}
trap restore_version_files EXIT

[ $# -eq 1 ] || die "usage: $0 <version>   (example: $0 0.3.1)"
VERSION="$1"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "version must look like 1.2.3: $VERSION"
TAG="v$VERSION"

cd "$(dirname "$0")/.."

# --- Checks ---------------------------------------------------------------

[ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] || die "not on the main branch"
[ -z "$(git status --porcelain)" ] || die "the working tree is not clean:
$(git status --short)"

git fetch --quiet --tags origin
git merge-base --is-ancestor origin/main HEAD || die "local main is behind origin/main"

current=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
[ -n "$current" ] || die "could not read the version from Cargo.toml"

tag_on_remote=0
if git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null; then
    tag_on_remote=1
fi
tag_local=0
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    tag_local=1
fi

published=0
if is_published; then
    published=1
fi

if [ "$published" = 1 ] && [ "$tag_on_remote" = 1 ]; then
    echo "$TAG is already released. Next: ./scripts/update-formula.sh $VERSION"
    exit 0
fi

if [ "$published" = 0 ] && { [ "$tag_on_remote" = 1 ] || [ "$tag_local" = 1 ]; }; then
    die "tag $TAG exists but $VERSION is not on crates.io. Check the tag before retrying."
fi

if [ "$published" = 1 ]; then
    [ "$current" = "$VERSION" ] || die "$VERSION is on crates.io, but Cargo.toml says $current"
    echo "$VERSION is already on crates.io. Only the tag is missing."
    confirm "Push tag $TAG at $(git rev-parse --short HEAD)?"
    push_tag
    echo "Done. When the Release build has finished: ./scripts/update-formula.sh $VERSION"
    exit 0
fi

# --- Version bump ---------------------------------------------------------

bumped=0
if [ "$current" != "$VERSION" ]; then
    newest=$(printf '%s\n%s\n' "$current" "$VERSION" | sort -V | tail -n 1)
    [ "$newest" = "$VERSION" ] || die "$VERSION is older than the current version $current"
    [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] \
        || die "local main has commits that are not on origin/main"

    bump_pending=1
    bumped=1
    sed -i.bak "s/^version = \"$current\"\$/version = \"$VERSION\"/" Cargo.toml
    rm -f Cargo.toml.bak
    for readme in "${READMES[@]}"; do
        sed -i.bak "s/^$CRATE = \".*\"\$/$CRATE = \"$VERSION\"/" "$readme"
        rm -f "$readme.bak"
    done
fi

# --- Verification ---------------------------------------------------------

cargo test

if [ "$bumped" = 1 ]; then
    # The tree was clean above, so the version files are the only changes.
    cargo publish --dry-run --allow-dirty

    grep -q "^version = \"$VERSION\"\$" Cargo.toml || die "Cargo.toml was not updated"
    for readme in "${READMES[@]}"; do
        grep -q "^$CRATE = \"$VERSION\"\$" "$readme" || die "$readme was not updated"
    done
    grep -A 1 "^name = \"$CRATE\"\$" Cargo.lock | grep -q "^version = \"$VERSION\"\$" \
        || die "Cargo.lock was not updated"

    git --no-pager diff -- "${VERSION_FILES[@]}"
else
    cargo publish --dry-run
fi

# --- Release --------------------------------------------------------------

echo
echo "About to release $VERSION: push main, publish to crates.io, push $TAG."
echo "A version published to crates.io can never be replaced or deleted."
confirm "Continue?"

if [ "$bumped" = 1 ]; then
    git add "${VERSION_FILES[@]}"
    git commit -m "feat: 🎸 $TAG"
    bump_pending=0
fi

git push origin main
cargo publish
push_tag

echo
echo "Done. When the Release build has finished: ./scripts/update-formula.sh $VERSION"
