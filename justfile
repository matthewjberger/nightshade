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

# Publishes the nightshade crate to crates.io
publish:
    cargo publish -p nightshade

# Runs the specified project (defaults to wgpu)
run $project="editor":
    cargo run -r -p {{project}}

# Runs the specified project with wgpu backend on native
run-wgpu $project="editor":
    cargo run -r -p {{project}} --no-default-features --features webgpu

# Build the app with wgpu + WebGL
build-webgl $project="editor":
    trunk build --features webgl --config apps/{{project}}/Trunk.toml

# Build the app with wgpu + WebGPU
build-webgpu $project="editor":
    trunk build --features webgpu --config apps/{{project}}/Trunk.toml

# Serve the app with wgpu + WebGL
run-webgl $project="editor":
    trunk serve --features webgl --config apps/{{project}}/Trunk.toml

# Serve the app with wgpu + WebGPU
run-webgpu $project="editor":
    trunk serve --features webgpu --open --config apps/{{project}}/Trunk.toml

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
watch $project="editor":
    cargo watch -x 'run -r -p {{project}}'
