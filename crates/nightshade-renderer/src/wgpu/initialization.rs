use super::brdf_lut;
use super::glyph_atlas;
use super::passes;
use super::rendergraph;
use crate::wgpu::rendergraph::{
    render_graph_add_color_texture, render_graph_add_depth_texture, render_graph_add_pass,
};

pub(super) fn pick_present_mode(
    modes: &[wgpu::PresentMode],
    vsync_enabled: bool,
) -> wgpu::PresentMode {
    if vsync_enabled {
        return wgpu::PresentMode::Fifo;
    }
    let preferred: &[wgpu::PresentMode] = if cfg!(target_os = "macos") {
        &[wgpu::PresentMode::Immediate, wgpu::PresentMode::FifoRelaxed]
    } else {
        &[wgpu::PresentMode::Mailbox, wgpu::PresentMode::FifoRelaxed]
    };
    for candidate in preferred.iter().copied() {
        if modes.contains(&candidate) {
            return candidate;
        }
    }
    wgpu::PresentMode::Fifo
}

impl super::WgpuRenderer {
    pub async fn new_async<W>(
        window_handle: W,
        initial_width: u32,
        initial_height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        W: Into<wgpu::SurfaceTarget<'static>>,
    {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(window_handle)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request adapter!");

        let adapter_info = adapter.get_info();

        let mut required_features = wgpu::Features::INDIRECT_FIRST_INSTANCE;

        if !cfg!(target_os = "macos") && !cfg!(target_arch = "wasm32") {
            required_features |= wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
        }

        let adapter_features = adapter.features();
        let adapter_limits = adapter.limits();

        const BINDLESS_MAX_TEXTURES: u32 = 4096;
        const BINDLESS_MIN_TEXTURES: u32 = 512;
        let bindless_features = wgpu::Features::TEXTURE_BINDING_ARRAY
            | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY;
        let bindless_capacity = adapter_limits
            .max_binding_array_elements_per_shader_stage
            .min(BINDLESS_MAX_TEXTURES);
        let bindless_enabled = !cfg!(target_arch = "wasm32")
            && adapter_features.contains(bindless_features)
            && bindless_capacity >= BINDLESS_MIN_TEXTURES;
        if bindless_enabled {
            required_features |= bindless_features;
        }
        let bindless_config = crate::wgpu::material_texture_arrays::BindlessConfig {
            enabled: bindless_enabled,
            max_textures: bindless_capacity,
        };

        let base_limits = if adapter_info.backend == wgpu::Backend::Gl {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };
        let mut required_limits = base_limits.using_resolution(adapter_limits.clone());
        required_limits.max_bind_groups = if cfg!(target_arch = "wasm32") { 4 } else { 8 };
        required_limits.max_sampled_textures_per_shader_stage =
            adapter_limits.max_sampled_textures_per_shader_stage.min(64);
        required_limits.max_samplers_per_shader_stage =
            adapter_limits.max_samplers_per_shader_stage.min(64);
        if bindless_enabled {
            required_limits.max_binding_array_elements_per_shader_stage = bindless_capacity;
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("WGPU Device"),
                memory_hints: wgpu::MemoryHints::default(),
                required_features,
                required_limits,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        tracing::info!("wgpu device features: {:?}", device.features());

        let gpu_profile = build_gpu_profile(&adapter_info);
        tracing::info!(
            "gpu adapter: {} ({:?} / {:?})",
            gpu_profile.name,
            gpu_profile.backend,
            gpu_profile.device_type
        );

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let present_mode = pick_present_mode(&surface_capabilities.present_modes, true);
        tracing::info!(
            "Surface present modes available: {:?}, selected: {:?}",
            surface_capabilities.present_modes,
            present_mode
        );
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_width,
            height: initial_height,
            present_mode,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let depth_format = wgpu::TextureFormat::Depth32Float;
        let hdr_format = wgpu::TextureFormat::Rgba16Float;
        let sky_pass = passes::SkyPass::new(&device, hdr_format, depth_format);
        let grid_pass = passes::GridPass::new(&device, hdr_format, depth_format);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let shadow_depth_pass = passes::ShadowDepthPass::new(&device, &queue).await;

        let material_texture_arrays = if bindless_config.enabled {
            crate::wgpu::material_texture_arrays::MaterialTextureArrays::new_bindless(
                &device,
                &queue,
                bindless_config.max_textures,
            )
        } else {
            crate::wgpu::material_texture_arrays::MaterialTextureArrays::new(&device)
        };
        let mip_generator = crate::wgpu::mip_generator::MipGenerator::new(&device);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let mut mesh_pass = passes::MeshPass::new(
            &device,
            &queue,
            hdr_format,
            depth_format,
            (initial_width, initial_height),
            bindless_config,
        )
        .await;
        mesh_pass.apply_material_textures(&device, &material_texture_arrays);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let mut skinned_mesh_pass = passes::SkinnedMeshPass::new(
            &device,
            &queue,
            hdr_format,
            depth_format,
            bindless_config,
        )
        .await;
        skinned_mesh_pass.apply_material_textures(&device, &material_texture_arrays);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let lines_pass = passes::LinesPass::new(&device, hdr_format);

        let particle_pass = passes::ParticlePass::new(&device, hdr_format);

        let text_pass = passes::TextPass::new(&device, hdr_format, depth_format);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let cloth_pass = passes::ClothPass::new(&device);

        #[cfg(feature = "terrain")]
        let terrain_pass = passes::TerrainPass::new(&device, hdr_format);
        #[cfg(feature = "grass")]
        let grass_pass = passes::GrassPass::new(&device, hdr_format);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let ui_texture_array = crate::wgpu::ui_texture_array::UiTextureArray::new(&device);
        let mut ui_image_pass = passes::UiImagePass::new(&device, surface_format);
        ui_image_pass.set_texture_bind_group(
            &device,
            &ui_texture_array.view,
            &ui_texture_array.sampler,
        );

        #[cfg(feature = "debug_render")]
        let selection_mask_pass = passes::SelectionMaskPass::new(&device, depth_format);

        let mut graph = rendergraph::render_graph_new();

        let depth_id = render_graph_add_depth_texture(&mut graph, "depth")
            .size(initial_width.max(1), initial_height.max(1))
            .usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .clear_depth(0.0)
            .transient();

        let scene_color_id = render_graph_add_color_texture(&mut graph, "scene_color")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .clear_color(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })
            .transient();

        let compute_output_id = render_graph_add_color_texture(&mut graph, "compute_output")
            .format(surface_format)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let swapchain_id = render_graph_add_color_texture(&mut graph, "swapchain")
            .format(surface_format)
            .external();

        let viewport_resource_id = render_graph_add_color_texture(&mut graph, "viewport_output")
            .format(surface_format)
            .external();

        let shadow_map_size = passes::CASCADE_ATLAS_SIZE;
        let shadow_depth_id = render_graph_add_depth_texture(&mut graph, "shadow_depth")
            .size(shadow_map_size, shadow_map_size)
            .format(wgpu::TextureFormat::Depth32Float)
            .clear_depth(0.0)
            .fixed_size()
            .transient();

        let spotlight_shadow_atlas_size = if cfg!(target_arch = "wasm32") {
            1024
        } else {
            4096
        };
        let spotlight_shadow_atlas_id =
            render_graph_add_depth_texture(&mut graph, "spotlight_shadow_atlas")
                .format(wgpu::TextureFormat::Depth32Float)
                .external();
        let spotlight_shadow_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spotlight Shadow Atlas"),
            size: wgpu::Extent3d {
                width: spotlight_shadow_atlas_size,
                height: spotlight_shadow_atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let spotlight_shadow_atlas_view =
            spotlight_shadow_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        #[cfg(feature = "debug_render")]
        let selection_mask_id = render_graph_add_color_texture(&mut graph, "selection_mask")
            .format(wgpu::TextureFormat::R8Unorm)
            .size(initial_width.max(1), initial_height.max(1))
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let entity_id_id = render_graph_add_color_texture(&mut graph, "entity_id")
            .format(wgpu::TextureFormat::R32Float)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let view_normals_id = render_graph_add_color_texture(&mut graph, "view_normals")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let velocity_id = render_graph_add_color_texture(&mut graph, "velocity")
            .format(wgpu::TextureFormat::Rg16Float)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color {
                r: 2.0,
                g: 2.0,
                b: 0.0,
                a: 0.0,
            })
            .transient();

        let ssao_half_width = (initial_width / 2).max(1);
        let ssao_half_height = (initial_height / 2).max(1);

        let ssao_raw_id = render_graph_add_color_texture(&mut graph, "ssao_raw")
            .format(wgpu::TextureFormat::R8Unorm)
            .size(ssao_half_width, ssao_half_height)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::WHITE)
            .transient();

        let ssao_id = render_graph_add_color_texture(&mut graph, "ssao")
            .format(wgpu::TextureFormat::R8Unorm)
            .size(ssao_half_width, ssao_half_height)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::WHITE)
            .transient();

