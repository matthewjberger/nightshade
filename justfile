set windows-shell := ["powershell.exe"]
export RUST_LOG := "info,wgpu_core=off"
export RUST_BACKTRACE := "1"

template_repo := "https://github.com/matthewjberger/nightshade-template.git"
template_leptos_repo := "https://github.com/matthewjberger/nightshade-template-leptos.git"
template_api_leptos_repo := "https://github.com/matthewjberger/nightshade-api-template-leptos.git"
template_api_repo := "https://github.com/matthewjberger/nightshade-api-template.git"

# Displays the list of available commands
@just:
    just --list

# Builds the project in release mode
build:
    cargo build -r

# Runs cargo check and format check
check:
    cargo check --all --all-targets
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
    cargo install --locked cargo-license
    cargo install --locked cargo-about
    cargo install --locked cargo-deny
    cargo install --locked cargo-machete
    cargo install --locked git-cliff

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
lint: lint-wasm lint-rendergraph lint-crates-isolated check-feature-forwarding lint-dag
    cargo clippy --all --all-targets -- -D warnings

# Enforces that the engine layer stays a module dependency DAG: the ecs
# domains, plugins, app, and runner must not form a reference cycle. A domain
# that reaches back up (say `use crate::plugins::physics` inside a domain)
# fails here instead of quietly reintroducing a cycle.
lint-dag:
    cargo run --quiet -p dag-check

# Clippy each publishable crate on its own default features. The workspace
# lint above unifies features across every crate, so a field or item that only
# has readers behind an optional feature still looks used and its dead-code
# warning stays hidden. Building each crate alone reproduces what a downstream
# consumer that enables only the narrow feature set actually compiles, so those
# warnings surface here instead of in someone else's build.
lint-crates-isolated:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in nightshade-ecs nightshade-text nightshade-paint nightshade-renderer nightshade-platform nightshade-ui nightshade nightshade-api; do
        echo "clippy (isolated): ${crate}"
        cargo clippy -p "${crate}" --all-targets -- -D warnings
    done

# Runs linter for wasm32 target
# The meshlet baker links METIS and meshopt, which are C libraries with no
# wasm target, so the app that bakes is native only and sits out this lint.
lint-wasm:
    cargo clippy --target wasm32-unknown-unknown --workspace --exclude meshlet --exclude dag-check -- -D warnings
    cargo clippy --target wasm32-unknown-unknown -p nightshade-api --no-default-features --features leptos,offscreen -- -D warnings

# Builds the frame graph alone. This is the layering firewall: the rendergraph
# module must compile with the rest of the renderer absent, so any reach from
# the graph into renderer code fails here.
lint-rendergraph:
    cargo clippy -p nightshade-renderer --no-default-features --features rendergraph -- -D warnings

# Verifies every forwarding feature in nightshade still forwards to the
# matching nightshade-renderer feature, so a dropped entry fails the gate
# instead of silently compiling the renderer without it.
check-feature-forwarding:
    #!/usr/bin/env bash
    status=0
    for feature in wgpu hdr egui debug_render screenshot grass terrain browser_yield meshlet meshlet_processor; do
        if ! grep -q "nightshade-renderer/${feature}" crates/nightshade/Cargo.toml; then
            echo "missing forwarding: nightshade '${feature}' does not forward to nightshade-renderer/${feature}"
            status=1
        fi
    done
    for target in nightshade-platform nightshade-ui; do
        if ! grep -q "${target}/gamepad" crates/nightshade/Cargo.toml; then
            echo "missing forwarding: nightshade 'gamepad' does not forward to ${target}/gamepad"
            status=1
        fi
    done
    exit ${status}

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

# Pushes template_leptos changes to nightshade-template-leptos repo
push-template-leptos:
    git subtree push --prefix=template_leptos {{template_leptos_repo}} main

# Force syncs template_leptos to nightshade-template-leptos repo (overwrites remote)
sync-template-leptos:
    git subtree split --prefix=template_leptos -b template-leptos-branch
    git push --force {{template_leptos_repo}} template-leptos-branch:main
    git branch -D template-leptos-branch

# Pulls updates from nightshade-template-leptos repo
pull-template-leptos:
    git subtree pull --prefix=template_leptos {{template_leptos_repo}} main --squash

# Pushes template_api_leptos changes to nightshade-api-template-leptos repo
push-template-api-leptos:
    git subtree push --prefix=template_api_leptos {{template_api_leptos_repo}} main

# Force syncs template_api_leptos to nightshade-api-template-leptos repo (overwrites remote)
sync-template-api-leptos:
    git subtree split --prefix=template_api_leptos -b template-api-leptos-branch
    git push --force {{template_api_leptos_repo}} template-api-leptos-branch:main
    git branch -D template-api-leptos-branch

