set windows-shell := ["powershell.exe"]
export RUST_BACKTRACE := "1"

# Displays the list of available commands
@just:
    just --list

# Installs the tools pinned in mise.toml (rust, wasm-bindgen, wasm-opt, trunk)
init:
    mise install

# Builds the worker crate to wasm and generates web bindings into runtime/
worker:
    cargo build --release -p worker --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir runtime --out-name engine target/wasm32-unknown-unknown/release/worker.wasm
    wasm-opt -O3 --enable-simd runtime/engine_bg.wasm -o runtime/engine_bg.wasm

# Builds the worker and the Leptos app bundle
build: worker
    trunk build

# Builds the web bundle and opens the app in a native webview window
run: build
    cargo run -p desktop

# Builds the worker, then serves the app in the browser at http://127.0.0.1:8080
run-web: worker
    trunk serve

# Serves the already-built app without rebuilding the worker
serve:
    trunk serve

# Produces a production web bundle in dist
dist: worker
    trunk build --release

# Builds the standalone executable with the web bundle embedded
build-desktop: dist
    cargo build --release -p desktop

# Runs cargo check and a format check across the workspace
check:
    cargo check -p protocol -p worker -p page --target wasm32-unknown-unknown
    cargo check -p desktop
    cargo fmt --all -- --check

# Runs clippy across the workspace and denies warnings
lint:
    cargo clippy -p protocol -p worker -p page --target wasm32-unknown-unknown -- -D warnings
    cargo clippy -p desktop -- -D warnings

# Formats the code
format:
    cargo fmt --all

# Removes build artifacts (Windows)
[windows]
clean:
    cargo clean
    Remove-Item -Recurse -Force dist, runtime/engine.js, runtime/engine_bg.wasm, runtime/engine.d.ts, runtime/engine_bg.wasm.d.ts -ErrorAction SilentlyContinue

# Removes build artifacts (Unix)
[unix]
clean:
    cargo clean
    rm -rf dist runtime/engine.js runtime/engine_bg.wasm runtime/engine.d.ts runtime/engine_bg.wasm.d.ts
