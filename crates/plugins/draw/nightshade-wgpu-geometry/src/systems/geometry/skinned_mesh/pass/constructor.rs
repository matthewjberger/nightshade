use super::SkinnedBindings;
use super::SkinnedBuffers;
use super::SkinnedClusters;
use super::SkinnedCompute;
use super::SkinnedImages;
use super::SkinnedMaterials;
use super::SkinnedMorph;
use super::SkinnedOit;
use super::SkinnedSkinning;
use nightshade_render::data::geometry::SkinnedVertex;
use nightshade_render::data::skinning::SkinningCache;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::collections::{HashMap, HashSet};

use super::super::types::*;
use super::{SkinnedMeshPass, skinned_texture_bind_group_layout_entries};
use nightshade_wgpu_shading::backend::shadow_inputs;

impl SkinnedMeshPass {
    pub fn new(
        build: PassBuild<'_>,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        bindless_config: nightshade_wgpu::backend::BindlessConfig,
        air: nightshade_wgpu_shading::backend::fog::AirReads,
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
        let skinning_compute_shader = composer.compile_wgsl(
            device,
            "skinning_compute.wgsl",
            nightshade_wgpu_shading::backend::SKINNING_COMPUTE,
        );
        let skinned_mesh_shader = composer.compile_wgsl_with_defs(
            device,
            "skinned_mesh.wgsl",
            include_str!("../../../../shaders/skinned_mesh.wgsl"),
            material_shader_defs,
        );

        let bone_transforms_buffer = build::storage(
            device,
            "Bone Transforms Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * INITIAL_MAX_TOTAL_BONES) as u64,
        );

        let inverse_bind_matrices_buffer = build::storage(
            device,
            "Inverse Bind Matrices Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS) as u64,
        );

        let joint_matrices_buffer = build::buffer(
            device,
            "Joint Matrices Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let prev_joint_matrices_buffer = build::storage(
            device,
            "Previous Joint Matrices Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS) as u64,
        );

        let skin_data_buffer = build::storage(
            device,
            "Skin Data Buffer",
            (std::mem::size_of::<nightshade_render::data::skinning::GpuSkinData>()
                * INITIAL_MAX_SKINS) as u64,
        );

        let skinning_compute_bind_group_layout = build::bind_layout(
            device,
            "Skinning Compute Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(2, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(3, true, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let skinning_compute_bind_group = build::binding(
            device,
            "Skinning Compute Bind Group",
            &skinning_compute_bind_group_layout,
            vec![
                bone_transforms_buffer.as_entire_binding(),
                inverse_bind_matrices_buffer.as_entire_binding(),
                skin_data_buffer.as_entire_binding(),
                joint_matrices_buffer.as_entire_binding(),
            ],
        );

        let skinning_compute_pipeline_layout = build::pipeline_layout(
            device,
            "Skinning Compute Pipeline Layout",
            &[&skinning_compute_bind_group_layout],
        );

        let skinning_compute_pipeline = build::compute(
            device,
            "Skinning Compute Pipeline",
            &skinning_compute_shader,
            "main",
            &skinning_compute_pipeline_layout,
        );

        let cull_shader = composer.compile_wgsl(
            device,
            "skinned_cull.wgsl",
            include_str!("../../../../shaders/skinned_cull.wgsl"),
        );

        let cull_storage_entry = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let cull_bind_group_layout = build::bind_layout(
            device,
            "Skinned Mesh Cull Bind Group Layout",
            &[
                cull_storage_entry(0, true),
                cull_storage_entry(1, false),
                cull_storage_entry(2, false),
                build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
            ],
        );
        let cull_pipeline_layout = build::pipeline_layout(
            device,
            "Skinned Mesh Cull Pipeline Layout",
            &[&cull_bind_group_layout],
        );
        let cull_pipeline = build::compute(
            device,
            "Skinned Mesh Cull Pipeline",
            &cull_shader,
            "main",
            &cull_pipeline_layout,
        );

        let bounds_compute_shader = composer.compile_wgsl(
            device,
            "skinned_bounds.wgsl",
            include_str!("../../../../shaders/skinned_bounds.wgsl"),
        );
        let bounds_compute_bind_group_layout = build::bind_layout(
            device,
            "Skinned Bounds Compute Bind Group Layout",
            &[
                cull_storage_entry(0, true),
                cull_storage_entry(1, true),
                cull_storage_entry(2, false),
            ],
        );
        let bounds_compute_pipeline_layout = build::pipeline_layout(
            device,
            "Skinned Bounds Compute Pipeline Layout",
            &[&bounds_compute_bind_group_layout],
        );
        let bounds_compute_pipeline = build::compute(
            device,
            "Skinned Bounds Compute Pipeline",
            &bounds_compute_shader,
            "main",
            &bounds_compute_pipeline_layout,
        );

        let batch_shader = composer.compile_wgsl(
            device,
            "skinned_gpu_batch.wgsl",
            include_str!("../../../../shaders/skinned_gpu_batch.wgsl"),
        );
        let batch_bind_group_layout = build::bind_layout(
            device,
            "Skinned Mesh Batch Bind Group Layout",
            &[
                cull_storage_entry(0, true),
                cull_storage_entry(1, false),
                cull_storage_entry(2, false),
                cull_storage_entry(3, false),
                cull_storage_entry(4, false),
                cull_storage_entry(5, false),
                cull_storage_entry(6, true),
                build::uniform_at(7, wgpu::ShaderStages::COMPUTE),
            ],
        );
        let batch_pipeline_layout = build::pipeline_layout(
            device,
            "Skinned Mesh Batch Pipeline Layout",
            &[&batch_bind_group_layout],
        );
        let make_batch_pipeline = |label: &str, entry: &str| {
            build::compute(device, label, &batch_shader, entry, &batch_pipeline_layout)
        };
        let batch_clear_pipeline = make_batch_pipeline("Skinned Mesh Batch Clear", "clear");
        let batch_count_pipeline = make_batch_pipeline("Skinned Mesh Batch Count", "count");
        let batch_build_pipeline = make_batch_pipeline("Skinned Mesh Batch Build", "build");
        let batch_assign_pipeline = make_batch_pipeline("Skinned Mesh Batch Assign", "assign");
        let batch_params_buffer = build::uniform(
            device,
            "Skinned Mesh Batch Params Buffer",
            (std::mem::size_of::<u32>() * 4) as u64,
        );
        let batch_mesh_geo_buffer_size = 64;
        let batch_mesh_geo_buffer = build::storage(
            device,
            "Skinned Mesh Batch Mesh Geo Buffer",
            (std::mem::size_of::<[u32; 4]>() * batch_mesh_geo_buffer_size) as u64,
        );

        let uniform_buffer = build::uniform(
            device,
            "Skinned Mesh Uniform Buffer",
            std::mem::size_of::<SceneUniforms>() as u64,
        );

        let uniform_bind_group_layout = build::bind_layout(
            device,
            "Skinned Mesh Uniform Bind Group Layout",
            &[build::uniform_at(
                0,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            )],
        );

        let uniform_bind_group = build::binding(
            device,
            "Skinned Mesh Uniform Bind Group",
            &uniform_bind_group_layout,
            vec![uniform_buffer.as_entire_binding()],
        );

        let morph_displacement_buffer_size = INITIAL_MORPH_DISPLACEMENT_COUNT;
        let morph_displacement_buffer = build::buffer(
            device,
            "Skinned Mesh Morph Displacement Buffer",
            (std::mem::size_of::<MorphDisplacement>() * morph_displacement_buffer_size) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let cluster_bounds_buffer = build::storage(
            device,
            "Skinned Mesh Cluster Bounds Buffer",
            (std::mem::size_of::<ClusterBounds>() * TOTAL_CLUSTERS as usize) as u64,
        );

        let view_matrix_buffer = build::uniform(
            device,
            "Skinned Mesh View Matrix Buffer",
            std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        );

        let instance_bind_group_layout = build::bind_layout(
            device,
            "Skinned Mesh Instance Bind Group Layout",
            &[
                build::storage_at(
                    0,
                    false,
                    wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ),
                build::storage_at(1, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(2, false, wgpu::ShaderStages::FRAGMENT),
                build::storage_at(3, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(4, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(5, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(6, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(7, false, wgpu::ShaderStages::FRAGMENT),
                build::storage_at(8, false, wgpu::ShaderStages::FRAGMENT),
                build::uniform_at(9, wgpu::ShaderStages::FRAGMENT),
                build::storage_at(10, false, wgpu::ShaderStages::FRAGMENT),
                build::uniform_at(11, wgpu::ShaderStages::FRAGMENT),
                build::storage_at(12, false, wgpu::ShaderStages::VERTEX),
            ],
        );

        let texture_bind_group_layout = build::bind_layout(
            device,
            "Skinned Mesh Texture Bind Group Layout",
            &skinned_texture_bind_group_layout_entries(material_bindless_max),
        );

        let shadow_sampler =
            shadow_inputs::shadow_comparison_sampler(device, "Skinned Mesh Shadow Sampler");
        let shadow_pcf_sampler =
            shadow_inputs::shadow_pcf_sampler(device, "Skinned Mesh Shadow PCF Sampler");
        let ltc = nightshade_wgpu_shading::backend::ltc::create(device, queue);

        let spotlight_shadow_buffer =
            shadow_inputs::spotlight_shadow_buffer(device, "Skinned Mesh Spotlight Shadow Buffer");

        let shadow_bind_group_layout =
            shadow_inputs::scene_bind_group_layout(device, "Skinned Mesh Shadow Bind Group Layout");

        let dummy_depth_texture = nightshade_wgpu::backend::placeholder::placeholder_depth_texture(
            device,
            queue,
            "Skinned Mesh Dummy Depth Texture",
        );
        let dummy_depth_view = dummy_depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Skinned Mesh Dummy Depth View"),
            ..Default::default()
        });

        let dummy_brdf_texture = nightshade_wgpu::backend::placeholder::placeholder_texture(
            device,
            queue,
            "Skinned Mesh Dummy BRDF LUT",
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            &nightshade_wgpu::backend::placeholder::TRANSPARENT_RGBA8,
        );
        let dummy_brdf_view =
            dummy_brdf_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let dummy_ibl_cube = nightshade_wgpu::backend::placeholder::placeholder_texture(
            device,
            queue,
            "Skinned Mesh Dummy IBL Cube",
            wgpu::TextureFormat::Rgba8Unorm,
            6,
            &nightshade_wgpu::backend::placeholder::TRANSPARENT_RGBA8,
        );
        let dummy_ibl_cube_view = dummy_ibl_cube.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let ibl_sampler_init = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Skinned Mesh IBL Init Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let point_shadow_buffer =
            shadow_inputs::point_shadow_buffer(device, "Skinned Mesh Point Light Shadow Buffer");

        let point_shadow_sampler =
            shadow_inputs::shadow_comparison_sampler(device, "Skinned Mesh Point Shadow Sampler");

        let dummy_point_shadow_view = shadow_inputs::dummy_point_shadow_cubemap_view(
            device,
            queue,
            "Skinned Mesh Dummy Point Shadow Cubemap Array",
        );

        let shadow_bind_group = build::binding(
            device,
            "Skinned Mesh Shadow Bind Group",
            &shadow_bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(&dummy_depth_view),
                wgpu::BindingResource::Sampler(&shadow_sampler),
                wgpu::BindingResource::TextureView(&dummy_depth_view),
                wgpu::BindingResource::Sampler(&shadow_sampler),
                spotlight_shadow_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&dummy_brdf_view),
                wgpu::BindingResource::Sampler(&ibl_sampler_init),
                wgpu::BindingResource::TextureView(&dummy_ibl_cube_view),
                wgpu::BindingResource::Sampler(&ibl_sampler_init),
                wgpu::BindingResource::TextureView(&dummy_ibl_cube_view),
                wgpu::BindingResource::Sampler(&ibl_sampler_init),
                wgpu::BindingResource::TextureView(&dummy_point_shadow_view),
                wgpu::BindingResource::Sampler(&point_shadow_sampler),
                point_shadow_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&dummy_ibl_cube_view),
                wgpu::BindingResource::TextureView(&dummy_ibl_cube_view),
                wgpu::BindingResource::TextureView(&dummy_brdf_view),
                wgpu::BindingResource::Sampler(&ibl_sampler_init),
                wgpu::BindingResource::Sampler(&shadow_pcf_sampler),
                wgpu::BindingResource::TextureView(&ltc.view_matrix),
                wgpu::BindingResource::TextureView(&ltc.view_terms),
                wgpu::BindingResource::Sampler(&ltc.sampler),
                wgpu::BindingResource::TextureView(&air.volume),
                wgpu::BindingResource::Sampler(&air.sampler),
                air.uniforms.as_entire_binding(),
            ],
        );

        let dummy_white_texture = build::surface(
            device,
            "Skinned Mesh Dummy White Texture",
            wgpu::TextureFormat::Rgba8Unorm,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let dummy_white_view = dummy_white_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Skinned Mesh Dummy White View"),
            ..Default::default()
        });

        let dummy_cube_texture = build::surface(
            device,
            "Skinned Mesh Dummy Cube Texture",
            wgpu::TextureFormat::Rgba8Unorm,
            [1, 1, 6],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let dummy_cube_view = dummy_cube_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Skinned Mesh Dummy Cube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let dummy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Skinned Mesh Dummy Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let ibl_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Skinned Mesh IBL Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let pipeline_layout = build::pipeline_layout(
            device,
            "Skinned Mesh Pipeline Layout",
            &[
                &uniform_bind_group_layout,
                &instance_bind_group_layout,
                &texture_bind_group_layout,
                &shadow_bind_group_layout,
            ],
        );

        let skinned_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkinnedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
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
                wgpu::VertexAttribute {
                    offset: 72,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Uint32x4,
                },
                wgpu::VertexAttribute {
                    offset: 88,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let render_pipeline =
            build::Pipe::new(device, "Skinned Mesh Pipeline", &skinned_mesh_shader)
                .layout(&pipeline_layout)
                .buffers(std::slice::from_ref(&skinned_vertex_layout))
                .targets(&[
                    Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::RED,
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
                ])
                .depth(build::depth_state(depth_format, true))
                .vertex("vs_main")
                .fragment("fs_main")
                .make();

        let skinned_mesh_oit_shader = composer.compile_wgsl_with_defs(
            device,
            "skinned_mesh_oit.wgsl",
            include_str!("../../../../shaders/skinned_mesh_oit.wgsl"),
            material_shader_defs,
        );

        let oit_render_pipeline = build::Pipe::new(
            device,
            "Skinned Mesh OIT Pipeline",
            &skinned_mesh_oit_shader,
        )
        .layout(&pipeline_layout)
        .buffers(std::slice::from_ref(&skinned_vertex_layout))
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
                format: wgpu::TextureFormat::R16Float,
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

        let blend_opaque_depth_prepass_pipeline = build::Pipe::new(
            device,
            "Skinned Mesh Blend Opaque Depth Prepass Pipeline",
            &skinned_mesh_oit_shader,
        )
        .layout(&pipeline_layout)
        .buffers(std::slice::from_ref(&skinned_vertex_layout))
        .depth(build::depth_nearer(depth_format))
        .vertex("vs_main")
        .fragment("fs_blend_opaque_prepass")
        .make();

        let oit_composite_shader = composer.compile_wgsl(
            device,
            "oit_composite.wgsl",
            include_str!("../../../../shaders/oit_composite.wgsl"),
        );

        let oit_composite_bind_group_layout = build::bind_layout(
            device,
            "Skinned Mesh OIT Composite Bind Group Layout",
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::painted_at(2),
                build::filtering_at(3),
            ],
        );

        let oit_composite_pipeline_layout = build::pipeline_layout(
            device,
            "Skinned Mesh OIT Composite Pipeline Layout",
            &[&oit_composite_bind_group_layout],
        );

        let oit_composite_pipeline = build::Pipe::new(
            device,
            "Skinned Mesh OIT Composite Pipeline",
            &oit_composite_shader,
        )
        .layout(&oit_composite_pipeline_layout)
        .target(wgpu::ColorTargetState {
            format: color_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })
        .vertex("vs_main")
        .fragment("fs_main")
        .make();

        let cluster_compute = nightshade_wgpu_shading::backend::cluster::ClusterCompute::new(build);
        let cluster_bounds_pipeline = cluster_compute.bounds_pipeline;
        let cluster_assign_pipeline = cluster_compute.assign_pipeline;
        let cluster_bounds_bind_group_layout = cluster_compute.bounds_bind_group_layout;
        let cluster_assign_bind_group_layout = cluster_compute.assign_bind_group_layout;

        let oit_accum_texture = build::surface(
            device,
            "Skinned Mesh OIT Accum Texture",
            wgpu::TextureFormat::Rgba16Float,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let oit_accum_view = oit_accum_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let oit_reveal_texture = build::surface(
            device,
            "Skinned Mesh OIT Reveal Texture",
            wgpu::TextureFormat::R16Float,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let oit_reveal_view =
            oit_reveal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let oit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Skinned Mesh OIT Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let skinned_vertex_buffer = build::buffer(
            device,
            "Skinned Vertex Buffer",
            (std::mem::size_of::<SkinnedVertex>() * INITIAL_SKINNED_VERTEX_COUNT) as u64,
            wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let skinned_index_buffer = build::buffer(
            device,
            "Skinned Index Buffer",
            (std::mem::size_of::<u32>() * INITIAL_SKINNED_INDEX_COUNT) as u64,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        );

        Self {
            clusters: SkinnedClusters {
                bounds_buffer: cluster_bounds_buffer,
                view_matrix_buffer,
                bounds_pipeline: cluster_bounds_pipeline,
                assign_pipeline: cluster_assign_pipeline,
                bounds_layout: cluster_bounds_bind_group_layout,
                assign_layout: cluster_assign_bind_group_layout,
            },
            materials: SkinnedMaterials {
                registered_textures: HashMap::new(),
                array_srgb_view: None,
                array_linear_view: None,
                array_samplers: None,
                bind_group: None,
                bindless_max: material_bindless_max,
                layer_map: HashMap::new(),
            },
            morph: SkinnedMorph {
                buffer: morph_displacement_buffer,
                buffer_size: morph_displacement_buffer_size,
                offset: 0,
            },
            buffers: SkinnedBuffers {
                vertex: skinned_vertex_buffer,
                index: skinned_index_buffer,
                vertex_size: (std::mem::size_of::<SkinnedVertex>() * INITIAL_SKINNED_VERTEX_COUNT)
                    as u64,
                index_size: (std::mem::size_of::<u32>() * INITIAL_SKINNED_INDEX_COUNT) as u64,
                vertex_offset: 0,
                index_offset: 0,
                ids: HashMap::new(),
                data: Vec::new(),
            },
            images: SkinnedImages {
                white: dummy_white_view,
                cube: dummy_cube_view,
                sampler: dummy_sampler,
                brdf_lut: brdf_lut_view,
                irradiance: None,
                prefiltered: None,
                irradiance_b: None,
                prefiltered_b: None,
            },
            bindings: SkinnedBindings {
                uniforms: uniform_buffer,
                uniform_group: uniform_bind_group,
                instance_layout: instance_bind_group_layout,
                texture_layout: texture_bind_group_layout,
                shadow_group: shadow_bind_group,
                shadow_layout: shadow_bind_group_layout,
                shadow_sampler,
                shadow_pcf_sampler,
                spotlight_shadows: spotlight_shadow_buffer,
                point_shadows: point_shadow_buffer,
                point_shadow_sampler,
                point_shadow_cubemap: dummy_point_shadow_view,
                ibl_sampler,
            },
            compute: SkinnedCompute {
                cull_pipeline,
                cull_layout: cull_bind_group_layout,
                bounds_pipeline: bounds_compute_pipeline,
                bounds_layout: bounds_compute_bind_group_layout,
                bounds_group: None,
                bounds_dispatch: 0,
                batch_clear: batch_clear_pipeline,
                batch_count: batch_count_pipeline,
                batch_build: batch_build_pipeline,
                batch_assign: batch_assign_pipeline,
                batch_layout: batch_bind_group_layout,
                batch_params: batch_params_buffer,
                batch_geo: batch_mesh_geo_buffer,
                batch_geo_size: batch_mesh_geo_buffer_size,
                batching_enabled: false,
            },
            oit: SkinnedOit {
                render_pipeline: oit_render_pipeline,
                composite_pipeline: oit_composite_pipeline,
                composite_layout: oit_composite_bind_group_layout,
                accum_texture: oit_accum_texture,
                accum_view: oit_accum_view,
                reveal_texture: oit_reveal_texture,
                reveal_view: oit_reveal_view,
                sampler: oit_sampler,
                texture_size: (1, 1),
                resource_cache: Vec::new(),
            },
            skinning: SkinnedSkinning {
                pipeline: skinning_compute_pipeline,
                bind_group: skinning_compute_bind_group,
                layout: skinning_compute_bind_group_layout,
                bone_transforms: bone_transforms_buffer,
                publish_pending: true,
                inverse_binds: inverse_bind_matrices_buffer,
                joints: joint_matrices_buffer,
                previous_joints: prev_joint_matrices_buffer,
                skin_data: skin_data_buffer,
                palette_generation: 0,
                skinning_generation: 0,
                bone_generation: 0,
                bone_transforms_size: INITIAL_MAX_TOTAL_BONES,
                inverse_binds_size: MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS,
                joints_size: MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS,
                skin_data_size: INITIAL_MAX_SKINS,
                joints_to_dispatch: 0,
                cache: SkinningCache::default(),
            },
            animation: super::super::animation::SkinnedAnimationGpu::new(build),
            ik: super::super::ik::SkinnedIkGpu::new(build),
            multi_ik: super::super::multi_ik::SkinnedMultiIkGpu::new(build),
            aim: super::super::aim::SkinnedAimGpu::new(build),
            spring: super::super::spring::SkinnedSpringGpu::new(build),
            render_pipeline,
            blend_opaque_depth_prepass_pipeline,
            ltc,
            air,
            state: None,
            frame_counter: 0,
            dirty_skinned_mesh_names: HashSet::new(),
            last_mesh_cache_clear_generation: 0,
        }
    }
}
