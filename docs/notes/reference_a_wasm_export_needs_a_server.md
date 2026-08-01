---
name: reference_a_wasm_export_needs_a_server
description: An exported web build plays over HTTP and shows a blank page from a file:// address, so just serve-export exists to give it an address
type: reference
---

`just export-wasm <app> [directory]` writes a whole static site into
`web/<app>`, and `just export-wasm-all` writes the whitelisted apps into `web/`
under a landing page. The tree is the one GitHub Pages wants: an `index.html`,
one JavaScript file, one `.wasm` file, and `icon.png`. Almost every asset an
app reads is inside the binary through `include_bytes!`, so the tree is the
whole game. The apps under `apps/rendering/` read their Khronos model over the
network at run time, so those pages want an internet connection as well.

**Opening `index.html` from the disk shows a blank page.** A browser refuses an
ES module and a `.wasm` fetch from a `file://` address, whatever the page says,
so the export cannot be checked by opening the file. `just serve-export
[directory] [port]` serves the tree on `127.0.0.1` and opens a browser at it.
It is a listener with its own media type table, because the type of a `.wasm`
file must be `application/wasm` or `WebAssembly.instantiateStreaming` refuses
it.

**The export builds with `--public-url ./`.** Trunk writes `<base href="/">` by
default, which only plays at the root of a domain. `./` writes relative paths,
so the same tree plays at a domain root, in a GitHub Pages subdirectory, and
under `serve-export`. Each directory also gets an empty `.nojekyll`, because
GitHub Pages otherwise hides files that start with an underscore.

The whitelist for `export-wasm-all` is the `export_apps` variable at the top of
the justfile. That recipe owns `web/` and clears it first, so the landing page
lists what is there and nothing else.
