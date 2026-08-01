use super::*;

pub(super) fn load_gltf_primitive(
    primitive: &Primitive,
    buffers: &[buffer::Data],
    default_morph_weights: Option<&[f32]>,
) -> Option<Mesh> {
    let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| &b.0[..]));

    let positions: Vec<Vec3> = reader
        .read_positions()?
        .map(|p| vec3(p[0], p[1], p[2]))
        .collect();

    if positions.is_empty() {
        tracing::warn!("glTF primitive has no positions, skipping");
        return None;
    }

    let normals: Vec<Vec3> = if let Some(normals_iter) = reader.read_normals() {
        let normals: Vec<_> = normals_iter.map(|n| vec3(n[0], n[1], n[2])).collect();

        if normals.len() != positions.len() {
            tracing::warn!("glTF primitive has mismatched normal count, generating normals");
            generate_normals(&positions, &reader.read_indices())
        } else {
            normals
        }
    } else {
        generate_normals(&positions, &reader.read_indices())
    };

    let mut texcoords: Vec<Vec2> = vec![vec2(0.0, 0.0); positions.len()];

    if let Some(tex_iter) = reader.read_tex_coords(0) {
        let coords: Vec<_> = tex_iter.into_f32().map(|t| vec2(t[0], t[1])).collect();

        if coords.len() == positions.len() {
            texcoords = coords;
        } else {
            tracing::warn!("glTF primitive has mismatched texcoord count, using defaults");
        }
    }

    let mut texcoords_1: Vec<Vec2> = vec![vec2(0.0, 0.0); positions.len()];

    if let Some(tex_iter) = reader.read_tex_coords(1) {
        let coords: Vec<_> = tex_iter.into_f32().map(|t| vec2(t[0], t[1])).collect();

        if coords.len() == positions.len() {
            texcoords_1 = coords;
        }
    }

    let colors: Vec<[f32; 4]> = if let Some(color_iter) = reader.read_colors(0) {
        let color_data: Vec<[f32; 4]> = color_iter.into_rgba_f32().collect();
        if color_data.len() == positions.len() {
            color_data
        } else {
            vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
        }
    } else {
        vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
    };

    let raw_indices: Vec<u32> = if let Some(indices_iter) = reader.read_indices() {
        indices_iter.into_u32().collect()
    } else {
        (0..positions.len() as u32).collect()
    };

    if raw_indices.is_empty() {
        tracing::warn!("glTF primitive has no indices, skipping");
        return None;
    }

    let has_normal_texture = primitive.material().normal_texture().is_some();

    let raw_tangents: Option<Vec<[f32; 4]>> =
        reader.read_tangents().map(|iter| iter.collect::<Vec<_>>());
    let tangents_were_provided = raw_tangents
        .as_ref()
        .is_some_and(|tangents| tangents.len() == positions.len());
    let tangents: Vec<[f32; 4]> = match raw_tangents {
        Some(tangent_data) if tangent_data.len() == positions.len() => tangent_data,
        _ => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
    };

    let has_joints = reader.read_joints(0).is_some();
    let joint_indices_opt: Option<Vec<[u32; 4]>> = if has_joints {
        let mut joint_indices: Vec<[u32; 4]> = vec![[0, 0, 0, 0]; positions.len()];
        if let Some(joints_iter) = reader.read_joints(0) {
            let joints: Vec<[u16; 4]> = joints_iter.into_u16().collect();
            if joints.len() == positions.len() {
                for (vertex_index, joint) in joints.iter().enumerate() {
                    joint_indices[vertex_index] = [
                        joint[0] as u32,
                        joint[1] as u32,
                        joint[2] as u32,
                        joint[3] as u32,
                    ];
                }
            }
        }
        Some(joint_indices)
    } else {
        None
    };

    let joint_weights_opt: Option<Vec<[f32; 4]>> = if has_joints {
        let joint_weights: Vec<[f32; 4]> = if let Some(weights_iter) = reader.read_weights(0) {
            let weights: Vec<[f32; 4]> = weights_iter.into_f32().collect();
            if weights.len() == positions.len() {
                weights
                    .into_iter()
                    .map(|w| {
                        let sum = w[0] + w[1] + w[2] + w[3];
                        if sum > 0.0 {
                            [w[0] / sum, w[1] / sum, w[2] / sum, w[3] / sum]
                        } else {
                            [1.0, 0.0, 0.0, 0.0]
                        }
                    })
                    .collect()
            } else {
                vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]
            }
        } else {
            vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]
        };
        Some(joint_weights)
    } else {
        None
    };

    let mode = primitive.mode();
    let bbox_extent = bounding_box_extent(&positions);
    let mut attributes = PrimitiveAttributes {
        positions,
        normals,
        texcoords,
        texcoords_1,
        colors,
        tangents,
        joint_indices: joint_indices_opt,
        joint_weights: joint_weights_opt,
    };
    let indices = convert_primitive_to_triangles(mode, &raw_indices, &mut attributes, bbox_extent);

    if indices.is_empty() {
        tracing::warn!("glTF primitive has no triangles after mode conversion, skipping");
        return None;
    }

    if !tangents_were_provided
        && has_normal_texture
        && matches!(
            mode,
            gltf::mesh::Mode::Triangles
                | gltf::mesh::Mode::TriangleStrip
                | gltf::mesh::Mode::TriangleFan
        )
    {
        attributes.tangents = generate_mikktspace_tangents(
            &attributes.positions,
            &attributes.normals,
            &attributes.texcoords,
            &indices,
        );
    }

    let PrimitiveAttributes {
        positions,
        normals,
        texcoords,
        texcoords_1,
        colors,
        tangents,
        joint_indices: joint_indices_opt,
        joint_weights: joint_weights_opt,
    } = attributes;

    let skin_data = if let (Some(joint_indices), Some(joint_weights)) =
        (joint_indices_opt, joint_weights_opt)
    {
        let skinned_vertices: Vec<SkinnedVertex> = positions
            .iter()
            .zip(normals.iter())
            .zip(texcoords.iter())
            .zip(texcoords_1.iter())
            .zip(tangents.iter())
            .zip(colors.iter())
            .zip(joint_indices.iter())
            .zip(joint_weights.iter())
            .map(
                |(
                    (
                        (((((position, normal), tex_coords), tex_coords_1), tangent), color),
                        joint_indices,
                    ),
                    joint_weights,
                )| {
                    SkinnedVertex {
                        position: [position.x, position.y, position.z],
                        normal: [normal.x, normal.y, normal.z],
                        tex_coords: [tex_coords.x, tex_coords.y],
                        tex_coords_1: [tex_coords_1.x, tex_coords_1.y],
                        tangent: *tangent,
                        color: *color,
                        joint_indices: *joint_indices,
                        joint_weights: *joint_weights,
                    }
                },
            )
            .collect();
        Some(SkinData::new(skinned_vertices))
    } else {
        None
    };

    if positions.len() != normals.len() || positions.len() != texcoords.len() {
        tracing::error!("glTF primitive has inconsistent vertex attribute lengths");
        return None;
    }

    let vertices: Vec<Vertex> = positions
        .iter()
        .zip(&normals)
        .zip(&texcoords)
        .zip(&texcoords_1)
        .zip(&tangents)
        .zip(&colors)
        .map(
            |(((((position, normal), tex_coords), tex_coords_1), tangent), color)| Vertex {
                position: [position.x, position.y, position.z],
                normal: [normal.x, normal.y, normal.z],
                tex_coords: [tex_coords.x, tex_coords.y],
                tex_coords_1: [tex_coords_1.x, tex_coords_1.y],
                tangent: *tangent,
                color: *color,
            },
        )
        .collect();

    let mut min = nalgebra_glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = nalgebra_glm::vec3(f32::MIN, f32::MIN, f32::MIN);

    for position in &positions {
        min.x = min.x.min(position.x);
        min.y = min.y.min(position.y);
        min.z = min.z.min(position.z);

        max.x = max.x.max(position.x);
        max.y = max.y.max(position.y);
        max.z = max.z.max(position.z);
    }

    let (center, half_extents, sphere_radius) = if positions.is_empty() {
        (
            nalgebra_glm::Vec3::zeros(),
            nalgebra_glm::Vec3::zeros(),
            0.0,
        )
    } else {
        let center = (min + max) * 0.5;
        let half_extents = (max - min) * 0.5;
        let sphere_radius = nalgebra_glm::length(&half_extents);
        (center, half_extents, sphere_radius)
    };

    let bounding_volume = nightshade_render::data::bounding_volume::BoundingVolume {
        obb: nightshade_render::data::bounding_volume::OrientedBoundingBox {
            center,
            half_extents,
            orientation: nalgebra_glm::quat_identity(),
        },
        sphere_radius,
    };

    let morph_targets = load_morph_targets(
        primitive,
        buffers,
        &positions,
        &normals,
        default_morph_weights,
    );

    let mut mesh = Mesh::with_bounding_volume(vertices, indices, bounding_volume);
    mesh.skin_data = skin_data;
    mesh.morph_targets = morph_targets;

    Some(mesh)
}

