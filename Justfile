# Show the available recipes by default.
default:
  @just --list

# Format all Rust sources.
fmt:
  cargo fmt --all

# Run the test suite.
test:
  cargo test

# Run the common local checks before a release.
check:
  cargo fmt --all --check
  cargo test

# Print the current package version from Cargo.toml.
version:
  @sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1

# Prepare a release locally:
# - bump Cargo.toml to the target version
# - refresh Cargo.lock
# - run fmt, tests, and a crates.io dry-run
# - create a release commit and a local annotated tag
# This does not publish to crates.io and does not push to GitHub.
prepare-release version:
  #!/usr/bin/env bash
  set -euo pipefail

  version="{{version}}"
  version="${version#v}"

  if [[ -z "${version}" ]]; then
    echo "version is required, for example: just prepare-release 0.1.1" >&2
    exit 1
  fi

  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "working tree has unstaged or staged changes; commit or stash them before releasing" >&2
    exit 1
  fi

  if git rev-parse "v${version}" >/dev/null 2>&1; then
    echo "tag v${version} already exists" >&2
    exit 1
  fi

  current_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
  if [[ "${current_version}" == "${version}" ]]; then
    echo "Cargo.toml is already at version ${version}" >&2
    exit 1
  fi

  perl -0pi -e 's/^version = ".*"$/version = "'"${version}"'"/m' Cargo.toml
  cargo generate-lockfile
  cargo fmt --all
  cargo test
  cargo publish --dry-run --allow-dirty --registry crates-io

  git add Cargo.toml Cargo.lock
  git commit -m "chore: release v${version}"
  git tag -a "v${version}" -m "v${version}"

  echo "Prepared release v${version}."
  echo "Next step: just publish ${version}"

# Publish an already prepared release:
# - require Cargo.toml and the local tag to match
# - publish to crates.io
# - push the current branch and the matching release tag
# Pushing the tag will trigger the GitHub Actions release workflow.
publish version:
  #!/usr/bin/env bash
  set -euo pipefail

  version="{{version}}"
  version="${version#v}"

  current_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
  if [[ "${current_version}" != "${version}" ]]; then
    echo "Cargo.toml version is ${current_version}, expected ${version}. Run: just prepare-release ${version}" >&2
    exit 1
  fi

  if ! git rev-parse "v${version}" >/dev/null 2>&1; then
    echo "tag v${version} does not exist locally. Run: just prepare-release ${version}" >&2
    exit 1
  fi

  tag_commit="$(git rev-list -n 1 "v${version}")"
  head_commit="$(git rev-parse HEAD)"
  if [[ "${tag_commit}" != "${head_commit}" ]]; then
    echo "tag v${version} is not pointing at HEAD. Check out the tagged release commit before publishing." >&2
    exit 1
  fi

  cargo publish --registry crates-io
  git push origin HEAD
  git push origin "v${version}"

  echo "Published v${version} and pushed the release commit and tag."