# Pulls updates from nightshade-api-template-leptos repo
pull-template-api-leptos:
    git subtree pull --prefix=template_api_leptos {{template_api_leptos_repo}} main --squash

# Pushes template_api changes to nightshade-api-template repo
push-template-api:
    git subtree push --prefix=template_api {{template_api_repo}} main

# Force syncs template_api to nightshade-api-template repo (overwrites remote)
sync-template-api:
    git subtree split --prefix=template_api -b template-api-branch
    git push --force {{template_api_repo}} template-api-branch:main
    git branch -D template-api-branch

# Pulls updates from nightshade-api-template repo
pull-template-api:
    git subtree pull --prefix=template_api {{template_api_repo}} main --squash

# Publishes the nightshade-ecs, nightshade-text, nightshade-paint, nightshade-renderer, nightshade-platform, nightshade-ui, nightshade, and nightshade-api crates to crates.io
publish-crate:
    cargo publish -p nightshade-ecs
    cargo publish -p nightshade-text
    cargo publish -p nightshade-paint
    cargo publish -p nightshade-renderer
    cargo publish -p nightshade-platform
    cargo publish -p nightshade-ui
    cargo publish -p nightshade
    cargo publish -p nightshade-api

# Dry run of publishing the nightshade-ecs, nightshade-text, nightshade-paint, nightshade-renderer, nightshade-platform, nightshade-ui, nightshade, and nightshade-api crates
publish-crate-dry:
    cargo publish -p nightshade-ecs --dry-run
    cargo publish -p nightshade-text --dry-run
    cargo publish -p nightshade-paint --dry-run
    cargo publish -p nightshade-renderer --dry-run
    cargo publish -p nightshade-platform --dry-run
    cargo publish -p nightshade-ui --dry-run
    cargo publish -p nightshade --dry-run
    cargo publish -p nightshade-api --dry-run

# Opens the editor (with its built-in scripting playground tab), or runs a
# native app by name (e.g. `just run sandbox`)
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

# Runs an OpenXR VR example by name (e.g. `just run-vr hands`). Needs a desktop
# OpenXR runtime and a connected headset.
run-vr $name="hello_vr":
    cargo run -r -p nightshade-openxr --example {{name}}

# Runs a nightshade-api example by name (e.g. `just run-example solar_system`)
run-example $name="spinning_cube":
    cargo run -r -p nightshade-api --example {{name}}

# Serves a nightshade-api example as wasm in the browser via trunk, the same
# pipeline the apps use (crates/nightshade-api/index.html + Trunk.toml). (Windows)
[windows]
run-example-wasm $name="collector":
    $env:TRUNK_BUILD_EXAMPLE = "{{name}}"; trunk serve --release --open --config crates/nightshade-api/Trunk.toml

# Serves a nightshade-api example as wasm in the browser via trunk, the same
# pipeline the apps use (crates/nightshade-api/index.html + Trunk.toml). (Unix)
[unix]
run-example-wasm $name="collector":
    TRUNK_BUILD_EXAMPLE={{name}} trunk serve --release --open --config crates/nightshade-api/Trunk.toml

# Lists the nightshade-api examples
[windows]
list-examples:
    Get-ChildItem crates/nightshade-api/examples -Filter *.rs | ForEach-Object { $_.BaseName }

# Lists the nightshade-api examples
[unix]
list-examples:
    @ls crates/nightshade-api/examples | sed 's/\.rs$//'

# Runs every nightshade-api example for 120 frames as a local smoke gate
[windows]
test-examples:
    $ErrorActionPreference = 'Stop'; Get-ChildItem crates/nightshade-api/examples -Filter *.rs | ForEach-Object { $env:NIGHTSHADE_API_FRAMES = "120"; Write-Host "Running example $($_.BaseName)"; cargo run -r -p nightshade-api --example $_.BaseName; if ($LASTEXITCODE -ne 0) { throw "example $($_.BaseName) failed" } }; Remove-Item Env:\NIGHTSHADE_API_FRAMES