pub(super) struct PrimitiveAttributes {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    texcoords: Vec<Vec2>,
    texcoords_1: Vec<Vec2>,
    colors: Vec<[f32; 4]>,
    tangents: Vec<[f32; 4]>,
    joint_indices: Option<Vec<[u32; 4]>>,
    joint_weights: Option<Vec<[f32; 4]>>,
}

pub(super) fn bounding_box_extent(positions: &[Vec3]) -> f32 {
    if positions.is_empty() {
        return 1.0;
    }
    let mut min = vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = vec3(f32::MIN, f32::MIN, f32::MIN);
    for position in positions {
        min.x = min.x.min(position.x);
        min.y = min.y.min(position.y);
        min.z = min.z.min(position.z);
        max.x = max.x.max(position.x);
        max.y = max.y.max(position.y);
        max.z = max.z.max(position.z);
    }
    let extent = max - min;
    extent.x.max(extent.y).max(extent.z).max(1.0e-6)
}

pub(super) fn convert_primitive_to_triangles(
    mode: gltf::mesh::Mode,
    raw_indices: &[u32],
    attributes: &mut PrimitiveAttributes,
    bbox_extent: f32,
) -> Vec<u32> {
    use gltf::mesh::Mode;
    match mode {
        Mode::Triangles => {
            let triangle_count = raw_indices.len() / 3;
            raw_indices[..triangle_count * 3].to_vec()
        }
        Mode::TriangleStrip => triangle_strip_to_list(raw_indices),
        Mode::TriangleFan => triangle_fan_to_list(raw_indices),
        Mode::Points => points_to_triangles(raw_indices, attributes, bbox_extent * 0.025),
        Mode::Lines => {
            let segments: Vec<(u32, u32)> = raw_indices
                .as_chunks::<2>()
                .0
                .iter()
                .map(|chunk| (chunk[0], chunk[1]))
                .collect();
            line_segments_to_triangles(&segments, attributes, bbox_extent * 0.005)
        }
        Mode::LineStrip => {
            if raw_indices.len() < 2 {
                return Vec::new();
            }
            let segments: Vec<(u32, u32)> = (0..raw_indices.len() - 1)
                .map(|segment_index| (raw_indices[segment_index], raw_indices[segment_index + 1]))
                .collect();
            line_segments_to_triangles(&segments, attributes, bbox_extent * 0.005)
        }
        Mode::LineLoop => {
            if raw_indices.len() < 2 {
                return Vec::new();
            }
            let mut segments: Vec<(u32, u32)> = (0..raw_indices.len() - 1)
                .map(|segment_index| (raw_indices[segment_index], raw_indices[segment_index + 1]))
                .collect();
            segments.push((raw_indices[raw_indices.len() - 1], raw_indices[0]));
            line_segments_to_triangles(&segments, attributes, bbox_extent * 0.005)
        }
    }
}

