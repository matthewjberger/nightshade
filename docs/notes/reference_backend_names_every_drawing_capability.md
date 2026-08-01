---
name: reference_backend_names_every_drawing_capability
description: Each backend names every capability that draws through its contract, and moving the frame types into nightshade-render was measured and refused
type: reference
---

# Each backend names every capability that draws

`CLAUDE.md` says no capability crate names a GPU API, so adding a backend
touches one crate. The inverse is also true and costs more.

`nightshade-wgpu` names eight capability crates and `nightshade-vulkan` names
seven. Every path they take goes through `::contract::`, so neither backend
takes any behavior:

    nightshade_sdf::contract::render_data::SdfClipmap
    nightshade_portal::contract::PortalFrame
    nightshade_decals::contract::DecalFrame
    nightshade_particles::contract::ParticleFrame
    nightshade_sprite::contract::render_data::SpriteFrame
    nightshade_acoustics::contract::AcousticsFrame
    nightshade_paint::rect_instance
    nightshade_text::contract::RenderText

So a capability that draws touches four files: both backend manifests and both
backend pass lists.

Moving the frame types down into `nightshade-render` would cut each backend
manifest from thirteen names to five. It was measured and refused. The contract
bands are not one envelope each: `nightshade-particles` is 1003 lines over four
files, `nightshade-acoustics` is 546 and `nightshade-sdf` is 343, and each
backend names emitter and grid types inside them rather than a single frame.
Moving a subset would split a type from its functions. `nightshade-decals` at 36
lines and `nightshade-sprite` at 67 would move cleanly, and a rule that holds for
two crates and breaks for three is worse than the coupling it removes.

The pass in each backend is the real cost and it cannot be removed, because a new
pass needs a pass. The manifest line was never the cost.
