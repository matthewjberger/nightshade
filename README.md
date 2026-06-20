# template_api

A minimal starting point for a [nightshade-api](https://crates.io/crates/nightshade-api) program. Procedural, single file, runs natively and in the browser from the same code. Copy this directory to start a project.

## Run

```sh
just run       # native window
just run-web   # serve in the browser via trunk
```

`just run-web` needs the wasm target and trunk, which `just init-web` installs.

## How it works

`src/main.rs` is the whole program. It uses `run!`, the portable entry point: a setup that runs once and returns your state, then one or more per-frame systems that receive it back. The same code drives a native window and a browser canvas.

```rust
use nightshade_api::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run!(
        |world| {
            spawn_floor(world, 10.0);
            spawn_cube(world, vec3(0.0, 0.5, 0.0))
        },
        |world, cube| rotate(world, *cube, Vec3::y(), delta_time(world)),
    )
}
```

`index.html` and `Trunk.toml` are the web setup: trunk compiles the crate to wasm and serves the page, which carries the `id="canvas"` the engine renders into. Keep that id.

Grow it by adding more systems to `run!` and more `spawn_` and per-frame calls. The full surface is in the [nightshade-api docs](https://docs.rs/nightshade-api).