pub(super) fn triangle_strip_to_list(strip_indices: &[u32]) -> Vec<u32> {
    if strip_indices.len() < 3 {
        return Vec::new();
    }
    let mut output = Vec::with_capacity((strip_indices.len() - 2) * 3);
    for triangle_offset in 0..strip_indices.len() - 2 {
        let (a, b, c) = if triangle_offset.is_multiple_of(2) {
            (
                strip_indices[triangle_offset],
                strip_indices[triangle_offset + 1],
                strip_indices[triangle_offset + 2],
            )
        } else {
            (
                strip_indices[triangle_offset + 1],
                strip_indices[triangle_offset],
                strip_indices[triangle_offset + 2],
            )
        };
        if a == b || b == c || a == c {
            continue;
        }
        output.push(a);
        output.push(b);
        output.push(c);
    }
    output
}

pub(super) fn triangle_fan_to_list(fan_indices: &[u32]) -> Vec<u32> {
    if fan_indices.len() < 3 {
        return Vec::new();
    }
    let mut output = Vec::with_capacity((fan_indices.len() - 2) * 3);
    let center = fan_indices[0];
    for triangle_offset in 1..fan_indices.len() - 1 {
        let edge_start = fan_indices[triangle_offset];
        let edge_end = fan_indices[triangle_offset + 1];
        if center == edge_start || edge_start == edge_end || center == edge_end {
            continue;
        }
        output.push(center);
        output.push(edge_start);
        output.push(edge_end);
    }
    output
}

