use super::*;

pub(super) struct NodeConversionContext<'a> {
    pub(super) doc: &'a Document,
    pub(super) buffers: &'a [buffer::Data],
    pub(super) mesh_index_to_primitives: &'a HashMap<usize, Vec<(String, usize)>>,
    pub(super) mesh_index_to_primitive_variants: &'a MeshVariantMap,
    pub(super) materials: &'a [Material],
    pub(super) document_variant_names: &'a [String],
    pub(super) point_intensity_scale: f32,
}

pub(super) fn convert_node_to_prefab_nodes(
    context: &NodeConversionContext<'_>,
    node: &Node,
    output_nodes: &mut Vec<PrefabNode>,
) {
    let (translation, rotation, scale) = node.transform().decomposed();

    let quat = Quat::new(rotation[3], rotation[0], rotation[1], rotation[2]);

    let local_transform = LocalTransform {
        translation: Vec3::new(translation[0], translation[1], translation[2]),
        rotation: quat,
        scale: Vec3::new(scale[0], scale[1], scale[2]),
    };

    let node_name = node.name().map(|s| s.to_string());

    let node_skin_index = node.skin().map(|s| s.index());

    let instance_transforms = read_gpu_instancing_transforms(context.doc, context.buffers, node);

    if let Some(mesh) = node.mesh() {
        let mesh_variants = context.mesh_index_to_primitive_variants.get(&mesh.index());
        if let Some(primitives) = context.mesh_index_to_primitives.get(&mesh.index()) {
            if primitives.len() == 1 {
                let (mesh_name, material_index) = &primitives[0];
                let mut components = PrefabComponents::default();

                if let Some(name_str) = &node_name {
                    components.name = Some(Name(name_str.clone()));
                }

                components.render_mesh = Some(RenderMesh::new(mesh_name.clone()));

                components.material = Some(
                    context
                        .materials
                        .get(*material_index)
                        .cloned()
                        .unwrap_or_else(Material::default),
                );

                if let Some(mappings) = mesh_variants.and_then(|v| v.first()) {
                    components.material_variants = build_prefab_material_variants(
                        mappings,
                        context.materials,
                        context.document_variant_names,
                    );
                }

                if let Some(camera) = node.camera() {
                    components.camera = Some(load_gltf_camera(&camera));
                }

                if let Some(light) = node.light() {
                    components.light = Some(load_gltf_light(&light, context.point_intensity_scale));
                }

                components.visibility = Some(Visibility { visible: true });
                components.skin_index = node_skin_index;
                components.instances = instance_transforms.clone();

                let mut children = Vec::new();
                for child in node.children() {
                    convert_node_to_prefab_nodes(context, &child, &mut children);
                }

                output_nodes.push(PrefabNode {
                    local_transform,
                    components,
                    children,
                    node_index: Some(node.index()),
                });
            } else {
                let mut parent_components = PrefabComponents::default();
                if let Some(name_str) = &node_name {
                    parent_components.name = Some(Name(name_str.clone()));
                }

                if let Some(camera) = node.camera() {
                    parent_components.camera = Some(load_gltf_camera(&camera));
                }

                if let Some(light) = node.light() {
                    parent_components.light =
                        Some(load_gltf_light(&light, context.point_intensity_scale));
                }

                parent_components.skin_index = node_skin_index;

                let mut children = Vec::new();

                for (index, (mesh_name, material_index)) in primitives.iter().enumerate() {
                    let mut primitive_components = PrefabComponents {
                        name: Some(Name(format!("Primitive_{}", index))),
                        render_mesh: Some(RenderMesh::new(mesh_name.clone())),
                        visibility: Some(Visibility { visible: true }),
                        skin_index: node_skin_index,
                        instances: instance_transforms.clone(),
                        ..Default::default()
                    };

                    primitive_components.material = Some(
                        context
                            .materials
                            .get(*material_index)
                            .cloned()
                            .unwrap_or_else(Material::default),
                    );

                    if let Some(mappings) = mesh_variants.and_then(|v| v.get(index)) {
                        primitive_components.material_variants = build_prefab_material_variants(
                            mappings,
                            context.materials,
                            context.document_variant_names,
                        );
                    }

                    children.push(PrefabNode {
                        local_transform: LocalTransform::default(),
                        components: primitive_components,
                        children: Vec::new(),
                        node_index: None,
                    });
                }

                for child in node.children() {
                    convert_node_to_prefab_nodes(context, &child, &mut children);
                }

                output_nodes.push(PrefabNode {
                    local_transform,
                    components: parent_components,
                    children,
                    node_index: Some(node.index()),
                });
            }
        } else {
            let mut components = PrefabComponents::default();
            if let Some(name_str) = &node_name {
                components.name = Some(Name(name_str.clone()));
            }

            if let Some(camera) = node.camera() {
                components.camera = Some(load_gltf_camera(&camera));
            }

            if let Some(light) = node.light() {
                components.light = Some(load_gltf_light(&light, context.point_intensity_scale));
            }

            components.skin_index = node_skin_index;

            let mut children = Vec::new();
            for child in node.children() {
                convert_node_to_prefab_nodes(context, &child, &mut children);
            }

            output_nodes.push(PrefabNode {
                local_transform,
                components,
                children,
                node_index: Some(node.index()),
            });
        }
    } else {
        let mut components = PrefabComponents::default();
        if let Some(name_str) = &node_name {
            components.name = Some(Name(name_str.clone()));
        }

        if let Some(camera) = node.camera() {
            components.camera = Some(load_gltf_camera(&camera));
        }

        if let Some(light) = node.light() {
            components.light = Some(load_gltf_light(&light, context.point_intensity_scale));
        }

        components.skin_index = node_skin_index;

        let mut children = Vec::new();
        for child in node.children() {
            convert_node_to_prefab_nodes(context, &child, &mut children);
        }

        output_nodes.push(PrefabNode {
            local_transform,
            components,
            children,
            node_index: Some(node.index()),
        });
    }
}

