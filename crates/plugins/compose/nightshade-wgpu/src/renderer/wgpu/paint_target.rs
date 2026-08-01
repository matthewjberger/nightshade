impl super::WgpuRenderer {
    pub fn set_external_paint_target(
        &mut self,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        size: (u32, u32),
    ) {
        self.paint_target = Some(super::PaintTarget {
            texture,
            view,
            size,
        });
    }

    pub fn clear_external_paint_target(&mut self) {
        self.paint_target = None;
    }
}
