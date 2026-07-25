use crate::paint::DrawRect;

use super::{
    GlobalUniforms, INITIAL_RECT_CAPACITY, INITIAL_TEXT_INSTANCE_CAPACITY, LayerDrawGroup,
    PaintPass, PaintRectInstance, TextDrawCall, TextInstanceData, TextSlotEntry, UiTextVertex,
};

impl PaintPass {
    fn ensure_rect_capacity(&mut self, device: &wgpu::Device, required: usize) {
        let needs_instance_grow = required > self.rect_instance_capacity;
        let needs_order_grow = required > self.rect_draw_order_capacity;
        if !needs_instance_grow && !needs_order_grow {
            return;
        }
        if needs_instance_grow {
            let new_capacity = (required * 2).max(INITIAL_RECT_CAPACITY);
            self.rect_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Rect Instance Buffer (Resized)"),
                size: (std::mem::size_of::<PaintRectInstance>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.rect_instance_capacity = new_capacity;
            self.prev_rect_instances.clear();
        }
        if needs_order_grow {
            let new_capacity = (required * 2).max(INITIAL_RECT_CAPACITY);
            self.rect_draw_order_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Rect Draw Order Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.rect_draw_order_capacity = new_capacity;
        }
        self.rect_instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PaintPass Rect Instance Bind Group (Resized)"),
            layout: &self.rect_instance_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.rect_instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.rect_draw_order_buffer.as_entire_binding(),
                },
            ],
        });
    }

    fn ensure_text_vertex_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.text_vertex_capacity {
            let new_capacity = (required * 2).max(1000);
            self.text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Text Vertex Buffer (Resized)"),
                size: (std::mem::size_of::<UiTextVertex>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.text_vertex_capacity = new_capacity;
            self.invalidate_text_slot_caches();
        }
    }

    fn ensure_text_index_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.text_index_capacity {
            let new_capacity = (required * 2).max(3000);
            self.text_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Text Index Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.text_index_capacity = new_capacity;
            self.invalidate_text_slot_caches();
        }
    }

    fn invalidate_text_slot_caches(&mut self) {
        self.text_slots.clear();
        self.text_vertex_slab = super::SlabAllocator::default();
        self.text_index_slab = super::SlabAllocator::default();
        self.char_color_slab = super::SlabAllocator::default();
        self.prev_character_colors.clear();
        self.prev_character_bg_colors.clear();
    }

    fn ensure_text_instance_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.text_instance_capacity {
            let new_capacity = (required * 2).max(INITIAL_TEXT_INSTANCE_CAPACITY);
            self.text_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Text Instance Buffer (Resized)"),
                size: (std::mem::size_of::<TextInstanceData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.text_instance_capacity = new_capacity;
            self.prev_text_instance_data.clear();
            self.rebuild_text_data_bind_group(device);
        }
    }

    fn ensure_character_color_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.character_color_capacity {
            let new_capacity = (required * 2).max(1000);
            self.character_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Character Color Buffer (Resized)"),
                size: (std::mem::size_of::<[f32; 4]>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.character_color_capacity = new_capacity;
            self.invalidate_text_slot_caches();
            self.rebuild_text_data_bind_group(device);
        }
    }

    fn ensure_character_bg_color_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.character_bg_color_capacity {
            let new_capacity = (required * 2).max(1000);
            self.character_bg_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PaintPass Character BG Color Buffer (Resized)"),
                size: (std::mem::size_of::<[f32; 4]>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.character_bg_color_capacity = new_capacity;
            self.invalidate_text_slot_caches();
            self.rebuild_text_data_bind_group(device);
        }
    }

    fn rebuild_text_data_bind_group(&mut self, device: &wgpu::Device) {
        self.text_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PaintPass Text Data Bind Group (Rebuilt)"),
            layout: &self.text_data_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.text_instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.character_color_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.character_bg_color_buffer.as_entire_binding(),
                },
            ],
        });
    }

    /// Caches the glyph atlas view and rebuilds the text font bind group.
    pub fn update_glyph_atlas(&mut self, device: &wgpu::Device, atlas_view: &wgpu::TextureView) {
        self.cached_atlas_view = Some(atlas_view.clone());
        self.rebuild_atlas_bind_group(device);
    }

    fn rebuild_atlas_bind_group(&mut self, device: &wgpu::Device) {
        let Some(atlas_view) = self.cached_atlas_view.as_ref() else {
            self.text_font_bind_groups.clear();
            return;
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PaintPass Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        self.text_font_bind_groups.clear();
        self.text_font_bind_groups
            .push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("PaintPass Glyph Atlas Bind Group"),
                layout: &self.text_font_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(atlas_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }));
    }

    /// Uploads the frame's UI rects and text into slot-stable GPU buffers,
    /// growing capacity as needed and building the per-layer draw groups.
    pub(super) fn prepare_pass(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        if let Some((width, height)) = configs.view.viewport_size {
            self.screen_width = width as f32;
            self.screen_height = height as f32;
        }

        let projection =
            nalgebra_glm::ortho(0.0, self.screen_width, self.screen_height, 0.0, -1.0, 1.0);
        let globals = GlobalUniforms { projection };
        queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[globals]),
        );

        let frame_rects = &configs.ui.rects;
        let frame_rect_entities = &configs.ui.rect_entities;
        let allocator = &configs.ui.render_slots;
        let stable_high_water = allocator.next_rect_slot;

        let mut rects_with_meta: Vec<(&DrawRect, u32)> = Vec::with_capacity(frame_rects.len());
        let mut next_frame_slot = stable_high_water;
        for (idx, rect) in frame_rects.iter().enumerate() {
            let entity = frame_rect_entities.get(idx).and_then(|opt| *opt);
            let slot = entity
                .and_then(|e| allocator.rect_slots.get(&e).copied())
                .unwrap_or_else(|| {
                    let s = next_frame_slot;
                    next_frame_slot += 1;
                    s
                });
            rects_with_meta.push((rect, slot));
        }
        rects_with_meta.sort_by(|(a, _), (b, _)| match a.layer.cmp(&b.layer) {
            std::cmp::Ordering::Equal => a.z_index.cmp(&b.z_index),
            other => other,
        });
        let rects: Vec<&DrawRect> = rects_with_meta.iter().map(|(r, _)| *r).collect();
        let total_slot_count = next_frame_slot as usize;

        if total_slot_count > 0 {
            self.ensure_rect_capacity(device, total_slot_count);
            if self.prev_rect_instances.len() < total_slot_count {
                self.prev_rect_instances
                    .resize(total_slot_count, bytemuck::Zeroable::zeroed());
            }

            for (rect, slot) in &rects_with_meta {
                let combined_z = (rect.layer as i32) * 100000 + rect.z_index;
                let depth = 0.1 + (combined_z as f32 / 10000000.0) * 0.8;
                let clip = rect
                    .clip_rect
                    .map(|c| [c.min.x, c.min.y, c.max.x, c.max.y])
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]);
                let (shadow_color, shadow_params) = match rect.shadow {
                    Some(shadow) => (
                        [
                            shadow.color.x,
                            shadow.color.y,
                            shadow.color.z,
                            shadow.color.w,
                        ],
                        [shadow.offset.x, shadow.offset.y, shadow.blur, shadow.spread],
                    ),
                    None => ([0.0; 4], [0.0; 4]),
                };
                let (quad_corner_01, quad_corner_23, is_quad) = match rect.quad_corners {
                    Some(corners) => (
                        [corners[0].x, corners[0].y, corners[1].x, corners[1].y],
                        [corners[2].x, corners[2].y, corners[3].x, corners[3].y],
                        1u32,
                    ),
                    None => ([0.0; 4], [0.0; 4], 0u32),
                };
                let instance = PaintRectInstance {
                    position_size: [rect.position.x, rect.position.y, rect.size.x, rect.size.y],
                    color: [rect.color.x, rect.color.y, rect.color.z, rect.color.w],
                    border_color: [
                        rect.border_color.x,
                        rect.border_color.y,
                        rect.border_color.z,
                        rect.border_color.w,
                    ],
                    clip_rect: clip,
                    params: [rect.corner_radius, rect.border_width, depth, rect.rotation],
                    shadow_color,
                    shadow_params,
                    effect_params: rect.effect_params,
                    quad_corner_01,
                    quad_corner_23,
                    effect_kind: rect.effect_kind,
                    is_quad,
                    _padding: [0; 2],
                };
                let slot_idx = *slot as usize;
                if self.prev_rect_instances[slot_idx] != instance {
                    queue.write_buffer(
                        &self.rect_instance_buffer,
                        (slot_idx * std::mem::size_of::<PaintRectInstance>()) as u64,
                        bytemuck::cast_slice(&[instance]),
                    );
                    self.prev_rect_instances[slot_idx] = instance;
                }
            }

            let draw_order: Vec<u32> = rects_with_meta.iter().map(|(_, slot)| *slot).collect();
            queue.write_buffer(
                &self.rect_draw_order_buffer,
                0,
                bytemuck::cast_slice(&draw_order),
            );
        }
        self.rect_count = rects_with_meta.len() as u32;

        let mut text_instance_writes: Vec<(u32, TextInstanceData)> = Vec::new();

        let allocator = &configs.ui.render_slots;
        let stable_text_high_water = allocator.next_text_slot;
        let mut next_text_frame_slot = stable_text_high_water;

        let mut sorted_text: Vec<_> = configs.ui.text_meshes.iter().collect();
        sorted_text.sort_by(|a, b| match a.layer.cmp(&b.layer) {
            std::cmp::Ordering::Equal => a.z_index.cmp(&b.z_index),
            other => other,
        });

        let mut frame_slot_assignments: Vec<u32> = Vec::with_capacity(sorted_text.len());
        let mut active_slots: std::collections::HashSet<u32> =
            std::collections::HashSet::with_capacity(sorted_text.len());
        for text_instance in &sorted_text {
            let slot = text_instance
                .entity
                .and_then(|e| allocator.text_slots.get(&e).copied())
                .unwrap_or_else(|| {
                    let s = next_text_frame_slot;
                    next_text_frame_slot += 1;
                    s
                });
            frame_slot_assignments.push(slot);
            active_slots.insert(slot);
        }

        let stale_slots: Vec<u32> = self
            .text_slots
            .keys()
            .copied()
            .filter(|s| !active_slots.contains(s))
            .collect();
        for slot in stale_slots {
            if let Some(entry) = self.text_slots.remove(&slot) {
                self.text_vertex_slab.free(entry.vertex_alloc);
                self.text_index_slab.free(entry.index_alloc);
                self.char_color_slab.free(entry.char_color_alloc);
            }
        }

        self.text_draw_calls.clear();

        for (text_instance, &slot) in sorted_text.iter().zip(frame_slot_assignments.iter()) {
            let signature = compute_text_signature(text_instance);
            let mesh = &text_instance.mesh;
            let vertex_count = mesh.vertices.len() as u32;
            let index_count = mesh.indices.len() as u32;
            let char_count = (mesh.vertices.len() / 4) as u32;

            let existing = self.text_slots.get(&slot).cloned();
            let needs_mesh_realloc = match &existing {
                None => true,
                Some(entry) => {
                    entry.signature != signature
                        || entry.vertex_alloc.capacity < vertex_count
                        || entry.index_alloc.capacity < index_count
                        || entry.char_color_alloc.capacity < char_count
                }
            };

            let entry = if needs_mesh_realloc {
                if let Some(existing) = existing {
                    if existing.vertex_alloc.capacity < vertex_count {
                        self.text_vertex_slab.free(existing.vertex_alloc);
                    }
                    if existing.index_alloc.capacity < index_count {
                        self.text_index_slab.free(existing.index_alloc);
                    }
                    if existing.char_color_alloc.capacity < char_count {
                        self.char_color_slab.free(existing.char_color_alloc);
                    }
                }
                let prev = self.text_slots.get(&slot).cloned();
                let vertex_alloc = match prev {
                    Some(ref e) if e.vertex_alloc.capacity >= vertex_count => e.vertex_alloc,
                    _ => self.text_vertex_slab.alloc(vertex_count),
                };
                let index_alloc = match prev {
                    Some(ref e) if e.index_alloc.capacity >= index_count => e.index_alloc,
                    _ => self.text_index_slab.alloc(index_count),
                };
                let char_color_alloc = match prev {
                    Some(ref e) if e.char_color_alloc.capacity >= char_count => e.char_color_alloc,
                    _ => self.char_color_slab.alloc(char_count),
                };

                let new_entry = TextSlotEntry {
                    signature,
                    vertex_alloc,
                    index_alloc,
                    index_count,
                    char_color_alloc,
                };

                if vertex_count > 0 {
                    let vertices: Vec<UiTextVertex> = mesh
                        .vertices
                        .iter()
                        .map(|vertex| UiTextVertex {
                            position: [vertex.position.x, vertex.position.y, 0.0],
                            tex_coords: [vertex.tex_coords.x, vertex.tex_coords.y],
                            character_index: vertex.character_index + char_color_alloc.start,
                            text_instance_index: slot,
                            _padding: 0,
                        })
                        .collect();
                    let needed_vertex_high = self.text_vertex_slab.high_water as usize;
                    self.ensure_text_vertex_capacity(device, needed_vertex_high);
                    queue.write_buffer(
                        &self.text_vertex_buffer,
                        (vertex_alloc.start as u64) * std::mem::size_of::<UiTextVertex>() as u64,
                        bytemuck::cast_slice(&vertices),
                    );
                }

                if index_count > 0 {
                    let indices: Vec<u32> = mesh
                        .indices
                        .iter()
                        .map(|i| i + vertex_alloc.start)
                        .collect();
                    let needed_index_high = self.text_index_slab.high_water as usize;
                    self.ensure_text_index_capacity(device, needed_index_high);
                    queue.write_buffer(
                        &self.text_index_buffer,
                        (index_alloc.start as u64) * std::mem::size_of::<u32>() as u64,
                        bytemuck::cast_slice(&indices),
                    );
                }

                self.text_slots.insert(slot, new_entry.clone());
                new_entry
            } else {
                existing.unwrap()
            };

            let char_color_high_water = self.char_color_slab.high_water as usize;
            self.ensure_character_color_capacity(device, char_color_high_water.max(1));
            self.ensure_character_bg_color_capacity(device, char_color_high_water.max(1));
            if self.prev_character_colors.len() < char_color_high_water {
                self.prev_character_colors
                    .resize(char_color_high_water, [f32::NAN; 4]);
            }
            if self.prev_character_bg_colors.len() < char_color_high_water {
                self.prev_character_bg_colors
                    .resize(char_color_high_water, [f32::NAN; 4]);
            }

            for char_index in 0..char_count as usize {
                let color = text_instance
                    .character_colors
                    .as_ref()
                    .and_then(|colors| colors.get(char_index).copied())
                    .flatten()
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]);
                let global_index = entry.char_color_alloc.start as usize + char_index;
                if self.prev_character_colors[global_index] != color {
                    queue.write_buffer(
                        &self.character_color_buffer,
                        (global_index * std::mem::size_of::<[f32; 4]>()) as u64,
                        bytemuck::cast_slice(&[color]),
                    );
                    self.prev_character_colors[global_index] = color;
                }

                let bg_color = text_instance
                    .character_background_colors
                    .as_ref()
                    .and_then(|colors| colors.get(char_index).copied())
                    .flatten()
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]);
                if self.prev_character_bg_colors[global_index] != bg_color {
                    queue.write_buffer(
                        &self.character_bg_color_buffer,
                        (global_index * std::mem::size_of::<[f32; 4]>()) as u64,
                        bytemuck::cast_slice(&[bg_color]),
                    );
                    self.prev_character_bg_colors[global_index] = bg_color;
                }
            }

            let combined_z = (text_instance.layer as i32) * 100000 + text_instance.z_index;
            let depth = 0.1 + (combined_z as f32 / 10000000.0) * 0.8;
            let clip = text_instance
                .clip_rect
                .map(|c| [c.min.x, c.min.y, c.max.x, c.max.y])
                .unwrap_or([0.0, 0.0, 0.0, 0.0]);

            text_instance_writes.push((
                slot,
                TextInstanceData {
                    color: [
                        text_instance.color.x,
                        text_instance.color.y,
                        text_instance.color.z,
                        text_instance.color.w,
                    ],
                    outline_color: [
                        text_instance.outline_color.x,
                        text_instance.outline_color.y,
                        text_instance.outline_color.z,
                        text_instance.outline_color.w,
                    ],
                    clip_rect: clip,
                    params: [text_instance.outline_width, depth, 0.0, 0.0],
                    position: [text_instance.position.x, text_instance.position.y, 0.0, 0.0],
                },
            ));

            self.text_draw_calls.push(TextDrawCall {
                index_start: entry.index_alloc.start,
                index_count: entry.index_count,
                layer: text_instance.layer,
            });
        }
        let total_text_slot_count = next_text_frame_slot as usize;

        self.layer_draw_groups.clear();
        {
            let mut rect_ranges: Vec<(u8, u32, u32)> = Vec::new();
            if !rects.is_empty() {
                let mut range_start = 0u32;
                let mut range_layer = rects[0].layer;
                for (index, rect) in rects.iter().enumerate() {
                    if rect.layer != range_layer {
                        rect_ranges.push((range_layer, range_start, index as u32 - range_start));
                        range_layer = rect.layer;
                        range_start = index as u32;
                    }
                }
                rect_ranges.push((range_layer, range_start, rects.len() as u32 - range_start));
            }

            let mut all_layers: Vec<u8> = Vec::new();
            for (layer, _, _) in &rect_ranges {
                if !all_layers.contains(layer) {
                    all_layers.push(*layer);
                }
            }
            for draw_call in &self.text_draw_calls {
                if !all_layers.contains(&draw_call.layer) {
                    all_layers.push(draw_call.layer);
                }
            }
            all_layers.sort();

            for layer in &all_layers {
                let (rect_start, rect_count) = rect_ranges
                    .iter()
                    .find(|(l, _, _)| l == layer)
                    .map(|(_, start, count)| (*start, *count))
                    .unwrap_or((0, 0));

                let text_draw_start = self
                    .text_draw_calls
                    .iter()
                    .position(|dc| dc.layer == *layer)
                    .unwrap_or(0);
                let text_draw_count = self
                    .text_draw_calls
                    .iter()
                    .filter(|dc| dc.layer == *layer)
                    .count();

                self.layer_draw_groups.push(LayerDrawGroup {
                    rect_start,
                    rect_count,
                    text_draw_start,
                    text_draw_count,
                });
            }
        }

        self.ensure_character_color_capacity(device, 1);
        self.ensure_character_bg_color_capacity(device, 1);

        if total_text_slot_count > 0 {
            self.ensure_text_instance_capacity(device, total_text_slot_count);
            if self.prev_text_instance_data.len() < total_text_slot_count {
                self.prev_text_instance_data
                    .resize(total_text_slot_count, bytemuck::Zeroable::zeroed());
            }
            for (slot, data) in &text_instance_writes {
                let slot_idx = *slot as usize;
                if self.prev_text_instance_data[slot_idx] != *data {
                    queue.write_buffer(
                        &self.text_instance_buffer,
                        (slot_idx * std::mem::size_of::<TextInstanceData>()) as u64,
                        bytemuck::cast_slice(&[*data]),
                    );
                    self.prev_text_instance_data[slot_idx] = *data;
                }
            }
        }
    }
}

fn compute_text_signature(text_instance: &crate::paint::DrawText) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text_instance.text.hash(&mut hasher);
    text_instance.font_size.to_bits().hash(&mut hasher);
    (text_instance.alignment as u8).hash(&mut hasher);
    (text_instance.vertical_alignment as u8).hash(&mut hasher);
    text_instance.monospace.hash(&mut hasher);
    text_instance
        .wrap_width
        .map(|w| w.to_bits())
        .unwrap_or(u32::MAX)
        .hash(&mut hasher);
    hasher.finish()
}
