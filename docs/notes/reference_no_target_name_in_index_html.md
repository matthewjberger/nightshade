---
name: reference_no_target_name_in_index_html
description: A web app's index.html must carry data-bin=<package>-run, because the package emits both a cdylib and a binary and trunk cannot choose on its own
type: reference
---

Every app page carries `data-bin="<package>-run"` on its `rel="rust"` link.
Every app package emits two wasm artifacts, the `cdylib` the android loader
needs and the `<package>-run` binary, so trunk 0.21 stops with `found more
than one target artifact` unless the page names one. The binary is the web
entry, because `main` calls `launch` and no library carries a wasm start.

**Why:** a page that names no target builds nothing, and the failure reads as
`found more than one target artifact`. Naming it with `data-bin` rather than
`data-target-name` makes cargo reject a wrong name, instead of trunk reporting
`cargo artifacts not found` with the `.wasm` sitting on disk.

**How to apply:** adding a web app means copying `Trunk.toml` and
`index.html`, changing the `<title>`, and setting `data-bin` to that
package's binary. Verify with `trunk build --config <app>/Trunk.toml`, which
reaches the artifact choice even without `--release`.
