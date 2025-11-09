set windows-shell := ["powershell.exe"]
export RUST_LOG := "info,wgpu_core=off"
export RUST_BACKTRACE := "1"

# Displays the list of available commands
@just:
    just --list

# Builds the project in release mode
build:
    cargo build -r

# Runs cargo check and format check
check:
    cargo check --all --tests
    cargo fmt --all -- --check

# Generates and opens documentation for a specific project
docs $project="nightshade":
    cargo doc --open -p {{project}}

# Fixes linting issues automatically
fix:
    cargo clippy --all --tests --fix

# Formats the code using cargo fmt
format:
    cargo fmt --all

# Install wasm tooling
init-wasm:
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk

# Runs linter and displays warnings
lint:
    cargo clippy --all --tests -- -D warnings
    just lint-wasm

# Runs linter for wasm32 target
lint-wasm:
    cargo clippy --target wasm32-unknown-unknown -- -D warnings

# Publishes the nightshade crate to crates.io
publish:
    cargo publish -p nightshade
    cargo publish -p nightshade-editor

# Dry run of publishing the nightshade crate
publish-dry:
    cargo publish -p nightshade --dry-run
    cargo publish -p nightshade-editor --dry-run

# Runs the specified project
run $project="nightshade-editor":
    cargo run -r -p {{project}}

# Build the app for WASM
build-wasm $project="editor":
    trunk build --config apps/{{project}}/Trunk.toml

# Serve the app in browser
run-wasm $project="editor":
    trunk serve --open --config apps/{{project}}/Trunk.toml

# Runs all tests
test:
    cargo test --all -- --nocapture

# Checks for unused dependencies
udeps:
    cargo machete

# Displays version information for Rust tools
@versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version

# Watches for changes and runs the specified project
watch $project="nightshade-editor":
    cargo watch -x 'run -r -p {{project}}'

# Generates changelog using git-cliff
changelog:
    git cliff -o CHANGELOG.md

# Shows all git tags
show-tags:
    git tag -l --sort=-v:refname

# Shows the current version from Cargo.toml (Windows)
[windows]
show-version:
    "v" + (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value

# Shows the current version from Cargo.toml (Unix)
[unix]
show-version:
    @echo "v$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"

# Deletes a git tag locally and remotely
strip-tag tag:
    git tag -d {{tag}}
    git push origin :refs/tags/{{tag}}

# Creates a git tag using the version from Cargo.toml and updates changelog (Windows)
[windows]
tag:
    $version = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; Write-Host "Creating tag v$version"; git cliff --tag "v$version" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore(release): prepare for v$version"; git tag "v$version"; Write-Host ""; Write-Host "Tag created! To push, run:"; Write-Host "  git push origin v$version && git push" -ForegroundColor Green

# Creates a git tag using the version from Cargo.toml and updates changelog (Unix)
[unix]
tag:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo "Creating tag v$VERSION"
    git cliff --tag "v$VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore(release): prepare for v$VERSION"
    git tag "v$VERSION"
    echo ""
    echo "Tag created! To push, run:"
    echo "  git push origin v$VERSION && git push"
