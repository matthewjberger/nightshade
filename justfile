set windows-shell := ["powershell.exe"]
export RUST_LOG := "info,wgpu_core=off"
export RUST_BACKTRACE := "1"

# Show the available commands
@just:
    just --list

# Run the app natively
run:
    cargo run -r

# Serve the app in the browser
run-web:
    trunk serve --release --open

# Build the app natively
build:
    cargo build -r

# Build the app for the web
build-web:
    trunk build --release

# Format and lint
check:
    cargo fmt --all
    cargo clippy --all-targets -- -D warnings

# Install the wasm target and trunk
init-web:
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk
