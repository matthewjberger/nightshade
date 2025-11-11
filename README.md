# Nightshade Template

A template for creating applications with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

## Quickstart

```bash
# native
just run

# wasm (webgpu)
just run-wasm
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)
* [cross](https://github.com/cross-rs/cross) (for Steam Deck builds)
  * Requires Docker (macOS/Linux) or Docker Desktop (Windows)

> Run `just` with no arguments to list all commands

## Steam Deck Deployment

Deploy to Steam Deck using `just deploy-steamdeck`. First-time setup on Steam Deck (must be in desktop mode):

1. Set password for `deck` user: `passwd`
2. Enable SSH: `sudo systemctl enable sshd && sudo systemctl start sshd`
3. Deploy the binary: `just deploy-steamdeck`
4. Add `~/Downloads/nightshade-template` as a non-steam game in Steam
5. Launch from Big Picture mode or Game mode after initial setup
6. Future deploys must be done from desktop mode, but the last deployed binary will run in game mode

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.
