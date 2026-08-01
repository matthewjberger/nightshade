use crate::data::geometry::{Mesh, Vertex};
use crate::data::meshlet::{
    ERROR_UNBOUNDED, MESHLET_GROUP_SIZE, MESHLET_MAX_LEVELS, MESHLET_MAX_TRIANGLES,
    MESHLET_MAX_VERTICES, Meshlet, MeshletMesh,
};

const CONE_WEIGHT: f32 = 0.5;

const ROUND_ERROR_LIMIT: f32 = 1.0;

pub fn build_meshlets(mesh: &Mesh) -> MeshletMesh {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return MeshletMesh::default();
    }

    let Ok(adapter) = meshopt::VertexDataAdapter::new(
        bytemuck::cast_slice(&mesh.vertices),
        std::mem::size_of::<Vertex>(),
        0,
    ) else {
        return MeshletMesh::default();
    };
    let scale = meshopt::simplify_scale(&adapter);

    let mut built = MeshletMesh::default();
    let ordered = meshopt::optimize_vertex_cache(&mesh.indices, mesh.vertices.len());
    let mut level = split_into_meshlets(&ordered, &adapter, &mut built, 0.0, None);

    for _ in 0..MESHLET_MAX_LEVELS {
        if level.len() < 2 {
            break;
        }
        let next = coarser_level(&level, &mut built, &adapter, mesh.vertices.len(), scale);
        if next.is_empty() || next.len() >= level.len() {
            break;
        }
        level = next;
    }

    built
}

struct Cluster {
    record: usize,
    indices: Vec<u32>,
}

fn split_into_meshlets(
    indices: &[u32],
    adapter: &meshopt::VertexDataAdapter<'_>,
    into: &mut MeshletMesh,
    error: f32,
    group: Option<[f32; 4]>,
) -> Vec<Cluster> {
    let built = meshopt::build_meshlets(
        indices,
        adapter,
        MESHLET_MAX_VERTICES,
        MESHLET_MAX_TRIANGLES,
        CONE_WEIGHT,
    );

    let vertex_base = into.vertices.len() as u32;
    let triangle_base = into.triangles.len() as u32;
    into.vertices.extend_from_slice(&built.vertices);
    into.triangles.extend_from_slice(&built.triangles);

    let mut clusters = Vec::with_capacity(built.len());
    for index in 0..built.len() {
        let meshlet = built.get(index);
        let bounds = meshopt::compute_meshlet_bounds(meshlet, adapter);
        let record = built.meshlets[index];
        let held: Vec<u32> = meshlet
            .triangles
            .iter()
            .map(|corner| meshlet.vertices[*corner as usize])
            .collect();
        let sphere = [
            bounds.center[0],
            bounds.center[1],
            bounds.center[2],
            bounds.radius,
        ];
        clusters.push(Cluster {
            record: into.meshlets.len(),
            indices: held,
        });
        into.meshlets.push(Meshlet {
            bounds: sphere,
            cone: [
                bounds.cone_axis[0],
                bounds.cone_axis[1],
                bounds.cone_axis[2],
                bounds.cone_cutoff,
            ],
            vertex_offset: vertex_base + record.vertex_offset,
            triangle_offset: triangle_base + record.triangle_offset,
            vertex_count: record.vertex_count,
            triangle_count: record.triangle_count,
            group: group.unwrap_or(sphere),
            group_error: error,
            parent: [0.0; 4],
            parent_error: ERROR_UNBOUNDED,
        });
    }
    clusters
}

struct Simplified {
    indices: Vec<u32>,
    error: f32,
    sphere: [f32; 4],
    children: Vec<usize>,
}

