# nightshade-wgpu-screenspace

Screen space ambient occlusion, global illumination and reflections.

Each effect reads the scene depth and the view normals, writes a half resolution buffer, and a bilateral blur widens it without crossing an edge. The tone map reads the three results as optional inputs, so a program that declines this crate gets no occlusion, no bounced light and no reflections rather than a fault.

Compose `ScreenSpaceRenderPlugin` beside a backend to turn them on.
