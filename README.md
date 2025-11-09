<p align="center">
  <img src="crates/nightshade/images/logo.png" alt="nightshade" width="400">
</p>

<p align="center">
  <a href="https://github.com/matthewjberger/nightshade"><img alt="github" src="https://img.shields.io/badge/github-matthewjberger/nightshade-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://crates.io/crates/nightshade"><img alt="crates.io" src="https://img.shields.io/crates/v/nightshade.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>
  <a href="https://docs.rs/nightshade"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-nightshade-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20"></a>
</p>

`Nightshade` is a data-oriented game engine written in rust 🦀

## Getting Started

The fastest way to get started is with the [nightshade-template](https://github.com/matthewjberger/nightshade-template).

## Quickstart

```bash
# native
just run

# wasm (webgpu)
just run-wasm

# serve site
cd site && just serve
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)
* [git-cliff](https://git-cliff.org/) (for changelog generation)
* [gh](https://cli.github.com/) (GitHub CLI, for creating releases)

> Run `just` with no arguments to list all commands

## Developer Notes

### Release Workflow

This project uses [git-cliff](https://git-cliff.org/) for automated changelog generation. The changelog is maintained in [CHANGELOG.md](CHANGELOG.md).

**To create a new release:**

1. Bump the version (patch version):
   ```bash
   just bump-minor-version
   ```
   Or for a major version bump:
   ```bash
   just bump-major-version
   ```
   This will:
   - Update version in `Cargo.toml` and `crates/nightshade/README.md`
   - Create a commit with the version bump
   - Create a git tag `vX.Y.Z`
   - Generate/update `CHANGELOG.md`
   - Create a commit with the changelog

2. Push the tag and commits:
   ```bash
   just push-version vX.Y.Z
   ```

3. Create a GitHub release:
   ```bash
   just publish-release
   ```
   This creates a GitHub release titled `nightshade-vX.Y.Z` with the full changelog as release notes.

**Utility commands:**
- `just show-version` - Display current version
- `just show-tags` - Show tags for current commit
- `just strip-tag vX.Y.Z` - Delete a tag locally and remotely
- `just changelog` - Manually generate/update changelog

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.
