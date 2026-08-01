use super::MeshBindings;
use super::MeshBuffers;
use super::MeshClusters;
use super::MeshCompute;
use super::MeshFrame;
use super::MeshImageLighting;
use super::MeshInstanced;
use super::MeshMaterials;
use super::MeshMorph;
use super::MeshPipelines;
use super::MeshRegistry;
use super::MeshTransmission;
use nightshade_render::data::geometry::{
    Vertex, create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_quad_mesh, create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::collections::{HashMap, HashSet};

use super::super::types::{
    ClusterBounds, CullingUniforms, INITIAL_INDEX_BUFFER_SIZE, INITIAL_VERTEX_BUFFER_SIZE,
    InstancedTransformComputeUniforms, MeshBoundsData, MorphDisplacement, SceneUniforms,
    TOTAL_CLUSTERS,
};

use super::{MeshOit, MeshPass};
use nightshade_wgpu_shading::backend::material_gpu::material_texture_bind_group_layout_entries;
use nightshade_wgpu_shading::backend::shadow_inputs;

impl MeshPass {
    pub fn new(
        build: PassBuild<'_>,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        window_size: (u32, u32),
        bindless_config: nightshade_wgpu::backend::BindlessConfig,
    ) -> Self {
        let PassBuild {
            device,
            queue,
            composer,
            ..
        } = build;
        let brdf_lut_view = nightshade_wgpu::backend::placeholder::placeholder_texture(
            device,
            queue,
            "BRDF Placeholder",
            wgpu::TextureFormat::Rg16Float,
            1,
            &[0, 0, 0, 0],
        )
        .create_view(&wgpu::TextureViewDescriptor::default());
        let material_bindless_max = bindless_config
            .enabled
            .then_some(bindless_config.max_textures);
        let material_shader_defs: &[(&str, naga_oil::compose::ShaderDefValue)] =
            if bindless_config.enabled {
                &[("BINDLESS", naga_oil::compose::ShaderDefValue::Bool(true))]
            } else {
                &[]
            };
        let shader = composer.compile_wgsl_with_defs(
            device,
            "mesh.wgsl",
            include_str!("../../../../shaders/mesh.wgsl"),
            material_shader_defs,
        );
        let oit_shader = composer.compile_wgsl_with_defs(
            device,
            "mesh_oit.wgsl",
            include_str!("../../../../shaders/mesh_oit.wgsl"),
            material_shader_defs,
        );
        let composite_shader = composer.compile_wgsl(
            device,
            "oit_composite.wgsl",
            include_str!("../../../../shaders/oit_composite.wgsl"),
        );
        let depth_prepass_shader = composer.compile_wgsl(
            device,
            "mesh_depth_prepass.wgsl",
            include_str!("../../../../shaders/mesh_depth_prepass.wgsl"),
        );

        let uniform_buffer = build::uniform(
            device,
            "Mesh Uniform Buffer",
            std::mem::size_of::<SceneUniforms>() as u64,
        );

        let uniform_bind_group_layout = build::bind_layout(
            device,
            "Mesh Uniform Bind Group Layout",
            &[build::uniform_at(
                0,
                wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
            )],
        );

        let uniform_bind_group = build::binding(
            device,
            "Mesh Uniform Bind Group",
            &uniform_bind_group_layout,
            vec![uniform_buffer.as_entire_binding()],
        );

        let overlay_uniform_buffer = build::uniform(
            device,
            "Mesh Overlay Uniform Buffer",
            std::mem::size_of::<SceneUniforms>() as u64,
        );

        let overlay_uniform_bind_group = build::binding(
            device,
            "Mesh Overlay Uniform Bind Group",
            &uniform_bind_group_layout,
            vec![overlay_uniform_buffer.as_entire_binding()],
        );

        let morph_displacement_buffer_size =
            (std::mem::size_of::<MorphDisplacement>() * 1000) as u64;
        let morph_displacement_buffer = build::buffer(
            device,
            "Mesh Morph Displacement Buffer",
            morph_displacement_buffer_size,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let cluster_bounds_buffer = build::storage(
            device,
            "Cluster Bounds Buffer",
            (std::mem::size_of::<ClusterBounds>() * TOTAL_CLUSTERS as usize) as u64,
        );

        let view_matrix_buffer = build::uniform(
            device,
            "View Matrix Buffer",
            std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        );

        let instance_bind_group_layout = build::bind_layout(
            device,
            "Mesh Instance Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(
                    1,
                    false,
                    wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ),
                build::storage_at(
                    2,
                    false,
                    wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ),
                build::storage_at(
                    3,
                    false,
                    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_at(4, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(5, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(6, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(
                    7,
                    false,
                    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_at(
                    8,
                    false,
                    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ),
                build::uniform_at(
                    9,
                    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_at(
                    10,
                    false,
                    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ),
                build::uniform_at(
                    11,
                    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_at(12, false, wgpu::ShaderStages::VERTEX),
            ],
        );

        let oit_accum_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("OIT Accumulation Texture"),
            size: nightshade_wgpu::backend::texture_extent(device, window_size.0, window_size.1, 1),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let oit_reveal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("OIT Reveal Texture"),
            size: nightshade_wgpu::backend::texture_extent(device, window_size.0, window_size.1, 1),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let oit_accum_view = oit_accum_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let oit_reveal_view =
            oit_reveal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let (transmission_color_texture, transmission_color_view, transmission_mip_views) =
            Self::create_transmission_textures(device, window_size.0, window_size.1);
        let transmission_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mesh Transmission Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let transmission_downsample_shader = composer.compile_wgsl(
            device,
            "transmission_downsample.wgsl",
            include_str!("../../../../shaders/transmission_downsample.wgsl"),
        );
        let transmission_downsample_layout = build::bind_layout(
            device,
            "Mesh Transmission Downsample Layout",
            &[build::painted_at(0), build::filtering_at(1)],
        );
        let transmission_downsample_pipeline_layout = build::pipeline_layout(
            device,
            "Mesh Transmission Downsample Pipeline Layout",
            &[&transmission_downsample_layout],
        );
        let transmission_downsample_pipeline = build::Pipe::new(
            device,
            "Mesh Transmission Downsample Pipeline",
            &transmission_downsample_shader,
        )
        .layout(&transmission_downsample_pipeline_layout)
        .target(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba16Float,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })
        .make();
        let transmission_downsample_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mesh Transmission Downsample Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let overlay_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Overlay Depth Texture"),
            size: nightshade_wgpu::backend::texture_extent(device, window_size.0, window_size.1, 1),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let overlay_depth_view =
            overlay_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("OIT Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout = build::bind_layout(
            device,
            "Mesh Texture Bind Group Layout",
            &material_texture_bind_group_layout_entries(material_bindless_max),
        );

        let dummy_shadow_texture = build::surface(
            device,
            "Dummy Shadow Texture",
            wgpu::TextureFormat::Depth32Float,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

        let dummy_shadow_view =
            dummy_shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_sampler = shadow_inputs::shadow_comparison_sampler(device, "Shadow Sampler");

        let spotlight_shadow_buffer =
            shadow_inputs::spotlight_shadow_buffer(device, "Spotlight Shadow Buffer");

        let point_shadow_buffer =
            shadow_inputs::point_shadow_buffer(device, "Point Light Shadow Buffer");

        let point_shadow_sampler =
            shadow_inputs::shadow_comparison_sampler(device, "Point Shadow Sampler");

        let scene_bind_group_layout =
            shadow_inputs::scene_bind_group_layout(device, "Scene Bind Group Layout");

        let shadow_pcf_sampler = shadow_inputs::shadow_pcf_sampler(device, "Shadow PCF Sampler");
        let ltc = nightshade_wgpu_shading::backend::ltc::create(device, queue);

        let ibl_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("IBL Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let dummy_cubemap = nightshade_wgpu::backend::placeholder::placeholder_texture(
            device,
            queue,
            "Dummy IBL Cubemap",
            wgpu::TextureFormat::Rgba16Float,
            6,
            &nightshade_wgpu::backend::placeholder::TRANSPARENT_RGBA16,
        );
        let dummy_cubemap_view = dummy_cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let dummy_point_shadow_view = shadow_inputs::dummy_point_shadow_cubemap_view(
            device,
            queue,
            "Dummy Point Shadow Cubemap Array",
        );

        let air = nightshade_wgpu_shading::backend::fog::Air::new(
            build,
            &uniform_bind_group_layout,
            &instance_bind_group_layout,
            &texture_bind_group_layout,
            color_format,
            depth_format,
            material_shader_defs,
        );
        let air_reads = air.reads();

        let scene_bind_group = build::binding(
            device,
            "Scene Bind Group",
            &scene_bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(&dummy_shadow_view),
                wgpu::BindingResource::Sampler(&shadow_sampler),
                wgpu::BindingResource::TextureView(&dummy_shadow_view),
                wgpu::BindingResource::Sampler(&shadow_sampler),
                spotlight_shadow_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&brdf_lut_view),
                wgpu::BindingResource::Sampler(&ibl_sampler),
                wgpu::BindingResource::TextureView(&dummy_cubemap_view),
                wgpu::BindingResource::Sampler(&ibl_sampler),
                wgpu::BindingResource::TextureView(&dummy_cubemap_view),
                wgpu::BindingResource::Sampler(&ibl_sampler),
                wgpu::BindingResource::TextureView(&dummy_point_shadow_view),
                wgpu::BindingResource::Sampler(&point_shadow_sampler),
                point_shadow_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&dummy_cubemap_view),
                wgpu::BindingResource::TextureView(&dummy_cubemap_view),
                wgpu::BindingResource::TextureView(&transmission_color_view),
                wgpu::BindingResource::Sampler(&transmission_sampler),
                wgpu::BindingResource::Sampler(&shadow_pcf_sampler),
                wgpu::BindingResource::TextureView(&ltc.view_matrix),
                wgpu::BindingResource::TextureView(&ltc.view_terms),
                wgpu::BindingResource::Sampler(&ltc.sampler),
                wgpu::BindingResource::TextureView(&air_reads.volume),
                wgpu::BindingResource::Sampler(&air_reads.sampler),
                air_reads.uniforms.as_entire_binding(),
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Mesh Pipeline Layout",
            &[
                &uniform_bind_group_layout,
                &instance_bind_group_layout,
                &texture_bind_group_layout,
                &scene_bind_group_layout,
            ],
        );

        let depth_prepass_pipeline_layout = build::pipeline_layout(
            device,
            "Depth Prepass Pipeline Layout",
            &[&uniform_bind_group_layout, &instance_bind_group_layout],
        );

        let vertex_attributes = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 24,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 40,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 56,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes,
        };

        let opaque_targets: Vec<Option<wgpu::ColorTargetState>> = vec![
            Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }),
            Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }),
            Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }),
            Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rg16Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }),
        ];

        let opaque_pipeline = build::Pipe::new(device, "Opaque Mesh Pipeline", &shader)
            .layout(&pipeline_layout)
            .buffers(std::slice::from_ref(&vertex_buffer_layout))
            .targets(&opaque_targets)
            .depth(build::depth_state(depth_format, true))
            .cull()
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let opaque_double_sided_pipeline =
            build::Pipe::new(device, "Opaque Double-Sided Mesh Pipeline", &shader)
                .layout(&pipeline_layout)
                .buffers(std::slice::from_ref(&vertex_buffer_layout))
                .targets(&opaque_targets)
                .depth(build::depth_state(depth_format, true))
                .vertex("vs_main")
                .fragment("fs_main")
                .make();

        let overlay_opaque_pipeline =
            build::Pipe::new(device, "Overlay Opaque Mesh Pipeline", &shader)
                .layout(&pipeline_layout)
                .buffers(&[vertex_buffer_layout])
                .targets(&opaque_targets)
                .depth(build::depth_nearer(depth_format))
                .cull()
                .vertex("vs_main")
                .fragment("fs_main")
                .make();

        let vertex_buffer_layout2 = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes,
        };

        let overlay_opaque_double_sided_pipeline =
            build::Pipe::new(device, "Overlay Opaque Double-Sided Mesh Pipeline", &shader)
                .layout(&pipeline_layout)
                .buffers(std::slice::from_ref(&vertex_buffer_layout2))
                .targets(&opaque_targets)
                .depth(build::depth_nearer(depth_format))
                .vertex("vs_main")
                .fragment("fs_main")
                .make();

        let depth_prepass_vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes,
        };

        let depth_prepass_pipeline =
            build::Pipe::new(device, "Depth Prepass Pipeline", &depth_prepass_shader)
                .layout(&depth_prepass_pipeline_layout)
                .buffers(std::slice::from_ref(&depth_prepass_vertex_buffer_layout))
                .depth(build::depth_nearer(depth_format))
                .cull()
                .vertex("vs_main")
                .fragment("fs_main")
                .make();

        let oit_pipeline = build::Pipe::new(device, "OIT Mesh Pipeline", &oit_shader)
            .layout(&pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &vertex_attributes,
            }])
            .targets(&[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::Src,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::Src,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::RED,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rg16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ])
            .depth(build::depth_state(depth_format, false))
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let blend_opaque_depth_prepass_pipeline =
            build::Pipe::new(device, "Blend Opaque Depth Prepass Pipeline", &oit_shader)
                .layout(&pipeline_layout)
                .buffers(&[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attributes,
                }])
                .depth(build::depth_nearer(depth_format))
                .vertex("vs_main")
                .fragment("fs_blend_opaque_prepass")
                .make();

        let composite_bind_group_layout = build::bind_layout(
            device,
            "OIT Composite Bind Group Layout",
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::painted_at(2),
                build::filtering_at(3),
            ],
        );

        let oit_composite_bind_group = build::binding(
            device,
            "OIT Composite Bind Group",
            &composite_bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(&oit_accum_view),
                wgpu::BindingResource::Sampler(&sampler),
                wgpu::BindingResource::TextureView(&oit_reveal_view),
                wgpu::BindingResource::Sampler(&sampler),
            ],
        );

        let composite_pipeline_layout = build::pipeline_layout(
            device,
            "OIT Composite Pipeline Layout",
            &[&composite_bind_group_layout],
        );

        let oit_composite_pipeline =
            build::Pipe::new(device, "OIT Composite Pipeline", &composite_shader)
                .layout(&composite_pipeline_layout)
                .target(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })
                .vertex("vs_main")
                .fragment("fs_main")
                .make();

        let vertex_buffer = build::buffer(
            device,
            "Mesh Vertex Buffer",
            (std::mem::size_of::<Vertex>() * INITIAL_VERTEX_BUFFER_SIZE) as u64,
            wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let index_buffer = build::buffer(
            device,
            "Mesh Index Buffer",
            (std::mem::size_of::<u32>() * INITIAL_INDEX_BUFFER_SIZE) as u64,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        );

        let vertex_buffer_size =
            (std::mem::size_of::<Vertex>() * INITIAL_VERTEX_BUFFER_SIZE) as u64;
        let index_buffer_size = (std::mem::size_of::<u32>() * INITIAL_INDEX_BUFFER_SIZE) as u64;

        let culling_shader = composer.compile_wgsl(
            device,
            "Mesh Culling Shader",
            include_str!("../../../../shaders/mesh_culling.wgsl"),
        );

        let culling_bind_group_layout = build::bind_layout(
            device,
            "Mesh Culling Bind Group Layout",
            &[
                build::storage_at(0, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, false, wgpu::ShaderStages::COMPUTE),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
                build::storage_at(3, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(4, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(5, true, wgpu::ShaderStages::COMPUTE),
                build::texture_at(
                    6,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
            ],
        );

        let culling_pipeline_layout = build::pipeline_layout(
            device,
            "Mesh Culling Pipeline Layout",
            &[&culling_bind_group_layout],
        );

        let culling_pipeline = build::compute(
            device,
            "Mesh Culling Pipeline",
            &culling_shader,
            "main",
            &culling_pipeline_layout,
        );

        let culling_uniform_buffer = build::uniform(
            device,
            "Mesh Culling Uniform Buffer",
            std::mem::size_of::<CullingUniforms>() as u64,
        );

        let culling_uniform_buffer_frustum = build::uniform(
            device,
            "Mesh Culling Uniform Buffer (Frustum Prepass)",
            std::mem::size_of::<CullingUniforms>() as u64,
        );

        let compaction_shader = composer.compile_wgsl(
            device,
            "Mesh Indirect Compaction Shader",
            include_str!("../../../../shaders/mesh_indirect_compact.wgsl"),
        );

        let compaction_storage_entry = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let compaction_bind_group_layout = build::bind_layout(
            device,
            "Mesh Indirect Compaction Bind Group Layout",
            &[
                compaction_storage_entry(0, false),
                compaction_storage_entry(1, false),
                compaction_storage_entry(2, true),
                build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let compaction_pipeline_layout = build::pipeline_layout(
            device,
            "Mesh Indirect Compaction Pipeline Layout",
            &[&compaction_bind_group_layout],
        );

        let compaction_pipeline = build::compute(
            device,
            "Mesh Indirect Compaction Pipeline",
            &compaction_shader,
            "main",
            &compaction_pipeline_layout,
        );

        let compaction_params: [u32; 4] = [super::super::types::NUM_DRAW_CLASSES as u32, 0, 0, 0];
        let compaction_params_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Indirect Compaction Params Buffer"),
                contents: bytemuck::cast_slice(&compaction_params),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );

        let supports_multi_draw_count = device
            .features()
            .contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT);

        let indirect_build_shader = composer.compile_wgsl(
            device,
            "Mesh Indirect Build Shader",
            include_str!("../../../../shaders/mesh_indirect_build.wgsl"),
        );

        let indirect_build_bind_group_layout = build::bind_layout(
            device,
            "Mesh Indirect Build Bind Group Layout",
            &[
                compaction_storage_entry(0, false),
                compaction_storage_entry(1, true),
                compaction_storage_entry(2, true),
                build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let indirect_build_pipeline_layout = build::pipeline_layout(
            device,
            "Mesh Indirect Build Pipeline Layout",
            &[&indirect_build_bind_group_layout],
        );

        let indirect_build_pipeline = build::compute(
            device,
            "Mesh Indirect Build Pipeline",
            &indirect_build_shader,
            "main",
            &indirect_build_pipeline_layout,
        );

        let indirect_build_params_buffer = build::uniform(
            device,
            "Mesh Indirect Build Params Buffer",
            (std::mem::size_of::<u32>() * 4) as u64,
        );

        let mesh_geo_buffer_size = 64;
        let mesh_geo_buffer = build::storage(
            device,
            "Mesh Geometry Table Buffer",
            (std::mem::size_of::<super::super::types::MeshGeo>() * mesh_geo_buffer_size) as u64,
        );

        let batch_assign_shader = composer.compile_wgsl(
            device,
            "Mesh Batch Assign Shader",
            include_str!("../../../../shaders/mesh_batch_assign.wgsl"),
        );

        let batch_assign_bind_group_layout = build::bind_layout(
            device,
            "Mesh Batch Assign Bind Group Layout",
            &[
                compaction_storage_entry(0, false),
                compaction_storage_entry(1, true),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
                compaction_storage_entry(3, true),
            ],
        );

        let batch_assign_pipeline_layout = build::pipeline_layout(
            device,
            "Mesh Batch Assign Pipeline Layout",
            &[&batch_assign_bind_group_layout],
        );

        let batch_assign_pipeline = build::compute(
            device,
            "Mesh Batch Assign Pipeline",
            &batch_assign_shader,
            "main",
            &batch_assign_pipeline_layout,
        );

        let batch_assign_params_buffer = build::uniform(
            device,
            "Mesh Batch Assign Params Buffer",
            (std::mem::size_of::<u32>() * 8) as u64,
        );

        let gpu_batch_shader = composer.compile_wgsl(
            device,
            "Mesh GPU Batch Shader",
            include_str!("../../../../shaders/mesh_gpu_batch.wgsl"),
        );
        let gpu_batch_bind_group_layout = build::bind_layout(
            device,
            "Mesh GPU Batch Bind Group Layout",
            &[
                compaction_storage_entry(0, false),
                compaction_storage_entry(1, true),
                compaction_storage_entry(2, false),
                build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
                compaction_storage_entry(4, false),
                compaction_storage_entry(5, false),
                compaction_storage_entry(6, false),
            ],
        );
        let gpu_batch_pipeline_layout = build::pipeline_layout(
            device,
            "Mesh GPU Batch Pipeline Layout",
            &[&gpu_batch_bind_group_layout],
        );
        let make_gpu_batch_pipeline = |label: &str, entry: &str| {
            build::compute(
                device,
                label,
                &gpu_batch_shader,
                entry,
                &gpu_batch_pipeline_layout,
            )
        };
        let gpu_batch_classify_pipeline =
            make_gpu_batch_pipeline("Mesh GPU Batch Classify Pipeline", "classify");
        let gpu_batch_clear_pipeline =
            make_gpu_batch_pipeline("Mesh GPU Batch Clear Pipeline", "clear_dense");
        let gpu_batch_count_pipeline =
            make_gpu_batch_pipeline("Mesh GPU Batch Count Pipeline", "count");
        let gpu_batch_build_pipeline =
            make_gpu_batch_pipeline("Mesh GPU Batch Build Pipeline", "build");
        let gpu_batch_params_buffer = build::uniform(
            device,
            "Mesh GPU Batch Params Buffer",
            (std::mem::size_of::<u32>() * 8) as u64,
        );
        let mesh_bounds_buffer_size = 64;
        let mesh_bounds_buffer = build::buffer(
            device,
            "Mesh Bounds Buffer",
            (std::mem::size_of::<MeshBoundsData>() * mesh_bounds_buffer_size) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let cluster_compute = nightshade_wgpu_shading::backend::cluster::ClusterCompute::new(build);
        let cluster_bounds_pipeline = cluster_compute.bounds_pipeline;
        let cluster_assign_pipeline = cluster_compute.assign_pipeline;
        let cluster_bounds_bind_group_layout = cluster_compute.bounds_bind_group_layout;
        let cluster_assign_bind_group_layout = cluster_compute.assign_bind_group_layout;

        let instanced_compute_shader = composer.compile_wgsl(
            device,
            "Instanced Transform Compute Shader",
            include_str!("../../../../shaders/instanced_transform_compute.wgsl"),
        );

        let instanced_compute_bind_group_layout = build::bind_layout(
            device,
            "Instanced Transform Compute Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, true, wgpu::ShaderStages::COMPUTE),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );

        let instanced_compute_pipeline_layout = build::pipeline_layout(
            device,
            "Instanced Transform Compute Pipeline Layout",
            &[&instanced_compute_bind_group_layout],
        );

        let instanced_compute_pipeline = build::compute(
            device,
            "Instanced Transform Compute Pipeline",
            &instanced_compute_shader,
            "main",
            &instanced_compute_pipeline_layout,
        );

        let instanced_compute_uniforms_buffer = build::uniform(
            device,
            "Instanced Transform Compute Uniforms Buffer",
            std::mem::size_of::<InstancedTransformComputeUniforms>() as u64,
        );

        let mut pass = Self {
            frame: MeshFrame {
                counter: 0,
                last_compaction: 0,
                resize_rebuild_pending: false,
                scratch_updates: Vec::new(),
                dirty: None,
                dirty_consumed: true,
                dirty_mesh_ids: HashSet::new(),
                last_cache_clear: 0,
            },
            morph: MeshMorph {
                buffer: morph_displacement_buffer,
                buffer_size: morph_displacement_buffer_size,
                offset: 0,
            },
            instanced: MeshInstanced {
                pipeline: instanced_compute_pipeline,
                layout: instanced_compute_bind_group_layout,
                uniforms: instanced_compute_uniforms_buffer,
                uniform_slots: 0,
            },
            bindings: MeshBindings {
                uniforms: uniform_buffer,
                uniform_group: uniform_bind_group,
                overlay_uniforms: overlay_uniform_buffer,
                overlay_uniform_group: overlay_uniform_bind_group,
                instance_layout: instance_bind_group_layout,
                texture_layout: texture_bind_group_layout,
                scene_group: scene_bind_group,
                scene_layout: scene_bind_group_layout.clone(),
                scene_group_dirty: true,
                shadow_sampler,
                shadow_pcf_sampler,
                spotlight_shadows: spotlight_shadow_buffer,
                point_shadows: point_shadow_buffer,
                point_shadow_sampler,
                ibl_sampler,
            },
            pipelines: MeshPipelines {
                opaque: opaque_pipeline,
                opaque_double_sided: opaque_double_sided_pipeline,
                overlay: overlay_opaque_pipeline,
                overlay_double_sided: overlay_opaque_double_sided_pipeline,
                depth_prepass: depth_prepass_pipeline,
                blend_depth_prepass: blend_opaque_depth_prepass_pipeline,
                overlay_depth_texture,
                overlay_depth_view,
            },
            image_lighting: MeshImageLighting {
                brdf_lut: brdf_lut_view,
                irradiance: dummy_cubemap_view.clone(),
                prefiltered: dummy_cubemap_view.clone(),
                irradiance_b: dummy_cubemap_view.clone(),
                prefiltered_b: dummy_cubemap_view,
                point_shadow_cubemap: dummy_point_shadow_view,
            },
            clusters: MeshClusters {
                bounds_buffer: cluster_bounds_buffer,
                bounds_pipeline: cluster_bounds_pipeline,
                assign_pipeline: cluster_assign_pipeline,
                bounds_layout: cluster_bounds_bind_group_layout,
                assign_layout: cluster_assign_bind_group_layout,
                view_matrix_buffer,
            },
            compute: MeshCompute {
                cull_pipeline: culling_pipeline,
                cull_layout: culling_bind_group_layout,
                cull_uniforms: culling_uniform_buffer,
                cull_uniforms_frustum: culling_uniform_buffer_frustum,
                compaction_pipeline,
                compaction_layout: compaction_bind_group_layout,
                compaction_params: compaction_params_buffer,
                supports_multi_draw_count,
                indirect_pipeline: indirect_build_pipeline,
                indirect_layout: indirect_build_bind_group_layout,
                indirect_params: indirect_build_params_buffer,
                assign_pipeline: batch_assign_pipeline,
                assign_layout: batch_assign_bind_group_layout,
                assign_params: batch_assign_params_buffer,
                batch_classify: gpu_batch_classify_pipeline,
                batch_clear: gpu_batch_clear_pipeline,
                batch_count: gpu_batch_count_pipeline,
                batch_build: gpu_batch_build_pipeline,
                batch_layout: gpu_batch_bind_group_layout,
                batch_params: gpu_batch_params_buffer,
                batching_enabled: false,
                batch_dirty: false,
                batch_cap: super::super::types::MAX_BATCHES_PER_CLASS,
                batch_keys_per_lane: 1,
            },
            materials: MeshMaterials {
                registered_textures: HashMap::new(),
                array_srgb_view: None,
                array_linear_view: None,
                array_samplers: None,
                bind_group: None,
                bindless_max: material_bindless_max,
                layer_map: HashMap::new(),
            },
            registry: MeshRegistry {
                ids: HashMap::new(),
                data: Vec::new(),
                names: Vec::new(),
                free_ids: Vec::new(),
                rescan: true,
                new_names: Vec::new(),
                geo_buffer: mesh_geo_buffer,
                geo_buffer_size: mesh_geo_buffer_size,
                geo_dirty: true,
                bounds_buffer: mesh_bounds_buffer,
                bounds_buffer_size: mesh_bounds_buffer_size,
                bounds_data: Vec::new(),
            },
            buffers: MeshBuffers {
                vertex: vertex_buffer,
                index: index_buffer,
                vertex_size: vertex_buffer_size,
                index_size: index_buffer_size,
                vertex_offset: 0,
                index_offset: 0,
                vertex_utilization: 1.0,
                index_utilization: 1.0,
                free_vertex: Vec::new(),
                free_index: Vec::new(),
            },
            transmission: MeshTransmission {
                color_texture: transmission_color_texture,
                publish_pending: true,
                color_view: transmission_color_view,
                sampler: transmission_sampler,
                size: window_size,
                mip_views: transmission_mip_views,
                downsample_pipeline: transmission_downsample_pipeline,
                downsample_layout: transmission_downsample_layout,
                downsample_sampler: transmission_downsample_sampler,
            },
            oit: MeshOit {
                pipeline: oit_pipeline,
                composite_pipeline: oit_composite_pipeline,
                accum_texture: oit_accum_texture,
                accum_view: oit_accum_view,
                reveal_texture: oit_reveal_texture,
                reveal_view: oit_reveal_view,
                composite_bind_group: oit_composite_bind_group,
                composite_bind_group_layout,
                sampler,
                texture_size: window_size,
                resource_cache: Vec::new(),
            },
            ltc,
            hiz: nightshade_wgpu_shading::backend::hiz::HizPass::new(build),
            air,
            state: None,
        };

        pass.add_mesh(device, queue, "Cube", &create_cube_mesh(), None);
        pass.add_mesh(device, queue, "Sphere", &create_sphere_mesh(1.0, 16), None);
        pass.add_mesh(device, queue, "Plane", &create_plane_mesh(2.0), None);
        pass.add_mesh(
            device,
            queue,
            "SubdividedPlane",
            &create_subdivided_plane_mesh(2.0, 20),
            None,
        );
        pass.add_mesh(
            device,
            queue,
            "Cylinder",
            &create_cylinder_mesh(0.5, 1.0, 16),
            None,
        );
        pass.add_mesh(device, queue, "Cone", &create_cone_mesh(0.5, 1.0, 16), None);
        pass.add_mesh(
            device,
            queue,
            "Torus",
            &create_torus_mesh(1.0, 0.3, 16, 16),
            None,
        );
        pass.add_mesh(device, queue, "Quad", &create_quad_mesh(1.0), None);

        pass
    }

    pub(in super::super) fn transmission_mip_count(width: u32, height: u32) -> u32 {
        let max_dimension = width.max(height).max(1);
        32 - max_dimension.leading_zeros()
    }

    pub(in super::super) fn create_transmission_textures(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView, Vec<wgpu::TextureView>) {
        let width = width.max(1);
        let height = height.max(1);
        let mip_count = Self::transmission_mip_count(width, height);
        let texture = build::surface(
            device,
            "Mesh Transmission Color Texture",
            wgpu::TextureFormat::Rgba16Float,
            [width, height, 1],
            mip_count,
            wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let full_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut mip_views = Vec::with_capacity(mip_count as usize);
        for level in 0..mip_count {
            mip_views.push(texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Mesh Transmission Mip View"),
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            }));
        }
        (texture, full_view, mip_views)
    }
}
