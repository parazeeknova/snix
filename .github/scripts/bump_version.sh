#!/usr/bin/env bash
set -e

# Extract current version
current_version=$(grep '^version =' Cargo.toml | head -1 | cut -d'"' -f2)
major=$(echo $current_version | cut -d. -f1)
minor=$(echo $current_version | cut -d. -f2)
patch=$(echo $current_version | cut -d. -f3)

# Increment patch
patch=$((patch + 1))

# If patch > 9, bump minor and reset patch
if [ "$patch" -gt 9 ]; then
  minor=$((minor + 1))
  patch=0
fi

new_version="$major.$minor.$patch"

# Update Cargo.toml
sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml

echo "$new_version"