        let ssgi_half_width = (initial_width / 2).max(1);
        let ssgi_half_height = (initial_height / 2).max(1);

        let ssgi_raw_id = render_graph_add_color_texture(&mut graph, "ssgi_raw")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(ssgi_half_width, ssgi_half_height)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let ssgi_id = render_graph_add_color_texture(&mut graph, "ssgi")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(ssgi_half_width, ssgi_half_height)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let ssr_half_width = (initial_width / 2).max(1);
        let ssr_half_height = (initial_height / 2).max(1);

        let ssr_raw_id = render_graph_add_color_texture(&mut graph, "ssr_raw")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(ssr_half_width, ssr_half_height)
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let ssr_id = render_graph_add_color_texture(&mut graph, "ssr")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let ui_depth_id = render_graph_add_depth_texture(&mut graph, "ui_depth")
            .size(initial_width.max(1), initial_height.max(1))
            .format(wgpu::TextureFormat::Depth32Float)
            .clear_depth(0.0)
            .transient();

        render_graph_add_pass(&mut graph, Box::new(sky_pass), &[("color", scene_color_id)])?;

        render_graph_add_pass(
            &mut graph,
            Box::new(shadow_depth_pass),
            &[
                ("shadow_depth", shadow_depth_id),
                ("spotlight_shadow_atlas", spotlight_shadow_atlas_id),
            ],
        )?;

