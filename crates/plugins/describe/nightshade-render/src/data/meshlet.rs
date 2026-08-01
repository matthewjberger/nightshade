#[derive(Clone, Debug, Default)]
pub struct MeshletMesh {
    pub meshlets: Vec<Meshlet>,
    pub vertices: Vec<u32>,
    pub triangles: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Meshlet {
    pub bounds: [f32; 4],
    pub cone: [f32; 4],
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub group: [f32; 4],
    pub group_error: f32,
    pub parent: [f32; 4],
    pub parent_error: f32,
}

pub const ERROR_UNBOUNDED: f32 = 1.0e30;

pub const MESHLET_MAX_VERTICES: usize = 64;
pub const MESHLET_MAX_TRIANGLES: usize = 124;

pub const MESHLET_GROUP_SIZE: usize = 4;

pub const MESHLET_MAX_LEVELS: usize = 10;