pub(super) fn points_to_triangles(
    point_indices: &[u32],
    attributes: &mut PrimitiveAttributes,
    radius: f32,
) -> Vec<u32> {
    let octahedron_offsets = [
        vec3(radius, 0.0, 0.0),
        vec3(-radius, 0.0, 0.0),
        vec3(0.0, radius, 0.0),
        vec3(0.0, -radius, 0.0),
        vec3(0.0, 0.0, radius),
        vec3(0.0, 0.0, -radius),
    ];
    let octahedron_triangles: [[u32; 3]; 8] = [
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [2, 0, 5],
        [1, 2, 5],
        [3, 1, 5],
        [0, 3, 5],
    ];

    let mut new_attributes = PrimitiveAttributes {
        positions: Vec::new(),
        normals: Vec::new(),
        texcoords: Vec::new(),
        texcoords_1: Vec::new(),
        colors: Vec::new(),
        tangents: Vec::new(),
        joint_indices: attributes.joint_indices.as_ref().map(|_| Vec::new()),
        joint_weights: attributes.joint_weights.as_ref().map(|_| Vec::new()),
    };
    let mut output_indices = Vec::new();

    for &point_index in point_indices {
        let source = point_index as usize;
        if source >= attributes.positions.len() {
            continue;
        }
        let center = attributes.positions[source];
        let texcoord = attributes.texcoords[source];
        let texcoord_1 = attributes.texcoords_1[source];
        let color = attributes.colors[source];
        let tangent = attributes.tangents[source];
        let source_joint_indices = attributes
            .joint_indices
            .as_ref()
            .map(|joints| joints[source]);
        let source_joint_weights = attributes
            .joint_weights
            .as_ref()
            .map(|weights| weights[source]);

        let base = new_attributes.positions.len() as u32;
        for offset in &octahedron_offsets {
            new_attributes.positions.push(center + offset);
            new_attributes.normals.push(nalgebra_glm::normalize(offset));
            new_attributes.texcoords.push(texcoord);
            new_attributes.texcoords_1.push(texcoord_1);
            new_attributes.colors.push(color);
            new_attributes.tangents.push(tangent);
            if let (Some(target), Some(value)) =
                (new_attributes.joint_indices.as_mut(), source_joint_indices)
            {
                target.push(value);
            }
            if let (Some(target), Some(value)) =
                (new_attributes.joint_weights.as_mut(), source_joint_weights)
            {
                target.push(value);
            }
        }
        for triangle in &octahedron_triangles {
            output_indices.push(base + triangle[0]);
            output_indices.push(base + triangle[1]);
            output_indices.push(base + triangle[2]);
        }
    }

    *attributes = new_attributes;
    output_indices
}