        // Terrain and grass are opaque ground and must populate scene color and
        // depth before the mesh passes composite their transparency. Otherwise a
        // transparent surface in front of the ground composites over the bare sky
        // and its depth prepass occludes the ground that renders afterward.
        #[cfg(feature = "terrain")]
        render_graph_add_pass(
            &mut graph,
            Box::new(terrain_pass),
            &[
                ("color", scene_color_id),
                ("depth", depth_id),
                ("entity_id", entity_id_id),
                ("shadow_depth", shadow_depth_id),
            ],
        )?;

        #[cfg(feature = "grass")]
        render_graph_add_pass(
            &mut graph,
            Box::new(grass_pass),
            &[("color", scene_color_id), ("depth", depth_id)],
        )?;

        render_graph_add_pass(
            &mut graph,
            Box::new(mesh_pass),
            &[
                ("color", scene_color_id),
                ("depth", depth_id),
                ("shadow_depth", shadow_depth_id),
                ("spotlight_shadow_atlas", spotlight_shadow_atlas_id),
                ("entity_id", entity_id_id),
                ("view_normals", view_normals_id),
                ("velocity", velocity_id),
            ],
        )?;

        // Meshlet geometry rasterizes after the mesh passes have laid down
        // depth, so its visibility buffer only keeps pixels that survive against
        // the rest of the scene, and writes depth of its own so the passes that
        // follow occlude against it.
        #[cfg(feature = "meshlet")]
        {
            let meshlet_visibility_id =
                render_graph_add_color_texture(&mut graph, "meshlet_visibility")
                    .format(wgpu::TextureFormat::R32Uint)
                    .size(initial_width.max(1), initial_height.max(1))
                    .usage(
                        wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                    )
                    .clear_color(wgpu::Color {
                        r: passes::geometry::meshlet::MESHLET_VISIBILITY_BUFFER_EMPTY as f64,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    })
                    .force_store()
                    .transient();

            let meshlet_pass = passes::MeshletPass::new(
                &device,
                hdr_format,
                depth_format,
                wgpu::TextureFormat::R32Uint,
            );
            render_graph_add_pass(
                &mut graph,
                Box::new(meshlet_pass),
                &[
                    ("meshlet_visibility", meshlet_visibility_id),
                    ("color", scene_color_id),
                    ("depth", depth_id),
                ],
            )?;
        }

