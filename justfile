set windows-shell := ["powershell.exe"]
export RUST_LOG := "info,wgpu_core=off"
export RUST_BACKTRACE := "1"

template_repo := "https://github.com/matthewjberger/nightshade-template.git"

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

# Install Debian/Ubuntu dependencies for building
[linux]
install-debian-deps:
    sudo apt-get update && sudo apt-get install -y libudev-dev libasound2-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libwebkit2gtk-4.1-dev

# Installs mise and the editor's pinned wasm toolchain (Windows)
[windows]
install-deps:
    if (-not (Get-Command mise -ErrorAction SilentlyContinue)) { cargo install mise --locked }
    just --justfile apps/editor/justfile --working-directory apps/editor install-deps

# Installs mise and the editor's pinned wasm toolchain (Unix)
[unix]
install-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v mise >/dev/null || cargo install mise --locked
    just --justfile apps/editor/justfile --working-directory apps/editor install-deps

# Install development tools
install-tools:
    cargo install cargo-license
    cargo install cargo-about
    cargo install cargo-deny
    cargo install cargo-machete
    cargo install git-cliff

# Install git hooks (Windows)
[windows]
install-hooks:
    Copy-Item -Path hooks/pre-commit -Destination .git/hooks/pre-commit -Force

# Install git hooks (Unix)
[unix]
install-hooks:
    cp hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

# Uninstall git hooks (Windows)
[windows]
uninstall-hooks:
    Remove-Item -Path .git/hooks/pre-commit -Force -ErrorAction SilentlyContinue

# Uninstall git hooks (Unix)
[unix]
uninstall-hooks:
    rm -f .git/hooks/pre-commit

# Runs linter and displays warnings
lint: lint-wasm
    cargo clippy --all --tests -- -D warnings

# Runs linter for wasm32 target
lint-wasm:
    cargo clippy --target wasm32-unknown-unknown --workspace -- -D warnings

# Pushes template changes to nightshade-template repo
push-template:
    git subtree push --prefix=template {{template_repo}} main

# Force syncs template to nightshade-template repo (overwrites remote)
sync-template:
    git subtree split --prefix=template -b template-branch
    git push --force {{template_repo}} template-branch:main
    git branch -D template-branch

# Pulls updates from nightshade-template repo
pull-template:
    git subtree pull --prefix=template {{template_repo}} main --squash

# Publishes the nightshade crate to crates.io
publish-crate:
    cargo publish -p nightshade

# Dry run of publishing the nightshade crate
publish-crate-dry:
    cargo publish -p nightshade --dry-run

# Opens the editor, or runs a native app by name (e.g. `just run sandbox`)
run *project:
    {{ if project == "" { "just --justfile apps/editor/justfile --working-directory apps/editor run" } else { "cargo run -r -p " + project } }}

# Builds and opens the MCP-driveable editor (agent feature on, endpoint at
# http://127.0.0.1:8790/mcp)
run-agent:
    just --justfile apps/editor/justfile --working-directory apps/editor run-agent

# Builds the editor and serves it in the browser at http://127.0.0.1:8080
run-web:
    just --justfile apps/editor/justfile --working-directory apps/editor run-web

# Runs a native app by name
run-app $project="sandbox" *arguments="":
    cargo run -r -p {{project}} -- {{arguments}}

# Serves an app in the browser by name (e.g. `just run-wasm sandbox`)
run-wasm $project="sandbox":
    trunk serve --release --open --config apps/{{project}}/Trunk.toml

# Opens the UI test harness in preview mode (no automated tests, just the UI)
[windows]
preview-ui-test-harness:
    $env:UI_HARNESS_PREVIEW = "1"; cargo run -r -p ui_test_harness

# Opens the UI test harness in preview mode (no automated tests, just the UI)
[unix]
preview-ui-test-harness:
    UI_HARNESS_PREVIEW=1 cargo run -r -p ui_test_harness

# Builds the browser editor (worker wasm plus Leptos bundle)
build-editor:
    just --justfile apps/editor/justfile --working-directory apps/editor build

# Installs the standalone editor as the global nightshade-editor command
install-editor:
    just --justfile apps/editor/justfile --working-directory apps/editor install

# Uploads the editor web bundle to the GitHub release and publishes the crate
publish-editor:
    just --justfile apps/editor/justfile --working-directory apps/editor publish-editor

# Runs all tests
test:
    cargo test --all -- --nocapture

# Checks for unused dependencies
udeps:
    cargo machete

# Prints a table of all dependencies and their licenses
licenses:
    cargo license

# Checks for problematic licenses in dependencies
licenses-check:
    cargo deny check licenses

# Generates third-party license attribution document
licenses-html:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html

# Checks dependencies for known security advisories (RustSec)
audit:
    cargo deny check advisories

# Runs all cargo-deny checks (licenses, advisories, bans, sources)
deny-check:
    cargo deny check all

