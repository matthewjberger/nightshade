# Installation

Nightshade builds on stable Rust, edition 2024. It needs a GPU that speaks
Vulkan, Metal, or DirectX 12, and it needs nothing else installed.

## The toolchain

Install Rust from [rustup](https://rustup.rs), then check the version:

```bash
rustc --version
cargo --version
```

Edition 2024 needs Rust 1.85 or newer. Let chains, which the engine uses
throughout, are stable from that release.

## Adding the engine

The engine is not published. A program takes it by path, and takes each
capability it wants as a separate crate:

```toml
[dependencies]
nightshade = { path = "../nightshade/crates/nightshade" }
nightshade-window = { path = "../nightshade/crates/plugins/compose/nightshade-window" }
nightshade-wgpu = { path = "../nightshade/crates/plugins/compose/nightshade-wgpu" }
nightshade-log = { path = "../nightshade/crates/plugins/domain/nightshade-log" }
```

Those four are the smallest set that puts a window on screen. Add more as you
need them, and only as you need them.

## Choosing a backend

Two backends read the same frame description:

1. `nightshade-wgpu` runs on wgpu and works everywhere, the browser included
2. `nightshade-vulkan` runs on Vulkan 1.3 and reaches task and mesh shaders

A program composes one of them. Both carry a `RenderPlugin`, and the hook
names differ enough that importing both is a compile error rather than a quiet
clash.

## Building

```bash
cargo build --release
cargo run --release --bin my_game
```

Build in release. A debug build of a renderer is slow enough to mislead you
about the frame time of your own code.

## A checklist before you start

- [x] Rust 1.85 or newer
- [x] A GPU with a current driver
- [ ] A path dependency on the engine
- [ ] One backend composed, not two