pub(super) fn line_segments_to_triangles(
    segments: &[(u32, u32)],
    attributes: &mut PrimitiveAttributes,
    radius: f32,
) -> Vec<u32> {
    let prism_triangles: [[u32; 3]; 12] = [
        [0, 3, 2],
        [0, 2, 1],
        [4, 5, 6],
        [4, 6, 7],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
    ];

    let mut new_attributes = PrimitiveAttributes {
        positions: Vec::new(),
        normals: Vec::new(),
        texcoords: Vec::new(),
        texcoords_1: Vec::new(),
        colors: Vec::new(),
        tangents: Vec::new(),
        joint_indices: attributes.joint_indices.as_ref().map(|_| Vec::new()),
        joint_weights: attributes.joint_weights.as_ref().map(|_| Vec::new()),
    };
    let mut output_indices = Vec::new();

    for &(start_index, end_index) in segments {
        let start = start_index as usize;
        let end = end_index as usize;
        if start >= attributes.positions.len() || end >= attributes.positions.len() {
            continue;
        }
        let p0 = attributes.positions[start];
        let p1 = attributes.positions[end];
        let direction = p1 - p0;
        if nalgebra_glm::length(&direction) < 1.0e-8 {
            continue;
        }
        let direction_norm = nalgebra_glm::normalize(&direction);
        let up_candidate = if direction_norm.y.abs() < 0.99 {
            vec3(0.0, 1.0, 0.0)
        } else {
            vec3(1.0, 0.0, 0.0)
        };
        let side_a =
            nalgebra_glm::normalize(&nalgebra_glm::cross(&direction_norm, &up_candidate)) * radius;
        let side_b =
            nalgebra_glm::normalize(&nalgebra_glm::cross(&direction_norm, &side_a)) * radius;

        let corners = [
            p0 - side_a - side_b,
            p0 + side_a - side_b,
            p0 + side_a + side_b,
            p0 - side_a + side_b,
            p1 - side_a - side_b,
            p1 + side_a - side_b,
            p1 + side_a + side_b,
            p1 - side_a + side_b,
        ];

        let texcoord = attributes.texcoords[start];
        let texcoord_1 = attributes.texcoords_1[start];
        let color = attributes.colors[start];
        let tangent = attributes.tangents[start];
        let source_joint_indices = attributes
            .joint_indices
            .as_ref()
            .map(|joints| joints[start]);
        let source_joint_weights = attributes
            .joint_weights
            .as_ref()
            .map(|weights| weights[start]);

        let segment_center = p0 + (p1 - p0) * 0.5;
        let base = new_attributes.positions.len() as u32;
        for corner in &corners {
            let outward = corner - segment_center;
            let normal = if nalgebra_glm::length(&outward) > 1.0e-6 {
                nalgebra_glm::normalize(&outward)
            } else {
                direction_norm
            };
            new_attributes.positions.push(*corner);
            new_attributes.normals.push(normal);
            new_attributes.texcoords.push(texcoord);
            new_attributes.texcoords_1.push(texcoord_1);
            new_attributes.colors.push(color);
            new_attributes.tangents.push(tangent);
            if let (Some(target), Some(value)) =
                (new_attributes.joint_indices.as_mut(), source_joint_indices)
            {
                target.push(value);
            }
            if let (Some(target), Some(value)) =
                (new_attributes.joint_weights.as_mut(), source_joint_weights)
            {
                target.push(value);
            }
        }
        for triangle in &prism_triangles {
            output_indices.push(base + triangle[0]);
            output_indices.push(base + triangle[1]);
            output_indices.push(base + triangle[2]);
        }
    }

    *attributes = new_attributes;
    output_indices
}

