---
name: reference_cross_crate_feature_forwarding_dies_on_a_merge
description: Merging crates silently kills feature gates that crossed the old boundary, and the build stays green while the behavior disappears
type: reference
---

When you move code between crates, every `#[cfg(feature = "x")]` in the moved
code now reads the NEW crate's feature list. If the old crate's feature was
turned on by a forward from somewhere else, that forward is gone and the gated
code silently becomes dead. Nothing fails to compile, and no lint fires.

**How to apply:** after any cross-crate move, for each crate list every feature
named in a `cfg` and check it against the manifest AND against who enables it:

```
rg -o 'feature = "([a-z_0-9]+)"' -r '$1' -g '*.rs' <crate>/src | sort -u
cargo tree -p <consumer> -e features | rg "<crate> feature"
```

A feature that is declared but that no path reaches is the failure. Put the
forwarding on the crate that now owns the code. Compiling proves nothing here,
because the gate compiles either way.