pub(super) fn load_gltf_light(
    light: &gltf::khr_lights_punctual::Light<'_>,
    point_intensity_scale: f32,
) -> Light {
    let kind = light.kind();
    let (light_type, inner_cone_angle, outer_cone_angle) = match kind {
        gltf::khr_lights_punctual::Kind::Directional => (LightType::Directional, 0.0, 0.0),
        gltf::khr_lights_punctual::Kind::Point => (LightType::Point, 0.0, 0.0),
        gltf::khr_lights_punctual::Kind::Spot {
            inner_cone_angle,
            outer_cone_angle,
        } => (LightType::Spot, inner_cone_angle, outer_cone_angle),
    };

    let color = light.color();
    let mut intensity = light.intensity();

    if !matches!(light_type, LightType::Directional) {
        intensity *= point_intensity_scale;
    }

    let range = light.range().unwrap_or(0.0);

    Light {
        light_type,
        color: Vec3::new(color[0], color[1], color[2]),
        intensity,
        range,
        inner_cone_angle,
        outer_cone_angle,
        cast_shadows: matches!(light_type, LightType::Directional),
        shadow_bias: 0.0005,
        shadow_resolution: 0,
        shadow_distance: 0.0,
        cookie_texture: None,
        ..Default::default()
    }
}

pub(super) fn point_light_intensity_scale_for_generator(generator: Option<&str>) -> f32 {
    let Some(generator) = generator else {
        return 1.0;
    };
    if !generator.contains("Khronos glTF Blender I/O") {
        return 1.0;
    }
    let Some(version_part) = generator.rsplit('v').next() else {
        return 1.0;
    };
    let Some(major_str) = version_part.split('.').next() else {
        return 1.0;
    };
    let Ok(major) = major_str.parse::<u32>() else {
        return 1.0;
    };
    if major < 3 {
        1.0 / (4.0 * std::f32::consts::PI)
    } else {
        1.0
    }
}

pub(super) fn load_gltf_camera(camera: &gltf::Camera<'_>) -> Camera {
    let projection = match camera.projection() {
        gltf::camera::Projection::Perspective(persp) => {
            Projection::Perspective(PerspectiveCamera {
                aspect_ratio: persp.aspect_ratio(),
                y_fov_rad: persp.yfov(),
                z_far: persp.zfar(),
                z_near: persp.znear(),
            })
        }
        gltf::camera::Projection::Orthographic(ortho) => {
            Projection::Orthographic(OrthographicCamera {
                x_mag: Some(ortho.xmag()),
                y_mag: ortho.ymag(),
                z_far: ortho.zfar(),
                z_near: ortho.znear(),
            })
        }
    };

    Camera {
        projection,
        smoothing: None,
    }
}