        render_graph_add_pass(
            &mut graph,
            Box::new(skinned_mesh_pass),
            &[
                ("color", scene_color_id),
                ("depth", depth_id),
                ("shadow_depth", shadow_depth_id),
                ("spotlight_shadow_atlas", spotlight_shadow_atlas_id),
                ("entity_id", entity_id_id),
                ("velocity", velocity_id),
            ],
        )?;

        render_graph_add_pass(
            &mut graph,
            Box::new(passes::DecalPass::new(&device, hdr_format)),
            &[("color", scene_color_id), ("depth", depth_id)],
        )?;

        render_graph_add_pass(
            &mut graph,
            Box::new(passes::WaterPass::new(&device, hdr_format)),
            &[
                ("color", scene_color_id),
                ("depth", depth_id),
                ("velocity", velocity_id),
            ],
        )?;

        let scene_overlay_pass = passes::SceneOverlayPass {
            grid: grid_pass,
            lines: lines_pass,
            particles: particle_pass,
            text: text_pass,
        };
        render_graph_add_pass(
            &mut graph,
            Box::new(scene_overlay_pass),
            &[("color", scene_color_id), ("depth", depth_id)],
        )?;

        render_graph_add_pass(
            &mut graph,
            Box::new(cloth_pass),
            &[("color", scene_color_id), ("depth", depth_id)],
        )?;

        #[cfg(feature = "debug_render")]
        {
            render_graph_add_pass(
                &mut graph,
                Box::new(selection_mask_pass),
                &[
                    ("selection_mask", selection_mask_id),
                    ("entity_id", entity_id_id),
                ],
            )?;

            let outline_pass = passes::OutlinePass::new(&device, hdr_format);
            render_graph_add_pass(
                &mut graph,
                Box::new(outline_pass),
                &[
                    ("selection_mask", selection_mask_id),
                    ("color", scene_color_id),
                ],
            )?;
        }

        let bloom_texture_id = render_graph_add_color_texture(&mut graph, "bloom")
            .format(hdr_format)
            .size((initial_width / 2).max(1), (initial_height / 2).max(1))
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let dof_output_id = render_graph_add_color_texture(&mut graph, "dof_output")
            .format(hdr_format)
            .size(initial_width.max(1), initial_height.max(1))
            .clear_color(wgpu::Color::BLACK)
            .transient();

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let bloom_pass =
            passes::BloomPass::new(&device, initial_width.max(1), initial_height.max(1));
        render_graph_add_pass(
            &mut graph,
            Box::new(bloom_pass),
            &[("hdr", scene_color_id), ("bloom", bloom_texture_id)],
        )?;

