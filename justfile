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
publish-crate:
    cargo publish -p nightshade
    cargo publish -p nightshade-editor

# Dry run of publishing the nightshade crate
publish-crate-dry:
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

# Shows git tags for the current commit
show-tags:
    git tag --points-at HEAD

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

# Pushes a version tag and commits (Windows)
[windows]
push-version:
    $version = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; git push origin "v$version"; git push

# Pushes a version tag and commits (Unix)
[unix]
push-version:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    git push origin "v$VERSION"
    git push

# Creates a GitHub release for the current version (Windows)
[windows]
publish-release:
    $version = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; gh release create "v$version" --title "nightshade-v$version" --notes-file CHANGELOG.md

# Creates a GitHub release for the current version (Unix)
[unix]
publish-release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    gh release create "v$VERSION" --title "nightshade-v$VERSION" --notes-file CHANGELOG.md

# Shows the GitHub release for the current version (Windows)
[windows]
show-release:
    $version = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; gh release view "v$version"

# Shows the GitHub release for the current version (Unix)
[unix]
show-release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    gh release view "v$VERSION"

# Deletes a GitHub release (by tag, e.g. v0.1.11) (Windows)
[windows]
strip-release tag:
    gh release delete {{tag}} --yes
    Write-Host ""
    Write-Host "To delete the git tag as well, run:" -ForegroundColor Green
    Write-Host "  just strip-tag {{tag}}" -ForegroundColor Green

# Deletes a GitHub release (by tag, e.g. v0.1.11) (Unix)
[unix]
strip-release tag:
    gh release delete {{tag}} --yes
    @echo ""
    @echo "To delete the git tag as well, run:"
    @echo "  just strip-tag {{tag}}"

# Bumps the minor version, updates changelog, and creates a git tag (Windows)
[windows]
bump-minor-version:
    $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newMinor = [int]$parts[1] + 1; $newVersion = "$($parts[0]).$newMinor.0"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; git add Cargo.toml crates/nightshade/README.md; git commit -m "chore: bump version to v$newVersion"; git tag "v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

# Bumps the minor version, updates changelog, and creates a git tag (Unix)
[unix]
bump-minor-version:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IFS='.' read -ra PARTS <<< "$CURRENT_VERSION"
    NEW_MINOR=$((PARTS[1] + 1))
    NEW_VERSION="${PARTS[0]}.$NEW_MINOR.0"
    echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    git add Cargo.toml crates/nightshade/README.md
    git commit -m "chore: bump version to v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps the major version, updates changelog, and creates a git tag (Windows)
[windows]
bump-major-version:
    $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newMajor = [int]$parts[0] + 1; $newVersion = "$newMajor.0.0"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; git add Cargo.toml crates/nightshade/README.md; git commit -m "chore: bump version to v$newVersion"; git tag "v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

# Bumps the major version, updates changelog, and creates a git tag (Unix)
[unix]
bump-major-version:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IFS='.' read -ra PARTS <<< "$CURRENT_VERSION"
    NEW_MAJOR=$((PARTS[0] + 1))
    NEW_VERSION="$NEW_MAJOR.0.0"
    echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    git add Cargo.toml crates/nightshade/README.md
    git commit -m "chore: bump version to v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps the patch version, updates changelog, and creates a git tag (Windows)
[windows]
bump-patch-version:
    $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newPatch = [int]$parts[2] + 1; $newVersion = "$($parts[0]).$($parts[1]).$newPatch"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; git add Cargo.toml crates/nightshade/README.md; git commit -m "chore: bump version to v$newVersion"; git tag "v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

# Bumps the patch version, updates changelog, and creates a git tag (Unix)
[unix]
bump-patch-version:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IFS='.' read -ra PARTS <<< "$CURRENT_VERSION"
    NEW_PATCH=$((PARTS[2] + 1))
    NEW_VERSION="${PARTS[0]}.${PARTS[1]}.$NEW_PATCH"
    echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    git add Cargo.toml crates/nightshade/README.md
    git commit -m "chore: bump version to v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps patch version, pushes, and publishes to crates.io (Windows)
[windows]
publish-patch:
    just bump-patch-version
    just push-version
    just publish-crate
    Write-Host ""
    Write-Host "To publish a GitHub release for this version, run:" -ForegroundColor Green
    Write-Host "  just publish-release" -ForegroundColor Green

# Bumps patch version, pushes, and publishes to crates.io (Unix)
[unix]
publish-patch:
    just bump-patch-version
    just push-version
    just publish-crate
    @echo ""
    @echo "To publish a GitHub release for this version, run:"
    @echo "  just publish-release"

# Bumps minor version, pushes, and publishes to crates.io (Windows)
[windows]
publish-minor:
    just bump-minor-version
    just push-version
    just publish-crate
    Write-Host ""
    Write-Host "To publish a GitHub release for this version, run:" -ForegroundColor Green
    Write-Host "  just publish-release" -ForegroundColor Green

# Bumps minor version, pushes, and publishes to crates.io (Unix)
[unix]
publish-minor:
    just bump-minor-version
    just push-version
    just publish-crate
    @echo ""
    @echo "To publish a GitHub release for this version, run:"
    @echo "  just publish-release"

# Bumps major version, pushes, and publishes to crates.io (Windows)
[windows]
publish-major:
    just bump-major-version
    just push-version
    just publish-crate
    Write-Host ""
    Write-Host "To publish a GitHub release for this version, run:" -ForegroundColor Green
    Write-Host "  just publish-release" -ForegroundColor Green

# Bumps major version, pushes, and publishes to crates.io (Unix)
[unix]
publish-major:
    just bump-major-version
    just push-version
    just publish-crate
    @echo ""
    @echo "To publish a GitHub release for this version, run:"
    @echo "  just publish-release"

# Builds the project for Steam Deck using cross
build-steamdeck $project="editor":
    cross build --release --target x86_64-unknown-linux-gnu -p {{project}}

# Builds and deploys the project to Steam Deck
deploy-steamdeck $project="editor":
    just build-steamdeck {{project}}
    scp ./target/x86_64-unknown-linux-gnu/release/{{project}} deck@steamdeck.local:~/Downloads
