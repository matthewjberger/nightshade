use super::brdf_lut;
use super::glyph_atlas;
use super::passes;
use super::rendergraph;
use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{
    render_graph_add_color_texture, render_graph_add_depth_texture, render_graph_add_pass,
};
use crate::renderer::wgpu::shader_compose::ShaderComposer;

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

pub fn pick_texture_format(
    adapter: &wgpu::Adapter,
    candidates: &[wgpu::TextureFormat],
    usage: wgpu::TextureUsages,
) -> wgpu::TextureFormat {
    candidates
        .iter()
        .copied()
        .find(|candidate| {
            adapter
                .get_texture_format_features(*candidate)
                .allowed_usages
                .contains(usage)
        })
        .unwrap_or_else(|| candidates[candidates.len() - 1])
}

pub struct DeviceConfig {
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits,
    pub bindless: crate::renderer::wgpu::caps::BindlessConfig,
}

pub fn negotiate_device_config(
    adapter_features: wgpu::Features,
    adapter_limits: &wgpu::Limits,
    adapter_info: &wgpu::AdapterInfo,
) -> DeviceConfig {
    let mut required_features = wgpu::Features::INDIRECT_FIRST_INSTANCE;

    if adapter_features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT) {
        required_features |= wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
    }

    if adapter_features.contains(wgpu::Features::FLOAT32_FILTERABLE) {
        required_features |= wgpu::Features::FLOAT32_FILTERABLE;
    }

    let timestamps =
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
    if adapter_features.contains(timestamps) {
        required_features |= timestamps;
    }

    const BINDLESS_MAX_TEXTURES: u32 = if cfg!(target_os = "android") {
        512
    } else {
        4096
    };
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
    let bindless_config = crate::renderer::wgpu::caps::BindlessConfig {
        enabled: bindless_enabled,
        max_textures: bindless_capacity,
    };

    let base_limits = if adapter_info.backend == wgpu::Backend::Gl {
        wgpu::Limits::downlevel_webgl2_defaults()
    } else {
        wgpu::Limits::default()
    };
    let mut required_limits = base_limits.using_resolution(adapter_limits.clone());
    required_limits.max_bind_groups = adapter_limits.max_bind_groups.min(8);
    required_limits.max_sampled_textures_per_shader_stage =
        adapter_limits.max_sampled_textures_per_shader_stage.min(64);
    required_limits.max_samplers_per_shader_stage =
        adapter_limits.max_samplers_per_shader_stage.min(64);
    if bindless_enabled {
        required_limits.max_binding_array_elements_per_shader_stage = bindless_capacity;
    }
    required_limits.max_storage_buffer_binding_size =
        adapter_limits.max_storage_buffer_binding_size;
    required_limits.max_texture_array_layers = adapter_limits.max_texture_array_layers;
    required_limits.max_buffer_size = adapter_limits.max_buffer_size;

    DeviceConfig {
        required_features,
        required_limits,
        bindless: bindless_config,
    }
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
            .map_err(|error| format!("no adapter can present to this surface: {error}"))?;

        let adapter_info = adapter.get_info();
        tracing::info!(
            "adapter found: {} ({:?} / {:?})",
            adapter_info.name,
            adapter_info.backend,
            adapter_info.device_type
        );
        let DeviceConfig {
            required_features,
            required_limits,
            bindless,
        } = negotiate_device_config(adapter.features(), &adapter.limits(), &adapter_info);
        tracing::info!(
            "bindless materials: enabled {}, capacity {}",
            bindless.enabled,
            bindless.max_textures
        );

        let missing_features = required_features - adapter.features();
        if !missing_features.is_empty() {
            return Err(format!(
                "the adapter does not have features the renderer must have: {missing_features:?}"
            )
            .into());
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

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let mut renderer = Self::new_with_device(
            &adapter,
            device,
            queue,
            surface,
            surface_format,
            bindless,
            initial_width,
            initial_height,
        )
        .await?;
        renderer.instance = Some(instance);
        Ok(renderer)
    }

    pub async fn new_with_device(
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_format: wgpu::TextureFormat,
        bindless_config: crate::renderer::wgpu::caps::BindlessConfig,
        initial_width: u32,
        initial_height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let adapter_info = adapter.get_info();

        device.set_device_lost_callback(|reason, message| {
            tracing::error!("the graphics device was lost ({reason:?}): {message}");
        });

        tracing::info!("wgpu device features: {:?}", device.features());

        let gpu_profile = build_gpu_profile(&adapter_info);
        tracing::info!(
            "gpu adapter: {} ({:?} / {:?})",
            gpu_profile.name,
            gpu_profile.backend,
            gpu_profile.device_type
        );

        let surface_capabilities = surface.get_capabilities(adapter);
        if surface_capabilities.formats.is_empty() {
            return Err(
                "the selected adapter cannot present to this surface: no supported formats".into(),
            );
        }

        let present_mode = pick_present_mode(&surface_capabilities.present_modes, true);
        tracing::info!(
            "Surface present modes available: {:?}, selected: {:?}",
            surface_capabilities.present_modes,
            present_mode
        );
        tracing::info!(
            "surface formats available: {:?}, selected: {surface_format:?}",
            surface_capabilities.formats
        );
        let alpha_mode = if surface_capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            surface_capabilities.alpha_modes[0]
        };

        let mut surface_usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        if surface_capabilities
            .usages
            .contains(wgpu::TextureUsages::COPY_SRC)
        {
            surface_usage |= wgpu::TextureUsages::COPY_SRC;
        }

        let surface_config = wgpu::SurfaceConfiguration {
            usage: surface_usage,
            format: surface_format,
            width: initial_width,
            height: initial_height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let shader_composer = ShaderComposer::new();
        let point_shadow_format = pick_texture_format(
            adapter,
            &[wgpu::TextureFormat::R32Float, wgpu::TextureFormat::R16Float],
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        tracing::info!("point shadow format: {point_shadow_format:?}");

        let hdr_format = crate::renderer::wgpu::HDR_FORMAT;

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;

        let material_texture_arrays = if bindless_config.enabled {
            crate::renderer::wgpu::material_texture_arrays::MaterialTextureArrays::new_bindless(
                &device,
                &queue,
                bindless_config.max_textures,
            )
        } else {
            crate::renderer::wgpu::material_texture_arrays::MaterialTextureArrays::new(&device)
        };
        let mip_generator =
            crate::renderer::wgpu::mip_generator::MipGenerator::new(&device, &shader_composer);

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;
        let (brdf_lut_texture, brdf_lut_view) =
            brdf_lut::generate_brdf_lut(&device, &shader_composer, &queue);

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;

        let mut graph = rendergraph::render_graph_new();

        let depth_id = render_graph_add_depth_texture(&mut graph, "depth")
            .size(initial_width.max(1), initial_height.max(1))
            .usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            )
            .clear_depth(0.0)
            .force_store()
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

        let shadow_map_size = crate::renderer::wgpu::caps::CASCADE_ATLAS_SIZE;
        let shadow_depth_id = render_graph_add_depth_texture(&mut graph, "shadow_depth")
            .size(shadow_map_size, shadow_map_size)
            .format(wgpu::TextureFormat::Depth32Float)
            .clear_depth(0.0)
            .fixed_size()
            .transient();

        let spotlight_shadow_atlas_size = crate::renderer::wgpu::caps::SPOTLIGHT_ATLAS_SIZE;
        let spotlight_shadow_atlas_id =
            render_graph_add_depth_texture(&mut graph, "spotlight_shadow_atlas")
                .format(wgpu::TextureFormat::Depth32Float)
                .external();
        let spotlight_shadow_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spotlight Shadow Atlas"),
            size: crate::renderer::texture_extent::texture_extent(
                &device,
                spotlight_shadow_atlas_size,
                spotlight_shadow_atlas_size,
                1,
            ),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let spotlight_shadow_atlas_view =
            spotlight_shadow_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        crate::renderer::wgpu::rendergraph::render_graph_set_external_texture(
            &mut graph,
            spotlight_shadow_atlas_id,
            Some(spotlight_shadow_atlas_texture.clone()),
            spotlight_shadow_atlas_view.clone(),
            spotlight_shadow_atlas_size,
            spotlight_shadow_atlas_size,
        );

        let brdf_lut_id = render_graph_add_color_texture(&mut graph, "brdf_lut")
            .format(wgpu::TextureFormat::Rg16Float)
            .external();
        crate::renderer::wgpu::rendergraph::render_graph_set_external_texture(
            &mut graph,
            brdf_lut_id,
            Some(brdf_lut_texture.clone()),
            brdf_lut_view.clone(),
            brdf_lut_texture.width(),
            brdf_lut_texture.height(),
        );
        let ibl_irradiance_id = render_graph_add_color_texture(&mut graph, "ibl_irradiance")
            .format(hdr_format)
            .external();
        let ibl_prefiltered_id = render_graph_add_color_texture(&mut graph, "ibl_prefiltered")
            .format(hdr_format)
            .external();
        let ibl_irradiance_b_id = render_graph_add_color_texture(&mut graph, "ibl_irradiance_b")
            .format(hdr_format)
            .external();
        let ibl_prefiltered_b_id = render_graph_add_color_texture(&mut graph, "ibl_prefiltered_b")
            .format(hdr_format)
            .external();

        let point_shadow_cubemap_id =
            render_graph_add_color_texture(&mut graph, "point_shadow_cubemap")
                .format(point_shadow_format)
                .external();
        let bone_matrices_id = render_graph_add_color_texture(&mut graph, "bone_matrices")
            .format(wgpu::TextureFormat::R32Float)
            .external();
        let transmission_grab_id = render_graph_add_color_texture(&mut graph, "transmission_grab")
            .format(hdr_format)
            .external();

        let glyph_atlas_id = render_graph_add_color_texture(&mut graph, "glyph_atlas")
            .format(wgpu::TextureFormat::Rgba8Unorm)
            .external();

        let outline_mask_id = render_graph_add_color_texture(&mut graph, "outline_mask")
            .format(wgpu::TextureFormat::R8Unorm)
            .size(initial_width.max(1), initial_height.max(1))
            .clear_color(wgpu::Color::TRANSPARENT)
            .transient();

        let entity_id_id = render_graph_add_color_texture(&mut graph, "entity_id")
            .format(wgpu::TextureFormat::R32Float)
            .size(initial_width.max(1), initial_height.max(1))
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .clear_color(wgpu::Color::TRANSPARENT)
            .force_store()
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

        let swapchain_depth_id = render_graph_add_depth_texture(&mut graph, "swapchain_depth")
            .size(initial_width.max(1), initial_height.max(1))
            .format(wgpu::TextureFormat::Depth32Float)
            .clear_depth(0.0)
            .transient();

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

        let aa_output_id = render_graph_add_color_texture(&mut graph, "aa_output")
            .format(surface_format)
            .size(initial_width.max(1), initial_height.max(1))
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let targets = super::RenderTargets {
            depth: depth_id,
            scene_color: scene_color_id,
            compute_output: compute_output_id,
            aa_output: aa_output_id,
            swapchain: swapchain_id,
            viewport_resource: viewport_resource_id,
            swapchain_depth: swapchain_depth_id,
            entity_id: entity_id_id,
            view_normals: view_normals_id,
            spotlight_shadow_atlas: spotlight_shadow_atlas_id,
            glyph_atlas: glyph_atlas_id,
            brdf_lut: brdf_lut_id,
            ibl_irradiance: ibl_irradiance_id,
            ibl_prefiltered: ibl_prefiltered_id,
            ibl_irradiance_b: ibl_irradiance_b_id,
            ibl_prefiltered_b: ibl_prefiltered_b_id,
            point_shadow_cubemap: point_shadow_cubemap_id,
            bone_matrices: bone_matrices_id,
            transmission_grab: transmission_grab_id,
            shadow_depth: shadow_depth_id,
            velocity: velocity_id,
            outline_mask: outline_mask_id,
            ssao_raw: ssao_raw_id,
            ssao: ssao_id,
            ssgi_raw: ssgi_raw_id,
            ssgi: ssgi_id,
            ssr_raw: ssr_raw_id,
            ssr: ssr_id,
        };

        let build = PassBuild {
            device: &device,
            queue: &queue,
            composer: &shader_composer,
            targets: &targets,
            surface_format,
            hdr_format,
            depth_format: crate::renderer::wgpu::DEPTH_FORMAT,
            point_shadow_format,
            size: (initial_width.max(1), initial_height.max(1)),
            bindless: bindless_config,
            low_power: gpu_profile.is_low_power(),
        };

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;
        let bloom_pass = passes::BloomPass::new(build, initial_width.max(1), initial_height.max(1));
        render_graph_add_pass(
            &mut graph,
            Box::new(bloom_pass),
            &[("hdr", scene_color_id), ("bloom", bloom_texture_id)],
        )?;

        let dof_pass = passes::DepthOfFieldPass::new(
            build,
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

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;
        let postprocess_pass = passes::PostProcessPass::new(build, surface_format, 1.0);
        render_graph_add_pass(
            &mut graph,
            Box::new(postprocess_pass),
            &[
                ("hdr", dof_output_id),
                ("scene_color", scene_color_id),
                ("bloom", bloom_texture_id),
                ("ssao", ssao_id),
                ("ssgi", ssgi_id),
                ("ssr", ssr_id),
                ("output", compute_output_id),
            ],
        )?;

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;
        let taa_pass = passes::TaaPass::new(build, surface_format);
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

        let fxaa_pass = passes::FxaaPass::new(build, surface_format);
        render_graph_add_pass(
            &mut graph,
            Box::new(fxaa_pass),
            &[("input", compute_output_id), ("output", aa_output_id)],
        )?;

        let aa_passthrough_pass = passes::BlitPass::new(build, surface_format)
            .with_name("aa_passthrough")
            .when(|configs| {
                configs.settings.anti_aliasing == nightshade_render::data::config::AntiAliasing::Off
            });
        render_graph_add_pass(
            &mut graph,
            Box::new(aa_passthrough_pass),
            &[("input", compute_output_id), ("output", aa_output_id)],
        )?;

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;
        let depth_pick_shader = shader_composer.compile_wgsl(
            &device,
            "Depth Pick Shader",
            include_str!("shaders/depth_pick.wgsl"),
        );

        let depth_pick_bind_group_layout = build::bind_layout(
            &device,
            "Depth Pick Bind Group Layout",
            &[
                build::depth_at(0, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, true, wgpu::ShaderStages::COMPUTE),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
                build::texture_at(
                    3,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
            ],
        );

        let depth_pick_pipeline_layout = build::pipeline_layout(
            &device,
            "Depth Pick Pipeline Layout",
            &[&depth_pick_bind_group_layout],
        );

        let depth_pick_compute_pipeline = build::compute(
            &device,
            "Depth Pick Compute Pipeline",
            &depth_pick_shader,
            "main",
            &depth_pick_pipeline_layout,
        );

        let depth_pick_buffer_size =
            (super::DEPTH_PICK_SAMPLE_SIZE * super::DEPTH_PICK_SAMPLE_SIZE * 8) as u64;
        let depth_pick_storage_buffer = build::buffer(
            &device,
            "Depth Pick Storage Buffer",
            depth_pick_buffer_size,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );

        let depth_pick_uniform_buffer = build::uniform(&device, "Depth Pick Uniform Buffer", 16);

        let depth_pick_staging_buffer = build::buffer(
            &device,
            "Depth Pick Staging Buffer",
            depth_pick_buffer_size,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );

        #[cfg(not(target_arch = "wasm32"))]
        let screenshot_staging_buffer = {
            let bytes_per_pixel = 4u32;
            let unpadded_bytes_per_row = initial_width.max(1) * bytes_per_pixel;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
            let buffer_size = (padded_bytes_per_row * initial_height.max(1)) as u64;
            build::buffer(
                &device,
                "Screenshot Staging Buffer",
                buffer_size,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            )
        };

        crate::renderer::wgpu::stagger::yield_to_event_loop(&queue).await;
        let glyph_atlas = glyph_atlas::GlyphAtlas::new(&device);
        crate::renderer::wgpu::rendergraph::render_graph_set_external_texture(
            &mut graph,
            glyph_atlas_id,
            Some(glyph_atlas.texture.clone()),
            glyph_atlas.texture_view.clone(),
            glyph_atlas.texture.width(),
            glyph_atlas.texture.height(),
        );
        crate::renderer::wgpu::stagger::drain_queued_work(&queue).await;

        let timing = crate::renderer::wgpu::timing::GpuTiming::new(&device, &queue);

        Ok(Self {
            surface: Some(surface),
            instance: None,
            device,
            queue,
            surface_config,
            surface_format,
            point_shadow_format,
            material_texture_arrays,
            supported_present_modes: surface_capabilities.present_modes,
            graph,
            targets,
            glyph_atlas,
            camera_viewports: std::collections::HashMap::new(),
            paint_target: None,
            brdf_lut_texture,
            brdf_lut_view,
            mip_generator,
            shader_composer,
            timing,
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
                source_is_bgra: false,
                writer: None,
            },
            render_buffer_size: (initial_width.max(1), initial_height.max(1)),
            window_render_state: super::WindowRenderState::default(),
            surface_skips: 0,
            frame_state: super::FrameState {
                index: 0,
                last_settings_signature: None,
                captured_ibl_atmosphere: None,
                captured_ibl_hour: 0.0,
                captured_day_night_snapshots: false,
                captured_ibl_bracket: None,
            },
            texture_store: crate::renderer::wgpu::texture_cache::TextureStore::default(),
            gpu_profile,
            bindless: bindless_config,
        })
    }
}

fn build_gpu_profile(info: &wgpu::AdapterInfo) -> crate::renderer::config::GpuProfile {
    use crate::renderer::config::{GpuBackend, GpuDeviceType, GpuProfile};

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
