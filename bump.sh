#!/usr/bin/env bash
# Bump project version, commit, tag, and optionally push.
#
# Usage:
#   ./bump.sh <version>       # e.g. ./bump.sh 4.0.5
#   ./bump.sh <version> -p    # also git push + push tag
#   ./bump.sh -p <version>
#   ./bump.sh -h
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

PUSH=false
VERSION=""

usage() {
  cat <<'EOF'
Usage: ./bump.sh <version> [-p]

Arguments:
  version   Semantic version (e.g. 4.0.5 or v4.0.5)
  -p        Push commit and tag to origin after local commit/tag
  -h        Show this help

Examples:
  ./bump.sh 4.0.5
  ./bump.sh 4.0.5 -p
  ./bump.sh -p v4.0.5
EOF
}

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
    -p|--push)
      PUSH=true
      ;;
    -*)
      echo "error: unknown option: $arg" >&2
      usage >&2
      exit 1
      ;;
    *)
      if [[ -n "$VERSION" ]]; then
        echo "error: unexpected argument: $arg" >&2
        usage >&2
        exit 1
      fi
      VERSION="$arg"
      ;;
  esac
done

if [[ -z "$VERSION" ]]; then
  echo "error: version is required" >&2
  usage >&2
  exit 1
fi

# Normalize: strip leading v for file versions; keep v-prefixed tag
VERSION="${VERSION#v}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  echo "error: invalid version format: $VERSION (expected x.y.z)" >&2
  exit 1
fi

TAG="v${VERSION}"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: not a git repository" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is not clean; commit or stash changes first" >&2
  git status --short
  exit 1
fi

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "error: tag already exists: $TAG" >&2
  exit 1
fi

CURRENT="$(node -e "console.log(require('./package.json').version)" 2>/dev/null || true)"
if [[ -z "$CURRENT" ]]; then
  CURRENT="$(grep -m1 '"version"' package.json | sed -E 's/.*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/')"
fi

if [[ "$CURRENT" == "$VERSION" ]]; then
  echo "error: already at version $VERSION" >&2
  exit 1
fi

echo "==> Bumping version: ${CURRENT:-unknown} -> ${VERSION}"

# package.json
if command -v node >/dev/null 2>&1; then
  node -e '
    const fs = require("fs");
    const path = process.argv[1];
    const version = process.argv[2];
    const pkg = JSON.parse(fs.readFileSync(path, "utf8"));
    pkg.version = version;
    fs.writeFileSync(path, JSON.stringify(pkg, null, 2) + "\n");
  ' package.json "$VERSION"
else
  perl -i -0pe 's/("name"\s*:\s*"fetch-github-hosts"\s*,\s*"version"\s*:\s*")[^"]+"/${1}'"$VERSION"'"/' package.json
fi

# package-lock.json (root package only, avoid reformatting whole lockfile)
if [[ -f package-lock.json ]]; then
  perl -i -0pe '
    s/("name"\s*:\s*"fetch-github-hosts"\s*,\s*\n\s*"version"\s*:\s*")[^"]+"/${1}'"$VERSION"'"/g;
    s/(""\s*:\s*\{\s*\n\s*"name"\s*:\s*"fetch-github-hosts"\s*,\s*\n\s*"version"\s*:\s*")[^"]+"/${1}'"$VERSION"'"/;
  ' package-lock.json
fi

# src-tauri/Cargo.toml — package version only
perl -i -0pe 's/(\[package\][^\[]*?^version\s*=\s*")[^"]+"/${1}'"$VERSION"'"/ms' src-tauri/Cargo.toml

# src-tauri/Cargo.lock — this crate only
if [[ -f src-tauri/Cargo.lock ]]; then
  perl -i -0pe 's/(name\s*=\s*"fetch-github-hosts"\s*\nversion\s*=\s*")[^"]+"/${1}'"$VERSION"'"/' src-tauri/Cargo.lock
fi

# src-tauri/tauri.conf.json
if command -v node >/dev/null 2>&1; then
  node -e '
    const fs = require("fs");
    const path = process.argv[1];
    const version = process.argv[2];
    const conf = JSON.parse(fs.readFileSync(path, "utf8"));
    conf.version = version;
    fs.writeFileSync(path, JSON.stringify(conf, null, 2) + "\n");
  ' src-tauri/tauri.conf.json "$VERSION"
else
  perl -i -0pe 's/("productName"\s*:\s*"[^"]*"\s*,\s*\n\s*"mainBinaryName"\s*:\s*"[^"]*"\s*,\s*\n\s*"version"\s*:\s*")[^"]+"/${1}'"$VERSION"'"/' src-tauri/tauri.conf.json
fi

echo "==> Updated files:"
git diff --stat

git add \
  package.json \
  package-lock.json \
  src-tauri/Cargo.toml \
  src-tauri/Cargo.lock \
  src-tauri/tauri.conf.json

if git diff --cached --quiet; then
  echo "error: no version changes staged (files may already match $VERSION)" >&2
  exit 1
fi

# Non-interactive: never open $EDITOR / commit template UI
export GIT_EDITOR=true
export GIT_SEQUENCE_EDITOR=true
export GIT_PAGER=cat
export PAGER=cat

git -c core.editor=true commit -F - <<EOF
chore: bump version to ${VERSION}
EOF

# -F - reads tag message from stdin; avoids editor even if -m is ignored by config/hooks
git -c core.editor=true tag -a "$TAG" -F - <<EOF
Release ${TAG}
EOF

echo "==> Created commit and tag ${TAG}"

if [[ "$PUSH" == true ]]; then
  BRANCH="$(git rev-parse --abbrev-ref HEAD)"
  echo "==> Pushing ${BRANCH} and tag ${TAG} to origin"
  git push origin "$BRANCH"
  git push origin "$TAG"
  echo "==> Done (pushed)"
else
  echo "==> Done (local only)"
  echo "    Push later with: git push origin HEAD && git push origin ${TAG}"
  echo "    Or re-run with -p after resolving any issues"
fi
