---
name: reference_mesh_skinned_passes_not_mergeable
description: Only a few resources are safely shared between the mesh and skinned_mesh geometry passes, and the rest differ by format or layout
type: reference
---

The mesh and skinned_mesh constructors,
`renderer/wgpu/passes/geometry/{mesh,skinned_mesh}/pass/constructor.rs`, read as
near-duplicates, and most of what looks shared differs. The OIT reveal texture
is `R8Unorm` at window size in mesh and `R16Float` at `(1, 1)` in skinned. The
dummy IBL and BRDF textures are `Rgba16Float` in mesh and `Rgba8Unorm` in
skinned. The scene and shadow bind group layouts and the pipelines diverge as
well, so a merge of either one changes what the pass draws.

What the two share sits in `geometry/shadow_inputs.rs`: the shadow comparison
sampler (nearest plus clamp, used for both the shadow and the point shadow
samplers), the spotlight and point shadow storage buffers, and the dummy point
shadow cubemap. `geometry/material_gpu.rs` holds the shared material bind group
builders, `bindless_material_bind_group` and `rebuild_array_material_bind_group`.
Keep a helper's argument count under clippy's `too_many_arguments` limit of
seven.

The transform helpers differ too: `LocalTransform::as_matrix` normalizes the
quaternion and scales the columns, and `InstancedTransform::as_matrix` composes
a plain T\*R\*S. The two agree only for a rotation that is already normalized,
so merging them changes the instanced hot path.

On the shader side the pure BRDF math (distribution, geometry, fresnel,
clearcoat, sheen, anisotropy, iridescence, and attenuation) is
`shaders/pbr_brdf.wgsl`, which declares
`#define_import_path nightshade_wgpu::pbr_brdf`, is registered in
`shader_compose.rs`, and is imported by the mesh, skinned, and OIT shaders. The
lighting and shadow code below it is per shader, because the four shaders
diverge in the `Uniforms` struct, in the group 1 binding layout (mesh
`transforms` against skinned `joint_matrices`), and in `ObjectData`,
`VertexOutput`, and `FragmentOutput`. Sharing more of it starts with one bind
group layout for the two passes. naga_oil composes at pass construction, so a
WGSL composition error shows at run time and `cargo build` stays green: open an
app after a shader edit.
