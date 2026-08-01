use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
use fast_surface_nets::{SurfaceNetsBuffer, surface_nets};
use nalgebra_glm::Vec3;

use crate::contract::field::{field_doubt, sdf_field};
use crate::contract::render_data::{SdfChunkKey, SdfCollisionMesh, SdfEditInstance};
use crate::data::collision::{CHUNK_CELLS, CHUNK_POINTS, CHUNK_SAMPLES};

const SAMPLED_HALF_DIAGONAL: f32 = 0.866_025_4;

type ChunkShape = ConstShape3u32<CHUNK_SAMPLES, CHUNK_SAMPLES, CHUNK_SAMPLES>;

pub fn chunk_corner(key: SdfChunkKey, cell: f32) -> Vec3 {
    let span = CHUNK_CELLS as f32 * cell;
    Vec3::new(
        key[0] as f32 * span,
        key[1] as f32 * span,
        key[2] as f32 * span,
    )
}

pub fn chunk_of(point: &Vec3, cell: f32) -> SdfChunkKey {
    let span = CHUNK_CELLS as f32 * cell;
    [
        (point.x / span).floor() as i32,
        (point.y / span).floor() as i32,
        (point.z / span).floor() as i32,
    ]
}

pub fn chunk_surface(edits: &[SdfEditInstance], key: SdfChunkKey, cell: f32) -> SdfCollisionMesh {
    let corner = chunk_corner(key, cell);
    let middle = corner + Vec3::repeat(CHUNK_CELLS as f32 * cell * 0.5);
    let reach = SAMPLED_HALF_DIAGONAL * CHUNK_SAMPLES as f32 * cell * field_doubt(edits) + cell;
    if sdf_field(edits, &middle, reach).abs() > reach {
        return SdfCollisionMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
    }
    let mut distances = vec![1.0_f32; CHUNK_POINTS];
    for (index, distance) in distances.iter_mut().enumerate() {
        let [x, y, z] = ChunkShape::delinearize(index as u32);
        let point = corner
            + Vec3::new(
                (x as f32 - 1.0) * cell,
                (y as f32 - 1.0) * cell,
                (z as f32 - 1.0) * cell,
            );
        *distance = sdf_field(edits, &point, cell);
    }
    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(
        &distances,
        &ChunkShape {},
        [0; 3],
        [CHUNK_SAMPLES - 1; 3],
        &mut buffer,
    );
    let vertices = buffer
        .positions
        .iter()
        .map(|position| {
            [
                corner.x + (position[0] - 1.0) * cell,
                corner.y + (position[1] - 1.0) * cell,
                corner.z + (position[2] - 1.0) * cell,
            ]
        })
        .collect();
    let triangles = buffer.indices.as_chunks::<3>().0.to_vec();
    SdfCollisionMesh {
        vertices,
        triangles,
    }
}