# Vendors all dependencies into the vendor directory
vendor:
    cargo vendor

# Displays version information for Rust tools
@versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version

# Watches for changes and runs the specified project
watch $project="sandbox":
    cargo watch -x 'run -r -p {{project}}'

# Generates changelog using git-cliff
changelog:
    git cliff -o CHANGELOG.md

# Shows the last tagged commit
show-tag:
    git describe --tags --abbrev=0

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

# Creates a GitHub release for the current version with the editor web bundle
# attached. Immutable releases only accept assets at creation, so the bundle
# must be built and attached here, never uploaded afterward. (Windows)
[windows]
publish-release:
    just --justfile apps/editor/justfile --working-directory apps/editor bundle
    $ErrorActionPreference = 'Stop'; $version = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; if (git status --porcelain apps/editor/desktop/bundle.sha256 apps/editor/Cargo.lock) { git add apps/editor/desktop/bundle.sha256 apps/editor/Cargo.lock; git commit -m "chore: update editor bundle for v$version"; git push }; git cliff --latest | gh release create "v$version" --title "nightshade-v$version" --notes-file - apps/editor/target/nightshade-editor-web.tar.gz; if ($LASTEXITCODE -ne 0) { throw "gh release create failed" }

# Creates a GitHub release for the current version with the editor web bundle
# attached. Immutable releases only accept assets at creation, so the bundle
# must be built and attached here, never uploaded afterward. (Unix)
[unix]
publish-release:
    #!/usr/bin/env bash
    set -euo pipefail
    just --justfile apps/editor/justfile --working-directory apps/editor bundle
    VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [ -n "$(git status --porcelain apps/editor/desktop/bundle.sha256 apps/editor/Cargo.lock)" ]; then
        git add apps/editor/desktop/bundle.sha256 apps/editor/Cargo.lock
        git commit -m "chore: update editor bundle for v$VERSION"
        git push
    fi
    git cliff --latest | gh release create "v$VERSION" --title "nightshade-v$VERSION" --notes-file - apps/editor/target/nightshade-editor-web.tar.gz

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
    $ErrorActionPreference = 'Stop'; $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newMinor = [int]$parts[1] + 1; $newVersion = "$($parts[0]).$newMinor.0"; $currentMajorMinor = "$($parts[0]).$($parts[1])"; $newMajorMinor = "$($parts[0]).$newMinor"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; (Get-Content 'template/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template/Cargo.toml'; (Get-Content 'crates/nightshade/src/lib.rs') -replace "nightshade = `"$currentMajorMinor`"", "nightshade = `"$newMajorMinor`"" | Set-Content 'crates/nightshade/src/lib.rs'; (Get-Content 'template/README.md') -replace "nightshade = \{ version = `"$currentMajorMinor`"", "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'template/README.md'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace 'nightshade = \{ version = "[0-9.]+"', "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; git add Cargo.toml crates/nightshade/README.md template/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/desktop/Cargo.toml; git commit -m "chore: bump version to v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; git tag "v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

# Bumps the minor version, updates changelog, and creates a git tag (Unix)
[unix]
bump-minor-version:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IFS='.' read -ra PARTS <<< "$CURRENT_VERSION"
    NEW_MINOR=$((PARTS[1] + 1))
    NEW_VERSION="${PARTS[0]}.$NEW_MINOR.0"
    CURRENT_MM="${PARTS[0]}.${PARTS[1]}"
    NEW_MM="${PARTS[0]}.$NEW_MINOR"
    echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template/Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_MM\"/nightshade = \"$NEW_MM\"/" crates/nightshade/src/lib.rs
    sed -i "s/nightshade = { version = \"$CURRENT_MM\"/nightshade = { version = \"$NEW_MM\"/" template/README.md
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" apps/editor/desktop/Cargo.toml
    sed -i "s/nightshade = { version = \"[0-9.]*\"/nightshade = { version = \"$NEW_MM\"/" apps/editor/desktop/Cargo.toml
    git add Cargo.toml crates/nightshade/README.md template/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/desktop/Cargo.toml
    git commit -m "chore: bump version to v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps the major version, updates changelog, and creates a git tag (Windows)
[windows]
bump-major-version:
    $ErrorActionPreference = 'Stop'; $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newMajor = [int]$parts[0] + 1; $newVersion = "$newMajor.0.0"; $currentMajorMinor = "$($parts[0]).$($parts[1])"; $newMajorMinor = "$newMajor.0"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; (Get-Content 'template/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template/Cargo.toml'; (Get-Content 'crates/nightshade/src/lib.rs') -replace "nightshade = `"$currentMajorMinor`"", "nightshade = `"$newMajorMinor`"" | Set-Content 'crates/nightshade/src/lib.rs'; (Get-Content 'template/README.md') -replace "nightshade = \{ version = `"$currentMajorMinor`"", "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'template/README.md'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace 'nightshade = \{ version = "[0-9.]+"', "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; git add Cargo.toml crates/nightshade/README.md template/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/desktop/Cargo.toml; git commit -m "chore: bump version to v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; git tag "v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

# Bumps the major version, updates changelog, and creates a git tag (Unix)
[unix]
bump-major-version:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IFS='.' read -ra PARTS <<< "$CURRENT_VERSION"
    NEW_MAJOR=$((PARTS[0] + 1))
    NEW_VERSION="$NEW_MAJOR.0.0"
    CURRENT_MM="${PARTS[0]}.${PARTS[1]}"
    NEW_MM="$NEW_MAJOR.0"
    echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template/Cargo.toml
    sed -i "s/nightshade = \"$CURRENT_MM\"/nightshade = \"$NEW_MM\"/" crates/nightshade/src/lib.rs
    sed -i "s/nightshade = { version = \"$CURRENT_MM\"/nightshade = { version = \"$NEW_MM\"/" template/README.md
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" apps/editor/desktop/Cargo.toml
    sed -i "s/nightshade = { version = \"[0-9.]*\"/nightshade = { version = \"$NEW_MM\"/" apps/editor/desktop/Cargo.toml
    git add Cargo.toml crates/nightshade/README.md template/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/desktop/Cargo.toml
    git commit -m "chore: bump version to v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps the patch version, updates changelog, and creates a git tag (Windows)
[windows]
bump-patch-version:
    $ErrorActionPreference = 'Stop'; $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newPatch = [int]$parts[2] + 1; $newVersion = "$($parts[0]).$($parts[1]).$newPatch"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; (Get-Content 'template/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; git add Cargo.toml crates/nightshade/README.md template/Cargo.toml apps/editor/desktop/Cargo.toml; git commit -m "chore: bump version to v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; git tag "v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

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
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template/Cargo.toml
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" apps/editor/desktop/Cargo.toml
    git add Cargo.toml crates/nightshade/README.md template/Cargo.toml apps/editor/desktop/Cargo.toml
    git commit -m "chore: bump version to v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps patch version, pushes, and publishes to crates.io (Windows)
[windows]
publish-patch:
    just bump-patch-version
    just push-version
    just publish-crate
    just push-template
    Write-Host ""
    Write-Host "To create a GitHub release, run:" -ForegroundColor Green
    Write-Host "  just publish-release" -ForegroundColor Green

# Bumps patch version, pushes, and publishes to crates.io (Unix)
[unix]
publish-patch:
    just bump-patch-version
    just push-version
    just publish-crate
    just push-template
    @echo ""
    @echo "To create a GitHub release, run:"
    @echo "  just publish-release"

# Bumps minor version, pushes, and publishes to crates.io (Windows)
[windows]
publish-minor:
    just bump-minor-version
    just push-version
    just publish-crate
    just push-template
    Write-Host ""
    Write-Host "To create a GitHub release, run:" -ForegroundColor Green
    Write-Host "  just publish-release" -ForegroundColor Green

# Bumps minor version, pushes, and publishes to crates.io (Unix)
[unix]
publish-minor:
    just bump-minor-version
    just push-version
    just publish-crate
    just push-template
    @echo ""
    @echo "To create a GitHub release, run:"
    @echo "  just publish-release"

# Bumps major version, pushes, and publishes to crates.io (Windows)
[windows]
publish-major:
    just bump-major-version
    just push-version
    just publish-crate
    just push-template
    Write-Host ""
    Write-Host "To create a GitHub release, run:" -ForegroundColor Green
    Write-Host "  just publish-release" -ForegroundColor Green

# Bumps major version, pushes, and publishes to crates.io (Unix)
[unix]
publish-major:
    just bump-major-version
    just push-version
    just publish-crate
    just push-template
    @echo ""
    @echo "To create a GitHub release, run:"
    @echo "  just publish-release"

# Builds the project for Steam Deck using cross
build-steamdeck $project="sandbox":
    cross build --release --target x86_64-unknown-linux-gnu -p {{project}}

# Builds and deploys the project to Steam Deck
deploy-steamdeck $project="sandbox":
    just build-steamdeck {{project}}
    scp ./target/x86_64-unknown-linux-gnu/release/{{project}} deck@steamdeck.local:~/Downloads

# Builds and deploys the project to Steam Deck as ~/Downloads/game
deploy-steamdeck-quick $project="sandbox":
    just build-steamdeck {{project}}
    scp ./target/x86_64-unknown-linux-gnu/release/{{project}} deck@steamdeck.local:~/Downloads/game
    ssh deck@steamdeck.local "chmod +x ~/Downloads/game"

# SSH into Steam Deck
steamdeck-ssh:
    ssh deck@steamdeck.local

