# A contract feature that gates nothing hides a behavior edge

A plugin crate that needs another takes it with `default-features = false` and
`features = ["contract"]`, so it gets the types and never the other
capability's behavior. That line is the whole promise. Nothing checks that the
crate on the other end keeps it.

`nightshade-graphics` declared `contract = []` and gated only `plugin` and
`systems` in its crate root. Every other module stayed open, so its "contract"
was the whole crate minus one struct, and four crates wrote the approved
manifest line while calling its registry commands. `nightshade-physics` built
meshes and materials in the renderer from inside a collider spawner, and
`dag-check` could not see it: the edge is not a `plugin` edge, so the ledger
never hears about it, and the crate is not taken whole, so the stray-whole-crate
check skips it.

The cause was placement rather than the feature. `MaterialRegistry`, `MeshCache`
and `TextureCache` are `nightshade-render` resources, and the functions that
change them sat in `nightshade-graphics`. A type lives with its functions, so
those modules moved down into `nightshade-render/src/commands/`, which every
crate may name because the description is all contract already. Physics and
openxr then named graphics nowhere and dropped the dependency.

Two checks hold it now. `dag-check` fails on a crate that declares a `contract`
feature its crate root gates nothing behind, and on a contract edge whose
consumer names a module the other crate keeps behind its `plugin`. When a
capability crate wants to call another, the answer is almost always that the
function belongs beside the data it changes, one level down.
