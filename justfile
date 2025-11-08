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
    cargo clippy --target wasm32-unknown-unknown -- -D warnings -A clippy::arc-with-non-send-sync

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