# Runs every nightshade-api example for 120 frames as a local smoke gate
[unix]
test-examples:
    #!/usr/bin/env bash
    set -euo pipefail
    for example in crates/nightshade-api/examples/*.rs; do
        name=$(basename "$example" .rs)
        echo "Running example $name"
        NIGHTSHADE_API_FRAMES=120 cargo run -r -p nightshade-api --example "$name"
    done

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

# Installs the standalone editor as the global nightshade command
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
    $ErrorActionPreference = 'Stop'; $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newMinor = [int]$parts[1] + 1; $newVersion = "$($parts[0]).$newMinor.0"; $currentMajorMinor = "$($parts[0]).$($parts[1])"; $newMajorMinor = "$($parts[0]).$newMinor"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-renderer = \{ version = `"$currentVersion`"", "nightshade-renderer = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-platform = \{ version = `"$currentVersion`"", "nightshade-platform = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-text = \{ version = `"$currentVersion`"", "nightshade-text = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-paint = \{ version = `"$currentVersion`"", "nightshade-paint = { version = `"$newVersion`"" | Set-Content 'Cargo.toml';(Get-Content 'Cargo.toml') -replace "nightshade-ui = \{ version = `"$currentVersion`"", "nightshade-ui = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade-renderer/README.md') -replace "nightshade-renderer = `"$currentVersion`"", "nightshade-renderer = `"$newVersion`"" | Set-Content 'crates/nightshade-renderer/README.md'; (Get-Content 'crates/nightshade-renderer/README.md') -replace "nightshade-renderer = \{ version = `"$currentVersion`"", "nightshade-renderer = { version = `"$newVersion`"" | Set-Content 'crates/nightshade-renderer/README.md'; (Get-Content 'crates/nightshade-ecs/README.md') -replace "nightshade-ecs = `"$currentVersion`"", "nightshade-ecs = `"$newVersion`"" | Set-Content 'crates/nightshade-ecs/README.md'; (Get-Content 'crates/nightshade-platform/README.md') -replace "nightshade-platform = `"$currentVersion`"", "nightshade-platform = `"$newVersion`"" | Set-Content 'crates/nightshade-platform/README.md'; (Get-Content 'crates/nightshade-text/README.md') -replace "nightshade-text = `"$currentVersion`"", "nightshade-text = `"$newVersion`"" | Set-Content 'crates/nightshade-text/README.md'; (Get-Content 'crates/nightshade-paint/README.md') -replace "nightshade-paint = `"$currentVersion`"", "nightshade-paint = `"$newVersion`"" | Set-Content 'crates/nightshade-paint/README.md';(Get-Content 'crates/nightshade-ui/README.md') -replace "nightshade-ui = `"$currentVersion`"", "nightshade-ui = `"$newVersion`"" | Set-Content 'crates/nightshade-ui/README.md';(Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; (Get-Content 'template/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template/Cargo.toml'; (Get-Content 'template_leptos/worker/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/worker/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/worker/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/Cargo.toml'; (Get-Content 'template_leptos/worker/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_leptos/worker/Cargo.toml'; (Get-Content 'template_leptos/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_leptos/Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade-api/README.md') -replace "nightshade-api = `"$currentVersion`"", "nightshade-api = `"$newVersion`"" | Set-Content 'crates/nightshade-api/README.md'; (Get-Content 'template_api/Cargo.toml') -replace "nightshade-api = `"$currentVersion`"", "nightshade-api = `"$newVersion`"" | Set-Content 'template_api/Cargo.toml'; (Get-Content 'crates/nightshade-api/src/lib.rs') -replace "nightshade-api = `"$currentMajorMinor`"", "nightshade-api = `"$newMajorMinor`"" | Set-Content 'crates/nightshade-api/src/lib.rs'; (Get-Content 'docs/book/src/api-overview.md') -replace "nightshade-api = `"$currentMajorMinor`"", "nightshade-api = `"$newMajorMinor`"" | Set-Content 'docs/book/src/api-overview.md'; (Get-Content 'docs/book/src/installation.md') -replace "nightshade = \{ version = `"$currentMajorMinor`"", "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'docs/book/src/installation.md';(Get-Content 'crates/nightshade/src/lib.rs') -replace "nightshade = `"$currentMajorMinor`"", "nightshade = `"$newMajorMinor`"" | Set-Content 'crates/nightshade/src/lib.rs'; (Get-Content 'template/README.md') -replace "nightshade = \{ version = `"$currentMajorMinor`"", "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'template/README.md'; (Get-Content 'apps/editor/Cargo.toml') -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content 'apps/editor/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace 'nightshade = \{ version = "[0-9.]+"', "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace 'nightshade-api = \{ version = "[0-9.]+"', "nightshade-api = { version = `"$newMajorMinor`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; git add Cargo.toml crates/nightshade/README.md crates/nightshade-api/README.md crates/nightshade-renderer/README.md crates/nightshade-ecs/README.md crates/nightshade-platform/README.md crates/nightshade-text/README.md crates/nightshade-paint/README.md crates/nightshade-ui/README.md crates/nightshade-api/src/lib.rs docs/book/src/api-overview.md docs/book/src/installation.md template/Cargo.toml template_leptos/Cargo.toml template_leptos/worker/Cargo.toml template_api_leptos/Cargo.toml template_api_leptos/worker/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/Cargo.toml apps/editor/desktop/Cargo.toml template_api/Cargo.toml; git commit -m "chore: bump version to v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; git tag "v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

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
    sed -i "s/nightshade-renderer = { version = \"$CURRENT_VERSION\"/nightshade-renderer = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-platform = { version = \"$CURRENT_VERSION\"/nightshade-platform = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-text = { version = \"$CURRENT_VERSION\"/nightshade-text = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-paint = { version = \"$CURRENT_VERSION\"/nightshade-paint = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-ui = { version = \"$CURRENT_VERSION\"/nightshade-ui = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-renderer = \"$CURRENT_VERSION\"/nightshade-renderer = \"$NEW_VERSION\"/" crates/nightshade-renderer/README.md
    sed -i "s/nightshade-renderer = { version = \"$CURRENT_VERSION\"/nightshade-renderer = { version = \"$NEW_VERSION\"/" crates/nightshade-renderer/README.md
    sed -i "s/nightshade-ecs = \"$CURRENT_VERSION\"/nightshade-ecs = \"$NEW_VERSION\"/" crates/nightshade-ecs/README.md
    sed -i "s/nightshade-platform = \"$CURRENT_VERSION\"/nightshade-platform = \"$NEW_VERSION\"/" crates/nightshade-platform/README.md
    sed -i "s/nightshade-text = \"$CURRENT_VERSION\"/nightshade-text = \"$NEW_VERSION\"/" crates/nightshade-text/README.md
    sed -i "s/nightshade-paint = \"$CURRENT_VERSION\"/nightshade-paint = \"$NEW_VERSION\"/" crates/nightshade-paint/README.md
    sed -i "s/nightshade-ui = \"$CURRENT_VERSION\"/nightshade-ui = \"$NEW_VERSION\"/" crates/nightshade-ui/README.md
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template/Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template_leptos/worker/Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template_api_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_api_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_api_leptos/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_leptos/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-api = \"$CURRENT_VERSION\"/nightshade-api = \"$NEW_VERSION\"/" crates/nightshade-api/README.md
    sed -i "s/nightshade-api = \"$CURRENT_VERSION\"/nightshade-api = \"$NEW_VERSION\"/" template_api/Cargo.toml
    sed -i "s/nightshade-api = \"$CURRENT_MM\"/nightshade-api = \"$NEW_MM\"/" crates/nightshade-api/src/lib.rs
    sed -i "s/nightshade-api = \"$CURRENT_MM\"/nightshade-api = \"$NEW_MM\"/" docs/book/src/api-overview.md
    sed -i "s/nightshade = { version = \"$CURRENT_MM\"/nightshade = { version = \"$NEW_MM\"/" docs/book/src/installation.md
    sed -i "s/nightshade = \"$CURRENT_MM\"/nightshade = \"$NEW_MM\"/" crates/nightshade/src/lib.rs
    sed -i "s/nightshade = { version = \"$CURRENT_MM\"/nightshade = { version = \"$NEW_MM\"/" template/README.md
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" apps/editor/Cargo.toml
    sed -i "s/nightshade = { version = \"[0-9.]*\"/nightshade = { version = \"$NEW_MM\"/" apps/editor/desktop/Cargo.toml
    sed -i "s/nightshade-api = { version = \"[0-9.]*\"/nightshade-api = { version = \"$NEW_MM\"/" apps/editor/desktop/Cargo.toml
    git add Cargo.toml crates/nightshade/README.md crates/nightshade-api/README.md crates/nightshade-renderer/README.md crates/nightshade-ecs/README.md crates/nightshade-platform/README.md crates/nightshade-text/README.md crates/nightshade-paint/README.md crates/nightshade-ui/README.md crates/nightshade-api/src/lib.rs docs/book/src/api-overview.md docs/book/src/installation.md template/Cargo.toml template_leptos/Cargo.toml template_leptos/worker/Cargo.toml template_api_leptos/Cargo.toml template_api_leptos/worker/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/Cargo.toml apps/editor/desktop/Cargo.toml template_api/Cargo.toml
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
    $ErrorActionPreference = 'Stop'; $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newMajor = [int]$parts[0] + 1; $newVersion = "$newMajor.0.0"; $currentMajorMinor = "$($parts[0]).$($parts[1])"; $newMajorMinor = "$newMajor.0"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-renderer = \{ version = `"$currentVersion`"", "nightshade-renderer = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-platform = \{ version = `"$currentVersion`"", "nightshade-platform = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-text = \{ version = `"$currentVersion`"", "nightshade-text = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-paint = \{ version = `"$currentVersion`"", "nightshade-paint = { version = `"$newVersion`"" | Set-Content 'Cargo.toml';(Get-Content 'Cargo.toml') -replace "nightshade-ui = \{ version = `"$currentVersion`"", "nightshade-ui = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade-renderer/README.md') -replace "nightshade-renderer = `"$currentVersion`"", "nightshade-renderer = `"$newVersion`"" | Set-Content 'crates/nightshade-renderer/README.md'; (Get-Content 'crates/nightshade-renderer/README.md') -replace "nightshade-renderer = \{ version = `"$currentVersion`"", "nightshade-renderer = { version = `"$newVersion`"" | Set-Content 'crates/nightshade-renderer/README.md'; (Get-Content 'crates/nightshade-ecs/README.md') -replace "nightshade-ecs = `"$currentVersion`"", "nightshade-ecs = `"$newVersion`"" | Set-Content 'crates/nightshade-ecs/README.md'; (Get-Content 'crates/nightshade-platform/README.md') -replace "nightshade-platform = `"$currentVersion`"", "nightshade-platform = `"$newVersion`"" | Set-Content 'crates/nightshade-platform/README.md'; (Get-Content 'crates/nightshade-text/README.md') -replace "nightshade-text = `"$currentVersion`"", "nightshade-text = `"$newVersion`"" | Set-Content 'crates/nightshade-text/README.md'; (Get-Content 'crates/nightshade-paint/README.md') -replace "nightshade-paint = `"$currentVersion`"", "nightshade-paint = `"$newVersion`"" | Set-Content 'crates/nightshade-paint/README.md';(Get-Content 'crates/nightshade-ui/README.md') -replace "nightshade-ui = `"$currentVersion`"", "nightshade-ui = `"$newVersion`"" | Set-Content 'crates/nightshade-ui/README.md';(Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; (Get-Content 'template/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template/Cargo.toml'; (Get-Content 'template_leptos/worker/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/worker/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/worker/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/Cargo.toml'; (Get-Content 'template_leptos/worker/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_leptos/worker/Cargo.toml'; (Get-Content 'template_leptos/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_leptos/Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade-api/README.md') -replace "nightshade-api = `"$currentVersion`"", "nightshade-api = `"$newVersion`"" | Set-Content 'crates/nightshade-api/README.md'; (Get-Content 'template_api/Cargo.toml') -replace "nightshade-api = `"$currentVersion`"", "nightshade-api = `"$newVersion`"" | Set-Content 'template_api/Cargo.toml'; (Get-Content 'crates/nightshade-api/src/lib.rs') -replace "nightshade-api = `"$currentMajorMinor`"", "nightshade-api = `"$newMajorMinor`"" | Set-Content 'crates/nightshade-api/src/lib.rs'; (Get-Content 'docs/book/src/api-overview.md') -replace "nightshade-api = `"$currentMajorMinor`"", "nightshade-api = `"$newMajorMinor`"" | Set-Content 'docs/book/src/api-overview.md'; (Get-Content 'docs/book/src/installation.md') -replace "nightshade = \{ version = `"$currentMajorMinor`"", "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'docs/book/src/installation.md';(Get-Content 'crates/nightshade/src/lib.rs') -replace "nightshade = `"$currentMajorMinor`"", "nightshade = `"$newMajorMinor`"" | Set-Content 'crates/nightshade/src/lib.rs'; (Get-Content 'template/README.md') -replace "nightshade = \{ version = `"$currentMajorMinor`"", "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'template/README.md'; (Get-Content 'apps/editor/Cargo.toml') -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content 'apps/editor/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace 'nightshade = \{ version = "[0-9.]+"', "nightshade = { version = `"$newMajorMinor`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; (Get-Content 'apps/editor/desktop/Cargo.toml') -replace 'nightshade-api = \{ version = "[0-9.]+"', "nightshade-api = { version = `"$newMajorMinor`"" | Set-Content 'apps/editor/desktop/Cargo.toml'; git add Cargo.toml crates/nightshade/README.md crates/nightshade-api/README.md crates/nightshade-renderer/README.md crates/nightshade-ecs/README.md crates/nightshade-platform/README.md crates/nightshade-text/README.md crates/nightshade-paint/README.md crates/nightshade-ui/README.md crates/nightshade-api/src/lib.rs docs/book/src/api-overview.md docs/book/src/installation.md template/Cargo.toml template_leptos/Cargo.toml template_leptos/worker/Cargo.toml template_api_leptos/Cargo.toml template_api_leptos/worker/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/Cargo.toml apps/editor/desktop/Cargo.toml template_api/Cargo.toml; git commit -m "chore: bump version to v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; git tag "v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

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
    sed -i "s/nightshade-renderer = { version = \"$CURRENT_VERSION\"/nightshade-renderer = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-platform = { version = \"$CURRENT_VERSION\"/nightshade-platform = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-text = { version = \"$CURRENT_VERSION\"/nightshade-text = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-paint = { version = \"$CURRENT_VERSION\"/nightshade-paint = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-ui = { version = \"$CURRENT_VERSION\"/nightshade-ui = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-renderer = \"$CURRENT_VERSION\"/nightshade-renderer = \"$NEW_VERSION\"/" crates/nightshade-renderer/README.md
    sed -i "s/nightshade-renderer = { version = \"$CURRENT_VERSION\"/nightshade-renderer = { version = \"$NEW_VERSION\"/" crates/nightshade-renderer/README.md
    sed -i "s/nightshade-ecs = \"$CURRENT_VERSION\"/nightshade-ecs = \"$NEW_VERSION\"/" crates/nightshade-ecs/README.md
    sed -i "s/nightshade-platform = \"$CURRENT_VERSION\"/nightshade-platform = \"$NEW_VERSION\"/" crates/nightshade-platform/README.md
    sed -i "s/nightshade-text = \"$CURRENT_VERSION\"/nightshade-text = \"$NEW_VERSION\"/" crates/nightshade-text/README.md
    sed -i "s/nightshade-paint = \"$CURRENT_VERSION\"/nightshade-paint = \"$NEW_VERSION\"/" crates/nightshade-paint/README.md
    sed -i "s/nightshade-ui = \"$CURRENT_VERSION\"/nightshade-ui = \"$NEW_VERSION\"/" crates/nightshade-ui/README.md
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template/Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template_leptos/worker/Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template_api_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_api_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_api_leptos/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_leptos/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-api = \"$CURRENT_VERSION\"/nightshade-api = \"$NEW_VERSION\"/" crates/nightshade-api/README.md
    sed -i "s/nightshade-api = \"$CURRENT_VERSION\"/nightshade-api = \"$NEW_VERSION\"/" template_api/Cargo.toml
    sed -i "s/nightshade-api = \"$CURRENT_MM\"/nightshade-api = \"$NEW_MM\"/" crates/nightshade-api/src/lib.rs
    sed -i "s/nightshade-api = \"$CURRENT_MM\"/nightshade-api = \"$NEW_MM\"/" docs/book/src/api-overview.md
    sed -i "s/nightshade = { version = \"$CURRENT_MM\"/nightshade = { version = \"$NEW_MM\"/" docs/book/src/installation.md
    sed -i "s/nightshade = \"$CURRENT_MM\"/nightshade = \"$NEW_MM\"/" crates/nightshade/src/lib.rs
    sed -i "s/nightshade = { version = \"$CURRENT_MM\"/nightshade = { version = \"$NEW_MM\"/" template/README.md
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" apps/editor/Cargo.toml
    sed -i "s/nightshade = { version = \"[0-9.]*\"/nightshade = { version = \"$NEW_MM\"/" apps/editor/desktop/Cargo.toml
    sed -i "s/nightshade-api = { version = \"[0-9.]*\"/nightshade-api = { version = \"$NEW_MM\"/" apps/editor/desktop/Cargo.toml
    git add Cargo.toml crates/nightshade/README.md crates/nightshade-api/README.md crates/nightshade-renderer/README.md crates/nightshade-ecs/README.md crates/nightshade-platform/README.md crates/nightshade-text/README.md crates/nightshade-paint/README.md crates/nightshade-ui/README.md crates/nightshade-api/src/lib.rs docs/book/src/api-overview.md docs/book/src/installation.md template/Cargo.toml template_leptos/Cargo.toml template_leptos/worker/Cargo.toml template_api_leptos/Cargo.toml template_api_leptos/worker/Cargo.toml crates/nightshade/src/lib.rs template/README.md apps/editor/Cargo.toml apps/editor/desktop/Cargo.toml template_api/Cargo.toml
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
    $ErrorActionPreference = 'Stop'; $currentVersion = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches.Groups[1].Value; $parts = $currentVersion.Split('.'); $newPatch = [int]$parts[2] + 1; $newVersion = "$($parts[0]).$($parts[1]).$newPatch"; Write-Host "Bumping version from $currentVersion to $newVersion"; (Get-Content 'Cargo.toml') -replace "^version = `"$currentVersion`"", "version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-renderer = \{ version = `"$currentVersion`"", "nightshade-renderer = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-platform = \{ version = `"$currentVersion`"", "nightshade-platform = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-text = \{ version = `"$currentVersion`"", "nightshade-text = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-paint = \{ version = `"$currentVersion`"", "nightshade-paint = { version = `"$newVersion`"" | Set-Content 'Cargo.toml';(Get-Content 'Cargo.toml') -replace "nightshade-ui = \{ version = `"$currentVersion`"", "nightshade-ui = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade-renderer/README.md') -replace "nightshade-renderer = `"$currentVersion`"", "nightshade-renderer = `"$newVersion`"" | Set-Content 'crates/nightshade-renderer/README.md'; (Get-Content 'crates/nightshade-renderer/README.md') -replace "nightshade-renderer = \{ version = `"$currentVersion`"", "nightshade-renderer = { version = `"$newVersion`"" | Set-Content 'crates/nightshade-renderer/README.md'; (Get-Content 'crates/nightshade-ecs/README.md') -replace "nightshade-ecs = `"$currentVersion`"", "nightshade-ecs = `"$newVersion`"" | Set-Content 'crates/nightshade-ecs/README.md'; (Get-Content 'crates/nightshade-platform/README.md') -replace "nightshade-platform = `"$currentVersion`"", "nightshade-platform = `"$newVersion`"" | Set-Content 'crates/nightshade-platform/README.md'; (Get-Content 'crates/nightshade-text/README.md') -replace "nightshade-text = `"$currentVersion`"", "nightshade-text = `"$newVersion`"" | Set-Content 'crates/nightshade-text/README.md'; (Get-Content 'crates/nightshade-paint/README.md') -replace "nightshade-paint = `"$currentVersion`"", "nightshade-paint = `"$newVersion`"" | Set-Content 'crates/nightshade-paint/README.md';(Get-Content 'crates/nightshade-ui/README.md') -replace "nightshade-ui = `"$currentVersion`"", "nightshade-ui = `"$newVersion`"" | Set-Content 'crates/nightshade-ui/README.md';(Get-Content 'crates/nightshade/README.md') -replace "nightshade = `"$currentVersion`"", "nightshade = `"$newVersion`"" | Set-Content 'crates/nightshade/README.md'; (Get-Content 'template/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template/Cargo.toml'; (Get-Content 'template_leptos/worker/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/worker/Cargo.toml') -replace "nightshade = \{ version = `"$currentVersion`"", "nightshade = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/worker/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/worker/Cargo.toml'; (Get-Content 'template_api_leptos/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_api_leptos/Cargo.toml'; (Get-Content 'template_leptos/worker/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_leptos/worker/Cargo.toml'; (Get-Content 'template_leptos/Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'template_leptos/Cargo.toml'; (Get-Content 'Cargo.toml') -replace "nightshade-api = \{ version = `"$currentVersion`"", "nightshade-api = { version = `"$newVersion`"" | Set-Content 'Cargo.toml'; (Get-Content 'crates/nightshade-api/README.md') -replace "nightshade-api = `"$currentVersion`"", "nightshade-api = `"$newVersion`"" | Set-Content 'crates/nightshade-api/README.md'; (Get-Content 'template_api/Cargo.toml') -replace "nightshade-api = `"$currentVersion`"", "nightshade-api = `"$newVersion`"" | Set-Content 'template_api/Cargo.toml';(Get-Content 'apps/editor/Cargo.toml') -replace '^version = ".+"', "version = `"$newVersion`"" | Set-Content 'apps/editor/Cargo.toml'; git add Cargo.toml crates/nightshade/README.md crates/nightshade-api/README.md crates/nightshade-renderer/README.md crates/nightshade-ecs/README.md crates/nightshade-platform/README.md crates/nightshade-text/README.md crates/nightshade-paint/README.md crates/nightshade-ui/README.md template/Cargo.toml template_leptos/Cargo.toml template_leptos/worker/Cargo.toml template_api_leptos/Cargo.toml template_api_leptos/worker/Cargo.toml apps/editor/Cargo.toml apps/editor/desktop/Cargo.toml template_api/Cargo.toml; git commit -m "chore: bump version to v$newVersion"; git cliff --tag "v$newVersion" -o CHANGELOG.md; git add CHANGELOG.md; git commit -m "chore: update changelog for v$newVersion"; git tag "v$newVersion"; Write-Host ""; Write-Host "Version bumped and tagged! To push, run:" -ForegroundColor Green; Write-Host "  just push-version" -ForegroundColor Green

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
    sed -i "s/nightshade-renderer = { version = \"$CURRENT_VERSION\"/nightshade-renderer = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-platform = { version = \"$CURRENT_VERSION\"/nightshade-platform = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-text = { version = \"$CURRENT_VERSION\"/nightshade-text = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-paint = { version = \"$CURRENT_VERSION\"/nightshade-paint = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-ui = { version = \"$CURRENT_VERSION\"/nightshade-ui = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-renderer = \"$CURRENT_VERSION\"/nightshade-renderer = \"$NEW_VERSION\"/" crates/nightshade-renderer/README.md
    sed -i "s/nightshade-renderer = { version = \"$CURRENT_VERSION\"/nightshade-renderer = { version = \"$NEW_VERSION\"/" crates/nightshade-renderer/README.md
    sed -i "s/nightshade-ecs = \"$CURRENT_VERSION\"/nightshade-ecs = \"$NEW_VERSION\"/" crates/nightshade-ecs/README.md
    sed -i "s/nightshade-platform = \"$CURRENT_VERSION\"/nightshade-platform = \"$NEW_VERSION\"/" crates/nightshade-platform/README.md
    sed -i "s/nightshade-text = \"$CURRENT_VERSION\"/nightshade-text = \"$NEW_VERSION\"/" crates/nightshade-text/README.md
    sed -i "s/nightshade-paint = \"$CURRENT_VERSION\"/nightshade-paint = \"$NEW_VERSION\"/" crates/nightshade-paint/README.md
    sed -i "s/nightshade-ui = \"$CURRENT_VERSION\"/nightshade-ui = \"$NEW_VERSION\"/" crates/nightshade-ui/README.md
    sed -i "s/nightshade = \"$CURRENT_VERSION\"/nightshade = \"$NEW_VERSION\"/" crates/nightshade/README.md
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template/Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template_leptos/worker/Cargo.toml
    sed -i "s/nightshade = { version = \"$CURRENT_VERSION\"/nightshade = { version = \"$NEW_VERSION\"/" template_api_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_api_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_api_leptos/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_leptos/worker/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" template_leptos/Cargo.toml
    sed -i "s/nightshade-api = { version = \"$CURRENT_VERSION\"/nightshade-api = { version = \"$NEW_VERSION\"/" Cargo.toml
    sed -i "s/nightshade-api = \"$CURRENT_VERSION\"/nightshade-api = \"$NEW_VERSION\"/" crates/nightshade-api/README.md
    sed -i "s/nightshade-api = \"$CURRENT_VERSION\"/nightshade-api = \"$NEW_VERSION\"/" template_api/Cargo.toml
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" apps/editor/Cargo.toml
    git add Cargo.toml crates/nightshade/README.md crates/nightshade-api/README.md crates/nightshade-renderer/README.md crates/nightshade-ecs/README.md crates/nightshade-platform/README.md crates/nightshade-text/README.md crates/nightshade-paint/README.md crates/nightshade-ui/README.md template/Cargo.toml template_leptos/Cargo.toml template_leptos/worker/Cargo.toml template_api_leptos/Cargo.toml template_api_leptos/worker/Cargo.toml apps/editor/Cargo.toml apps/editor/desktop/Cargo.toml template_api/Cargo.toml
    git commit -m "chore: bump version to v$NEW_VERSION"
    git cliff --tag "v$NEW_VERSION" -o CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore: update changelog for v$NEW_VERSION"
    git tag "v$NEW_VERSION"
    echo ""
    echo "Version bumped and tagged! To push, run:"
    echo "  just push-version"

# Bumps patch version, pushes, publishes to crates.io, creates the GitHub release, and publishes the editor (Windows)
[windows]
publish-patch:
    just bump-patch-version
    just push-version
    just publish-crate
    just publish-release
    just publish-editor
    just push-template
    just push-template-leptos
    just push-template-api-leptos
    just push-template-api

# Bumps patch version, pushes, publishes to crates.io, creates the GitHub release, and publishes the editor (Unix)
[unix]
publish-patch:
    just bump-patch-version
    just push-version
    just publish-crate
    just publish-release
    just publish-editor
    just push-template
    just push-template-leptos
    just push-template-api-leptos
    just push-template-api

# Bumps minor version, pushes, publishes to crates.io, creates the GitHub release, and publishes the editor (Windows)
[windows]
publish-minor:
    just bump-minor-version
    just push-version
    just publish-crate
    just publish-release
    just publish-editor
    just push-template
    just push-template-leptos
    just push-template-api-leptos
    just push-template-api

# Bumps minor version, pushes, publishes to crates.io, creates the GitHub release, and publishes the editor (Unix)
[unix]
publish-minor:
    just bump-minor-version
    just push-version
    just publish-crate
    just publish-release
    just publish-editor
    just push-template
    just push-template-leptos
    just push-template-api-leptos
    just push-template-api

# Bumps major version, pushes, publishes to crates.io, creates the GitHub release, and publishes the editor (Windows)
[windows]
publish-major:
    just bump-major-version
    just push-version
    just publish-crate
    just publish-release
    just publish-editor
    just push-template
    just push-template-leptos
    just push-template-api-leptos
    just push-template-api

# Bumps major version, pushes, publishes to crates.io, creates the GitHub release, and publishes the editor (Unix)
[unix]
publish-major:
    just bump-major-version
    just push-version
    just publish-crate
    just publish-release
    just publish-editor
    just push-template
    just push-template-leptos
    just push-template-api-leptos
    just push-template-api

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

