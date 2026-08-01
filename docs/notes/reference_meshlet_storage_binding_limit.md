---
name: reference_meshlet_storage_binding_limit
description: wgpu's default 128MB max_storage_buffer_binding_size caps meshlet scenes at a few million unique vertices, and the meshlet feature raises it to the adapter's limit
type: reference
---

wgpu's `Limits::default` caps a single storage binding at 128 MB
(`max_storage_buffer_binding_size = 134217728`). Virtualized geometry keeps
every asset's vertices in one shared stream per attribute, so a scene with tens
of millions of unique vertices passes that in a single buffer and
`create_bind_group` fails with:

```
Buffer binding N range <bytes> exceeds `max_*_buffer_binding_size` limit 134217728
```

A scene of 27.6M unique vertices reaches it: the uv stream alone, at 8 bytes
each, is 210 MB. The hardware does not mind, and an adapter reports far more.
The cap is a conservative default.

`initialization.rs` requests `adapter_limits.max_storage_buffer_binding_size`
and `max_buffer_size`, gated on `#[cfg(feature = "meshlet")]` so builds without
it keep the conservative default.

**Why it matters:** this is invisible until a scene is genuinely large, it
reports as a validation error inside bind group creation rather than anywhere
near the streams, and instancing does not trigger it (ten thousand instances of
one asset upload once). Only *unique* geometry grows the streams. If a meshlet
scene dies at bind group creation, check the stream sizes against the adapter's
limit before suspecting the streams themselves.
