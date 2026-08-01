use super::*;

pub(super) struct IndexedMesh<'a> {
    positions: &'a [Vec3],
    normals: &'a [Vec3],
    texcoords: &'a [Vec2],
    indices: &'a [u32],
    tangents: Vec<[f32; 4]>,
    written: Vec<bool>,
}

impl bevy_mikktspace::Geometry for IndexedMesh<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let position = self.positions[self.indices[face * 3 + vert] as usize];
        [position.x, position.y, position.z]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let normal = self.normals[self.indices[face * 3 + vert] as usize];
        [normal.x, normal.y, normal.z]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let texcoord = self.texcoords[self.indices[face * 3 + vert] as usize];
        [texcoord.x, 1.0 - texcoord.y]
    }

    fn set_tangent(
        &mut self,
        tangent_space: Option<bevy_mikktspace::TangentSpace>,
        face: usize,
        vert: usize,
    ) {
        let Some(ts) = tangent_space else {
            return;
        };
        let vertex_index = self.indices[face * 3 + vert] as usize;
        if self.written[vertex_index] {
            return;
        }
        let tangent = ts.tangent();
        let sign = if ts.is_orientation_preserving() {
            1.0
        } else {
            -1.0
        };
        self.tangents[vertex_index] = [tangent[0], tangent[1], tangent[2], sign];
        self.written[vertex_index] = true;
    }
}

pub(super) fn generate_mikktspace_tangents(
    positions: &[Vec3],
    normals: &[Vec3],
    texcoords: &[Vec2],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let mut mesh = IndexedMesh {
        positions,
        normals,
        texcoords,
        indices,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        written: vec![false; positions.len()],
    };

    if let Err(error) = bevy_mikktspace::generate_tangents(&mut mesh) {
        tracing::warn!(
            "mikktspace tangent generation failed ({:?}); falling back to default tangents",
            error
        );
        return vec![[1.0, 0.0, 0.0, 1.0]; positions.len()];
    }

    mesh.tangents
}