        let dof_pass = passes::DepthOfFieldPass::new(
            &device,
            hdr_format,
            initial_width.max(1),
            initial_height.max(1),
        );
        render_graph_add_pass(
            &mut graph,
            Box::new(dof_pass),
            &[
                ("hdr", scene_color_id),
                ("depth", depth_id),
                ("dof_output", dof_output_id),
            ],
        )?;

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let ssao_pass = passes::SsaoPass::new(&device);
        render_graph_add_pass(
            &mut graph,
            Box::new(ssao_pass),
            &[
                ("depth", depth_id),
                ("view_normals", view_normals_id),
                ("ssao_raw", ssao_raw_id),
            ],
        )?;

        let ssao_blur_pass = passes::SsaoBlurPass::new(&device);
        render_graph_add_pass(
            &mut graph,
            Box::new(ssao_blur_pass),
            &[
                ("ssao_raw", ssao_raw_id),
                ("depth", depth_id),
                ("view_normals", view_normals_id),
                ("ssao", ssao_id),
            ],
        )?;

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let postprocess_pass = passes::PostProcessPass::new(&device, surface_format, 1.0);
        render_graph_add_pass(
            &mut graph,
            Box::new(postprocess_pass),
            &[
                ("hdr", dof_output_id),
                ("scene_color", scene_color_id),
                ("bloom", bloom_texture_id),
                ("ssao", ssao_id),
                ("output", compute_output_id),
            ],
        )?;

        let aa_output_id = render_graph_add_color_texture(&mut graph, "aa_output")
            .format(surface_format)
            .size(initial_width.max(1), initial_height.max(1))
            .clear_color(wgpu::Color::BLACK)
            .transient();

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let taa_pass = passes::TaaPass::new(&device, surface_format);
        render_graph_add_pass(
            &mut graph,
            Box::new(taa_pass),
            &[
                ("input", compute_output_id),
                ("depth", depth_id),
                ("velocity", velocity_id),
                ("output", aa_output_id),
            ],
        )?;

        let taa_passthrough_pass =
            passes::BlitPass::new(&device, surface_format).with_name("taa_passthrough");
        render_graph_add_pass(
            &mut graph,
            Box::new(taa_passthrough_pass),
            &[("input", compute_output_id), ("output", aa_output_id)],
        )?;

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let (brdf_lut_texture, brdf_lut_view) = brdf_lut::generate_brdf_lut(&device, &queue);

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let depth_pick_shader = crate::wgpu::shader_compose::compile_wgsl(
            &device,
            "Depth Pick Shader",
            include_str!("shaders/depth_pick.wgsl"),
        );

