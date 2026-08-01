# A nightshade web build

The wasm, the loader, and the assets of a nightshade app, built and ready to
host. There is no source here and nothing to compile.

A browser refuses an ES module and a wasm fetch from a file address, so opening
`index.html` from the disk shows a blank page. The page plays from any HTTP
address.

## Publish it

1. Make a repository and push this directory to `main`.
2. Open Settings, then Pages, and set the source to GitHub Actions.

`.github/workflows/pages.yml` copies this directory into `_site`, leaving out
the git data and itself, and deploys that on every push to `main`. It compiles
nothing. Every path in the page is relative, so the page plays at the root of a
domain and in a subdirectory alike.

## Look at it here first

Any static server works. From the nightshade repository, `just serve-export
<directory>` opens one on localhost with the media types a wasm page needs.