fn coarser_level(
    level: &[Cluster],
    built: &mut MeshletMesh,
    adapter: &meshopt::VertexDataAdapter<'_>,
    vertices: usize,
    scale: f32,
) -> Vec<Cluster> {
    let groups = gather_groups(level, built);
    let held = locked_vertices(level, &groups, vertices);

    let mut made: Vec<Simplified> = Vec::new();
    for group in &groups {
        let mut indices = Vec::new();
        let mut children = Vec::new();
        let mut worst = 0.0f32;
        for member in group {
            indices.extend_from_slice(&level[*member].indices);
            children.push(level[*member].record);
            worst = worst.max(built.meshlets[level[*member].record].group_error);
        }
        if indices.len() < 6 {
            continue;
        }

        let target = (indices.len() / 3 / 2).max(1) * 3;
        let mut moved = 0.0f32;
        let simplified = meshopt::simplify_with_locks(
            &indices,
            adapter,
            &held,
            target,
            ROUND_ERROR_LIMIT,
            meshopt::SimplifyOptions::LockBorder,
            Some(&mut moved),
        );
        if simplified.len() >= indices.len() || simplified.len() < 3 {
            continue;
        }
        made.push(Simplified {
            indices: simplified,
            error: worst + moved * scale,
            sphere: group_sphere(group, level, built),
            children,
        });
    }

    let mut next = Vec::new();
    for group in made {
        for child in &group.children {
            let record = &mut built.meshlets[*child];
            record.parent = group.sphere;
            record.parent_error = group.error;
        }
        next.extend(split_into_meshlets(
            &group.indices,
            adapter,
            built,
            group.error,
            Some(group.sphere),
        ));
    }
    next
}

fn group_sphere(group: &[usize], level: &[Cluster], built: &MeshletMesh) -> [f32; 4] {
    let mut lowest = [f32::MAX; 3];
    let mut highest = [f32::MIN; 3];
    for member in group {
        let bounds = built.meshlets[level[*member].record].bounds;
        for axis in 0..3 {
            lowest[axis] = lowest[axis].min(bounds[axis] - bounds[3]);
            highest[axis] = highest[axis].max(bounds[axis] + bounds[3]);
        }
    }
    let center = [
        (lowest[0] + highest[0]) * 0.5,
        (lowest[1] + highest[1]) * 0.5,
        (lowest[2] + highest[2]) * 0.5,
    ];
    let radius = (0..3)
        .map(|axis| (highest[axis] - lowest[axis]) * 0.5)
        .fold(0.0f32, |total, half| total + half * half)
        .sqrt();
    [center[0], center[1], center[2], radius]
}

fn gather_groups(level: &[Cluster], built: &MeshletMesh) -> Vec<Vec<usize>> {
    let keys: Vec<u64> = level
        .iter()
        .map(|cluster| morton_of(built.meshlets[cluster.record].bounds))
        .collect();
    let mut order: Vec<usize> = (0..level.len()).collect();
    order.sort_unstable_by_key(|index| keys[*index]);
    order
        .chunks(MESHLET_GROUP_SIZE)
        .map(<[usize]>::to_vec)
        .collect()
}

fn morton_of(bounds: [f32; 4]) -> u64 {
    let spread = |value: f32| -> u64 {
        let mut bits = ((value * 512.0) + 1_048_576.0).clamp(0.0, 2_097_151.0) as u64;
        bits = (bits | bits << 32) & 0x001f_0000_0000_ffff;
        bits = (bits | bits << 16) & 0x001f_0000_ff00_00ff;
        bits = (bits | bits << 8) & 0x100f_00f0_0f00_f00f;
        bits = (bits | bits << 4) & 0x10c3_0c30_c30c_30c3;
        bits = (bits | bits << 2) & 0x1249_2492_4924_9249;
        bits
    };
    spread(bounds[0]) | spread(bounds[1]) << 1 | spread(bounds[2]) << 2
}

fn locked_vertices(level: &[Cluster], groups: &[Vec<usize>], vertices: usize) -> Vec<bool> {
    let mut owner: Vec<u32> = vec![u32::MAX; vertices];
    let mut locked = vec![false; vertices];
    for (group_index, group) in groups.iter().enumerate() {
        for member in group {
            for vertex in &level[*member].indices {
                let slot = *vertex as usize;
                if slot >= vertices {
                    continue;
                }
                if owner[slot] == u32::MAX {
                    owner[slot] = group_index as u32;
                } else if owner[slot] != group_index as u32 {
                    locked[slot] = true;
                }
            }
        }
    }
    locked
}
