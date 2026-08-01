---
name: reference_compute_storage_buffer_limit
description: Compute bind groups cap at 8 storage buffers on the WebGPU and wasm baseline, so pack or merge rather than adding a ninth
type: reference
---

A single compute shader stage can bind at most **8 storage buffers** (`max_storage_buffers_per_shader_stage`). This is the WebGPU/wasm baseline, so raising the requested device limit is not an option (it would break every browser build). Native panics at `create_bind_group_layout` with "Too many bindings of type StorageBuffers... limit is 8" if you exceed it.

When a compute pass needs more logical buffers than 8, **pack/merge** rather than add bindings. Real example: the skinned animation compute (`skinned_mesh/animation.rs` + `animation_compute.wgsl`) needs skeletons, bones, channels, times, values, bone_transforms(rw), plus per-bone clip references and per-clip (time, weight) runtime. Base-pose active clips and masked layers share one `bone_clips` buffer (each is `{channel_start, channel_count, runtime_index, pad}`) and one `clip_runtime` (`vec2<f32>` = time, weight) buffer, keeping the total at exactly 8. 