        let depth_pick_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Depth Pick Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let depth_pick_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Depth Pick Pipeline Layout"),
                bind_group_layouts: &[Some(&depth_pick_bind_group_layout)],
                immediate_size: 0,
            });

        let depth_pick_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Depth Pick Compute Pipeline"),
                layout: Some(&depth_pick_pipeline_layout),
                module: &depth_pick_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let depth_pick_buffer_size =
            (super::DEPTH_PICK_SAMPLE_SIZE * super::DEPTH_PICK_SAMPLE_SIZE * 8) as u64;
        let depth_pick_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Depth Pick Storage Buffer"),
            size: depth_pick_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let depth_pick_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Depth Pick Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_pick_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Depth Pick Staging Buffer"),
            size: depth_pick_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        #[cfg(not(target_arch = "wasm32"))]
        let screenshot_staging_buffer = {
            let bytes_per_pixel = 4u32;
            let unpadded_bytes_per_row = initial_width.max(1) * bytes_per_pixel;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
            let buffer_size = (padded_bytes_per_row * initial_height.max(1)) as u64;
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Screenshot Staging Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };

        crate::wgpu::stagger::yield_to_event_loop(&queue).await;
        let glyph_atlas = glyph_atlas::GlyphAtlas::new(&device);
        crate::wgpu::stagger::drain_queued_work(&queue).await;

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            surface_format,
            supported_present_modes: surface_capabilities.present_modes,
            graph,
            targets: super::RenderTargets {
                depth: depth_id,
                scene_color: scene_color_id,
                compute_output: compute_output_id,
                aa_output: aa_output_id,
                swapchain: swapchain_id,
                viewport_resource: viewport_resource_id,
                ui_depth: ui_depth_id,
                entity_id: entity_id_id,
                view_normals: view_normals_id,
                velocity: velocity_id,
                ssao_raw: ssao_raw_id,
                ssao: ssao_id,
                ssgi_raw: ssgi_raw_id,
                ssgi: ssgi_id,
                ssr_raw: ssr_raw_id,
                ssr: ssr_id,
                spotlight_shadow_atlas: spotlight_shadow_atlas_id,
            },
            spotlight_shadow_atlas_texture,
            spotlight_shadow_atlas_view,
            ui_image_pass: Some(Box::new(ui_image_pass)),
            glyph_atlas,
            camera_viewports: std::collections::HashMap::new(),
            _brdf_lut_texture: brdf_lut_texture,
            brdf_lut_view,
            material_texture_arrays,
            ui_texture_array,
            mip_generator,
            depth_pick: super::DepthPickState {
                compute_pipeline: depth_pick_compute_pipeline,
                bind_group_layout: depth_pick_bind_group_layout,
                storage_buffer: depth_pick_storage_buffer,
                uniform_buffer: depth_pick_uniform_buffer,
                staging_buffer: depth_pick_staging_buffer,
                bind_group: None,
                pending: false,
                map_complete: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                center: (0, 0),
                texture_size: (1, 1),
                camera: None,
            },
            #[cfg(not(target_arch = "wasm32"))]
            screenshot: super::ScreenshotState {
                staging_buffer: screenshot_staging_buffer,
                pending: false,
                map_complete: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                path: None,
                width: initial_width.max(1),
                height: initial_height.max(1),
                max_dimension: None,
            },
            render_buffer_size: (initial_width.max(1), initial_height.max(1)),
            window_render_state: super::WindowRenderState::default(),
            frame_state: super::FrameState {
                index: 0,
                last_settings_signature: None,
                text_mesh_signatures: std::collections::HashMap::new(),
                captured_ibl_atmosphere: None,
                captured_ibl_hour: 0.0,
                captured_day_night_snapshots: false,
            },
            gpu_profile,
            ibl_views: crate::config::IblViews::default(),
        })
    }
}

/// Translates the selected adapter's `wgpu::AdapterInfo` into the engine's
/// backend-agnostic `GpuProfile` so apps can read it without depending on wgpu.
fn build_gpu_profile(info: &wgpu::AdapterInfo) -> crate::config::GpuProfile {
    use crate::config::{GpuBackend, GpuDeviceType, GpuProfile};

    let backend = match info.backend {
        wgpu::Backend::Vulkan => GpuBackend::Vulkan,
        wgpu::Backend::Metal => GpuBackend::Metal,
        wgpu::Backend::Dx12 => GpuBackend::Dx12,
        wgpu::Backend::Gl => {
            if cfg!(target_arch = "wasm32") {
                GpuBackend::WebGl
            } else {
                GpuBackend::Gl
            }
        }
        wgpu::Backend::BrowserWebGpu => GpuBackend::WebGpu,
        _ => GpuBackend::Other,
    };
    let device_type = match info.device_type {
        wgpu::DeviceType::IntegratedGpu => GpuDeviceType::IntegratedGpu,
        wgpu::DeviceType::DiscreteGpu => GpuDeviceType::DiscreteGpu,
        wgpu::DeviceType::VirtualGpu => GpuDeviceType::VirtualGpu,
        wgpu::DeviceType::Cpu => GpuDeviceType::Cpu,
        wgpu::DeviceType::Other => GpuDeviceType::Other,
    };
    GpuProfile {
        backend,
        device_type,
        name: info.name.clone(),
    }
}
