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

# Generates and opens documentation
docs:
    cargo doc --open

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

# Runs the app natively
run:
    cargo run -r

# Build the app for WASM
build-wasm:
    trunk build --release

# Serve the app in browser
run-wasm:
    trunk serve --open

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

# Watches for changes and runs the app
watch:
    cargo watch -x 'run -r'

# Builds the project for Steam Deck using cross
build-steamdeck:
    cross build --release --target x86_64-unknown-linux-gnu

# Builds and deploys the project to Steam Deck
deploy-steamdeck:
    just build-steamdeck
    scp ./target/x86_64-unknown-linux-gnu/release/nightshade-template deck@steamdeck.local:~/Downloads
