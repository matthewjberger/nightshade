# nightshade-wgpu-paint

Draws the `PaintFrame` that the retained interface publishes into `nightshade-paint`.

The rectangles, the text and the images draw in the swapchain stage over the composed scene. The text reads the glyph atlas the backend publishes, so a glyph that arrives mid frame reaches the draw without a second upload.

Compose `PaintRenderPlugin` beside a backend to turn the interface on.