pub(super) fn load_morph_targets(
    primitive: &Primitive,
    buffers: &[buffer::Data],
    positions: &[Vec3],
    normals: &[Vec3],
    default_weights: Option<&[f32]>,
) -> Option<MorphTargetData> {
    let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| &b.0[..]));
    let vertex_count = positions.len();

    let morph_target_iter = reader.read_morph_targets();
    let targets: Vec<MorphTarget> = morph_target_iter
        .filter_map(|morph_target| {
            let position_displacements: Vec<[f32; 3]> =
                morph_target.0?.map(|p| [p[0], p[1], p[2]]).collect();

            if position_displacements.len() != vertex_count {
                return None;
            }

            let mut target = MorphTarget::new(position_displacements);

            if let Some(normals_iter) = morph_target.1 {
                let normal_displacements: Vec<[f32; 3]> =
                    normals_iter.map(|n| [n[0], n[1], n[2]]).collect();
                if normal_displacements.len() == vertex_count {
                    target = target.with_normals(normal_displacements);
                }
            }

            if let Some(tangents_iter) = morph_target.2 {
                let tangent_displacements: Vec<[f32; 3]> =
                    tangents_iter.map(|t| [t[0], t[1], t[2]]).collect();
                if tangent_displacements.len() == vertex_count {
                    target = target.with_tangents(tangent_displacements);
                }
            }

            Some(target)
        })
        .collect();

    if targets.is_empty() {
        None
    } else {
        let base_positions: Vec<[f32; 3]> = positions.iter().map(|p| [p.x, p.y, p.z]).collect();
        let base_normals: Vec<[f32; 3]> = normals.iter().map(|n| [n.x, n.y, n.z]).collect();
        let weights = match default_weights {
            Some(w) => w.iter().take(targets.len()).copied().collect(),
            None => vec![0.0; targets.len()],
        };
        Some(
            MorphTargetData::new(targets)
                .with_base_data(base_positions, base_normals)
                .with_default_weights(weights),
        )
    }
}

pub(super) fn generate_normals(
    positions: &[Vec3],
    indices_reader: &Option<gltf::mesh::util::ReadIndices>,
) -> Vec<Vec3> {
    let mut normals = vec![Vec3::zeros(); positions.len()];
    let mut face_counts = vec![0u32; positions.len()];

    let indices: Vec<u32> = if let Some(indices_iter) = indices_reader {
        indices_iter.clone().into_u32().collect()
    } else {
        (0..positions.len() as u32).collect()
    };

    for triangle in indices.chunks(3) {
        if triangle.len() != 3 {
            continue;
        }

        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = nalgebra_glm::cross(&edge1, &edge2);

        let face_normal = if nalgebra_glm::length(&face_normal) > 1e-6 {
            nalgebra_glm::normalize(&face_normal)
        } else {
            vec3(0.0, 1.0, 0.0)
        };

        normals[i0] += face_normal;
        normals[i1] += face_normal;
        normals[i2] += face_normal;

        face_counts[i0] += 1;
        face_counts[i1] += 1;
        face_counts[i2] += 1;
    }

    for (index, normal) in normals.iter_mut().enumerate() {
        if face_counts[index] > 0 {
            *normal /= face_counts[index] as f32;
            let magnitude = nalgebra_glm::length(normal);
            if magnitude > 1e-6 {
                *normal = nalgebra_glm::normalize(normal);
            } else {
                *normal = vec3(0.0, 1.0, 0.0);
            }
        } else {
            *normal = vec3(0.0, 1.0, 0.0);
        }
    }

    normals
}
