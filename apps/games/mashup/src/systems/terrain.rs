use crate::data;
use crate::systems::textures::{self, DIRT, GRASS_SIDE, GRASS_TOP, STONE};
use nightshade::prelude::Vec3;
use nightshade_render::data::geometry::{Mesh, Vertex};

const SIDES: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

struct Builder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Builder {
    fn quad(&mut self, corners: [Vec3; 4], normal: Vec3, cell: u32, shade: f32) {
        let (low, high) = textures::cell_bounds(cell);
        let uvs = [
            [low[0], high[1]],
            [high[0], high[1]],
            [high[0], low[1]],
            [low[0], low[1]],
        ];
        let base = self.vertices.len() as u32;
        for (index, corner) in corners.iter().enumerate() {
            self.vertices.push(Vertex::with_all(
                *corner,
                normal,
                uvs[index],
                [0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0],
                [shade, shade, shade, 1.0],
            ));
        }
        for offset in [0_u32, 1, 2, 0, 2, 3] {
            self.indices.push(base + offset);
        }
    }
}

fn cell_for(level: i32, top: i32) -> u32 {
    if level == top - 1 {
        GRASS_SIDE
    } else if level > 2 {
        DIRT
    } else {
        STONE
    }
}

pub fn build() -> Mesh {
    let mut builder = Builder {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    let span = data::QUARRY_SPAN;
    for column_x in -span..=span {
        for column_z in -span..=span {
            let top = data::column_height(column_x, column_z);
            let x = column_x as f32;
            let z = column_z as f32;
            let y = top as f32;
            builder.quad(
                [
                    Vec3::new(x, y, z + 1.0),
                    Vec3::new(x + 1.0, y, z + 1.0),
                    Vec3::new(x + 1.0, y, z),
                    Vec3::new(x, y, z),
                ],
                Vec3::y(),
                GRASS_TOP,
                1.0,
            );
            for (step_x, step_z) in SIDES {
                let neighbour =
                    if (column_x + step_x).abs() > span || (column_z + step_z).abs() > span {
                        0
                    } else {
                        data::column_height(column_x + step_x, column_z + step_z)
                    };
                if neighbour >= top {
                    continue;
                }
                let shade = if step_x == 0 { 0.78 } else { 0.62 };
                for level in neighbour..top {
                    let low = level as f32;
                    let high = low + 1.0;
                    let corners = match (step_x, step_z) {
                        (0, 1) => [
                            Vec3::new(x, low, z + 1.0),
                            Vec3::new(x + 1.0, low, z + 1.0),
                            Vec3::new(x + 1.0, high, z + 1.0),
                            Vec3::new(x, high, z + 1.0),
                        ],
                        (1, 0) => [
                            Vec3::new(x + 1.0, low, z + 1.0),
                            Vec3::new(x + 1.0, low, z),
                            Vec3::new(x + 1.0, high, z),
                            Vec3::new(x + 1.0, high, z + 1.0),
                        ],
                        (0, -1) => [
                            Vec3::new(x + 1.0, low, z),
                            Vec3::new(x, low, z),
                            Vec3::new(x, high, z),
                            Vec3::new(x + 1.0, high, z),
                        ],
                        _ => [
                            Vec3::new(x, low, z),
                            Vec3::new(x, low, z + 1.0),
                            Vec3::new(x, high, z + 1.0),
                            Vec3::new(x, high, z),
                        ],
                    };
                    let normal = Vec3::new(step_x as f32, 0.0, step_z as f32);
                    builder.quad(corners, normal, cell_for(level, top), shade);
                }
            }
        }
    }
    Mesh::new(builder.vertices, builder.indices)
}
