# Nightshade

`Nightshade` is a data-oriented game engine written in rust 🦀

## Quickstart

```bash
# native (wgpu by default)
just run

# native (vulkan)
just run-vulkan

# webgpu
just run-webgpu

# webgl
just run-webgl
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support wgpu.
> Firefox also [supports wgpu](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)

> Run `just` with no arguments to list all commands

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.