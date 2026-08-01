---
name: reference_lint_wasm_never_links
description: just lint-wasm runs clippy, which stops before the linker, so a native library in the graph passes every gate and breaks the web build
type: reference
---

`just lint-wasm` runs `cargo clippy --target wasm32-unknown-unknown`. Clippy
checks and stops, so `rust-lld` never runs and a link fault passes the whole
gate. The web build is where the linker first speaks, which makes
`just export-wasm <app>` the only check that a web target still links.

A crate that compiles C or C++ is the usual cause. `wasm32-unknown-unknown`
carries no C++ runtime, so an object that calls `new` or `delete` leaves
`_Znwm` and `_ZdlPv` undefined and `rust-lld` refuses the binary. The error
names the `.rlib` and the object inside it, which is how you find the crate.

The shape this engine uses is to declare such a dependency for native targets
alone and to leave the capability out of the web build:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
basis-universal = { workspace = true }
meshopt = { workspace = true }
```

`nightshade-render` declares both that way. `basis-universal` transcodes
universal textures, and the web arm of `transcode_ktx2` reports what it cannot
read. `meshopt` builds the meshlets, and `queries/meshlets` is gated off wasm
with it. Only `nightshade-vulkan` reads meshlets, and that backend never
builds for the web, so `nightshade-graphics` gates its whole `clusters` band
and the two lines in `plugin.rs` that install the cache and the build system.
A web frame then spends nothing on clusters nothing reads.
