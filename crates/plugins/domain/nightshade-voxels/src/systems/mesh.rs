use crate::commands::drop_the_mesh;
use crate::contract::{
    AIR, Blocks, Cell, ChunkKey, ChunkLayer, ColumnKey, Face, Patch, Shape, SlabKey,
};
use crate::data;
use crate::queries::{
    block_index, chunk_openings, column_of, layer_material, slab_chunks, slab_mesh_name, slab_of,
    slab_ready, surface_index, view_rank, with_bounds,
};
use crate::resources::{
    Chunk, ChunkStage, Ground, Meshes, Neighborhood, SlabWork, VoxelSettings, VoxelViewer,
    VoxelWorld,
};
use nalgebra_glm::{IVec3, Vec2, Vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::{Commands, Res, ResMut};
use nightshade_render::commands::mesh_cache::{
    mesh_cache_insert, mesh_cache_protect, mesh_cache_release,
};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_render::queries::material_registry::material_registry_resolve;
use nightshade_scene::contract::primitives::{Name, Visibility};
use nightshade_scene::contract::{GlobalTransform, LocalTransform};
use nightshade_worker::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

impl Job for Ground {
    type Key = SlabKey;
    type Done = SlabWork;
    type Tools = ();

    fn key(&self) -> Self::Key {
        self.key
    }

    fn run(self, _tools: &mut Self::Tools) -> Self::Done {
        let key = self.key;
        let openings = read_the_openings(&self);
        let mut around = spread_the_ground(&self);
        around.sky = measure_the_sky(&around);
        SlabWork {
            key,
            meshes: build_the_slab(&around),
            openings,
        }
    }
}

fn read_the_openings(ground: &Ground) -> Vec<(ChunkKey, u64)> {
    let reach = data::SLAB_COLUMNS;
    let base = [
        ground.key[0] * data::SLAB - 1,
        ground.key[2] * data::SLAB - 1,
    ];
    let mut found = Vec::with_capacity((data::SLAB * data::SLAB * data::SLAB_LEVELS) as usize);
    let deep = ground.key[1] * data::SLAB_LEVELS;
    for step_y in 0..data::SLAB_LEVELS {
        for step_z in 1..=data::SLAB {
            for step_x in 1..=data::SLAB {
                let cell = (((step_y + 1) * reach + step_z) * reach + step_x) as usize;
                let Some(chunk) = ground.chunks[cell].as_ref() else {
                    continue;
                };
                let key = [base[0] + step_x, deep + step_y, base[1] + step_z];
                found.push((key, chunk_openings(chunk, ground.blocks.as_ref())));
            }
        }
    }
    found
}

pub fn rebuild_dirty(
    mut world: ResMut<VoxelWorld>,
    mut masons: ResMut<Workers<Ground>>,
    viewer: Res<VoxelViewer>,
    settings: Res<VoxelSettings>,
) {
    if world.dirty.is_empty() || workers_in_flight(&masons) >= data::SLABS_IN_FLIGHT {
        return;
    }
    let eye = viewer.eye / settings.block_size;
    let standing = slab_of(
        eye.x.floor() as i32,
        eye.y.floor() as i32,
        eye.z.floor() as i32,
    );
    let reach = |key: &SlabKey| {
        let apart = key[1] - standing[1];
        view_rank(
            key[0] - standing[0],
            key[2] - standing[2],
            viewer.forward,
            data::VIEW_BEHIND,
        ) + apart * apart * data::LEVEL_RANK
    };
    let mut nearest: Vec<(i32, SlabKey)> = Vec::with_capacity(data::MESH_BATCH + 1);
    for key in world.dirty.iter() {
        let rank = reach(key);
        let spot = nearest.partition_point(|(held, _)| *held <= rank);
        if spot >= data::MESH_BATCH {
            continue;
        }
        nearest.insert(spot, (rank, *key));
        if nearest.len() > data::MESH_BATCH {
            nearest.pop();
        }
    }
    for (rank, key) in nearest {
        if workers_in_flight(&masons) >= data::SLABS_IN_FLIGHT {
            break;
        }
        if workers_holds(&masons, key) {
            continue;
        }
        if !viewer.shown.contains(&key) {
            for holder in slab_chunks(key) {
                if let Some(chunk) = world.chunks.get_mut(&holder) {
                    chunk.stage = ChunkStage::Decorated;
                }
            }
            world.dirty.remove(&key);
            continue;
        }
        if !slab_ready(&world, key) {
            continue;
        }
        world.dirty.remove(&key);
        if !slab_worth_meshing(&world, key) {
            continue;
        }
        let ground = gather_the_ground(&world, key, &settings);
        workers_submit(&mut masons, ground, rank);
    }
}

pub fn place_the_meshes(
    mut world: ResMut<VoxelWorld>,
    mut masons: ResMut<Workers<Ground>>,
    mut meshes: ResMut<Meshes>,
    mut mesh_cache: ResMut<MeshCache>,
    viewer: Res<VoxelViewer>,
    settings: Res<VoxelSettings>,
    materials: Res<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut commands: Commands,
) {
    let finished = workers_take(&mut masons, data::PLACE_LIMIT);
    if finished.is_empty() {
        return;
    }
    let mut wanted: HashMap<(SlabKey, ChunkLayer), bool> = HashMap::new();
    let mut opened = false;
    for work in finished {
        for (layer, mesh) in work.meshes {
            if mesh.vertices.is_empty() {
                wanted.insert((work.key, layer), false);
                continue;
            }
            let name = slab_mesh_name(work.key, layer);
            mesh_cache_insert(&mut mesh_cache, name.clone(), mesh);
            mesh_cache_protect(&mut mesh_cache, name);
            wanted.insert((work.key, layer), true);
        }
        for (holder, openings) in work.openings {
            let Some(chunk) = world.chunks.get_mut(&holder) else {
                continue;
            };
            chunk.stage = ChunkStage::Meshed;
            if chunk.openings != openings {
                chunk.openings = openings;
                opened = true;
            }
        }
    }
    if opened {
        world.sight += 1;
    }
    for ((key, layer), visible) in wanted {
        let name = slab_mesh_name(key, layer);
        if let Some(entity) = meshes.entities.get(&(key, layer)).copied() {
            if !visible {
                meshes.entities.remove(&(key, layer));
                mesh_cache_release(&mut mesh_cache, &name);
                drop_the_mesh(&mut commands, entity);
            }
            continue;
        }
        if !visible {
            continue;
        }
        let casts = matches!(layer, ChunkLayer::Solid | ChunkLayer::Cutout)
            && meshes.casting.contains(&key);
        let shown = viewer.shown.contains(&key);
        let material =
            material_registry_resolve(&materials, layer_material(&settings, &world.blocks, layer))
                .unwrap_or_default();
        let entity = spawn_slab_entity(
            &mut commands,
            material,
            settings.block_size,
            key,
            &name,
            casts,
            shown,
        );
        meshes.entities.insert((key, layer), entity);
    }
}

fn slab_worth_meshing(world: &VoxelWorld, key: SlabKey) -> bool {
    if !slab_chunks(key).any(|holder| world.chunks.contains_key(&holder)) {
        return false;
    }
    let base = [key[0] * data::SLAB - 1, key[2] * data::SLAB - 1];
    let deep = key[1] * data::SLAB_LEVELS;
    for level in deep - 1..=deep + data::SLAB_LEVELS {
        for step_z in 0..data::SLAB_COLUMNS {
            for step_x in 0..data::SLAB_COLUMNS {
                let holder = [base[0] + step_x, level, base[1] + step_z];
                match world.chunks.get(&holder) {
                    Some(chunk) => {
                        if !chunk.solid {
                            return true;
                        }
                    }
                    None => return true,
                }
            }
        }
    }
    false
}

fn gather_the_ground(world: &VoxelWorld, key: SlabKey, settings: &VoxelSettings) -> Ground {
    let base = [key[0] * data::SLAB - 1, key[2] * data::SLAB - 1];
    let reach = data::SLAB_COLUMNS;
    let deep = key[1] * data::SLAB_LEVELS;
    let mut chunks = Vec::with_capacity((reach * reach * (data::SLAB_LEVELS + 2)) as usize);
    for level in deep - 1..=deep + data::SLAB_LEVELS {
        for step_z in 0..reach {
            for step_x in 0..reach {
                let holder = [base[0] + step_x, level, base[1] + step_z];
                chunks.push(world.chunks.get(&holder).cloned());
            }
        }
    }
    let mut heights = Vec::with_capacity((reach * reach) as usize);
    for step_z in 0..reach {
        for step_x in 0..reach {
            let column = [base[0] + step_x, base[1] + step_z];
            heights.push(world.surface.get(&column).cloned());
        }
    }
    Ground {
        key,
        chunks,
        heights,
        blocks: world.blocks.clone(),
        floor_block: settings.floor_block,
    }
}

fn spread_the_ground(ground: &Ground) -> Neighborhood {
    let wide = data::SLAB_WIDE;
    let key = ground.key;
    let base = [key[0] * data::SLAB - 1, key[2] * data::SLAB - 1];
    let reach = data::SLAB_COLUMNS;
    let origin = IVec3::new(
        key[0] * data::SLAB_SPAN - 1,
        key[1] * data::SLAB_HIGH - 1,
        key[2] * data::SLAB_SPAN - 1,
    );
    let mut kinds = vec![AIR; data::SLAB_CELLS];
    for offset_y in 0..data::SLAB_TALL {
        let level = origin.y + offset_y;
        if level < data::FLOOR_Y {
            for cell in kinds
                .iter_mut()
                .skip((offset_y * wide * wide) as usize)
                .take((wide * wide) as usize)
            {
                *cell = ground.floor_block;
            }
            continue;
        }
        if level >= data::WORLD_Y {
            continue;
        }
        let layer = level.div_euclid(data::CHUNK) - (key[1] * data::SLAB_LEVELS - 1);
        if !(0..data::SLAB_LEVELS + 2).contains(&layer) {
            continue;
        }
        for offset_z in 0..wide {
            let mut cached_column: Option<ColumnKey> = None;
            let mut cached_chunk: Option<&Chunk> = None;
            for offset_x in 0..wide {
                let position = origin + IVec3::new(offset_x, offset_y, offset_z);
                let column = column_of(position.x, position.z);
                if cached_column != Some(column) {
                    cached_column = Some(column);
                    let step_x = column[0] - base[0];
                    let step_z = column[1] - base[1];
                    cached_chunk = if (0..reach).contains(&step_x) && (0..reach).contains(&step_z) {
                        ground.chunks[((layer * reach + step_z) * reach + step_x) as usize].as_ref()
                    } else {
                        None
                    };
                }
                let Some(chunk) = cached_chunk else {
                    continue;
                };
                let cell = ((offset_y * wide + offset_z) * wide + offset_x) as usize;
                kinds[cell] = chunk.blocks[block_index(position)];
            }
        }
    }
    let mut tops = vec![data::WORLD_Y; (wide * wide) as usize];
    for offset_z in 0..wide {
        for offset_x in 0..wide {
            let (x, z) = (origin.x + offset_x, origin.z + offset_z);
            let column = column_of(x, z);
            let step_x = column[0] - base[0];
            let step_z = column[1] - base[1];
            let found = if (0..reach).contains(&step_x) && (0..reach).contains(&step_z) {
                ground.heights[(step_z * reach + step_x) as usize]
                    .as_ref()
                    .map(|heights| heights[surface_index(x, z)] as i32)
                    .unwrap_or(data::SURFACE_NONE as i32)
            } else {
                data::SURFACE_NONE as i32
            };
            tops[(offset_z * wide + offset_x) as usize] = found;
        }
    }
    Neighborhood {
        origin,
        table: ground.blocks.clone(),
        blocks: kinds,
        tops,
        sky: Vec::new(),
    }
}

fn measure_the_sky(around: &Neighborhood) -> Vec<u8> {
    let table = around.table.as_ref();
    let wide = data::SLAB_WIDE;
    let origin = around.origin;
    let kinds = &around.blocks;
    let tops = &around.tops;
    let mut sky = vec![0u8; data::SLAB_CELLS];
    let mut floors = vec![data::SLAB_TALL; (wide * wide) as usize];
    for offset_z in 0..wide {
        for offset_x in 0..wide {
            let spot = (offset_z * wide + offset_x) as usize;
            if origin.y + data::SLAB_TALL - 1 <= tops[spot] {
                continue;
            }
            for offset_y in (0..data::SLAB_TALL).rev() {
                let cell = ((offset_y * wide + offset_z) * wide + offset_x) as usize;
                if table.occludes(kinds[cell]) {
                    break;
                }
                sky[cell] = data::SKY_LEVELS;
                floors[spot] = offset_y;
            }
        }
    }
    let mut queue = VecDeque::new();
    for offset_z in 0..wide {
        for offset_x in 0..wide {
            let floor = floors[(offset_z * wide + offset_x) as usize];
            if floor >= data::SLAB_TALL {
                continue;
            }
            let mut reach = floor;
            for step in [[-1, 0], [1, 0], [0, -1], [0, 1]] {
                let near_x = offset_x + step[0];
                let near_z = offset_z + step[1];
                if !(0..wide).contains(&near_x) || !(0..wide).contains(&near_z) {
                    continue;
                }
                reach = reach.max(floors[(near_z * wide + near_x) as usize]);
            }
            for offset_y in floor..reach {
                queue.push_back([offset_x, offset_y, offset_z]);
            }
        }
    }
    while let Some(spot) = queue.pop_front() {
        let cell = ((spot[1] * wide + spot[2]) * wide + spot[0]) as usize;
        let level = sky[cell];
        if level <= 1 {
            continue;
        }
        for face in Face::ALL {
            let step = face.normal();
            let next = [spot[0] + step[0], spot[1] + step[1], spot[2] + step[2]];
            if !(0..wide).contains(&next[0])
                || !(0..data::SLAB_TALL).contains(&next[1])
                || !(0..wide).contains(&next[2])
            {
                continue;
            }
            let cell = ((next[1] * wide + next[2]) * wide + next[0]) as usize;
            if table.occludes(kinds[cell]) || sky[cell] >= level - 1 {
                continue;
            }
            sky[cell] = level - 1;
            queue.push_back(next);
        }
    }
    sky
}

fn spawn_slab_entity(
    commands: &mut Commands,
    material: MaterialId,
    block_size: f32,
    key: SlabKey,
    mesh_name: &str,
    casts: bool,
    shown: bool,
) -> Entity {
    let mut transform = LocalTransform::from_translation(
        Vec3::new(
            (key[0] * data::SLAB_SPAN) as f32,
            (key[1] * data::SLAB_HIGH) as f32,
            (key[2] * data::SLAB_SPAN) as f32,
        ) * block_size,
    );
    transform.scale = Vec3::repeat(block_size);
    let extent = Vec3::new(
        data::SLAB_SPAN as f32,
        data::SLAB_HIGH as f32,
        data::SLAB_SPAN as f32,
    ) * block_size;
    let entity = commands.spawn((
        Name(mesh_name.to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(mesh_name),
        MaterialRef::new(material),
        Visibility { visible: shown },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(Vec3::zeros(), extent),
            extent.norm() * 0.5,
        ),
    ));
    if casts {
        commands.set(entity, CastsShadow);
    }
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

#[derive(Clone, Copy)]
struct FaceArt {
    cell: Cell,
    low: Vec2,
    high: Vec2,
    sky: [u8; 4],
}

impl FaceArt {
    fn patched(cell: Cell, patch: Patch, sky: u8) -> Self {
        Self {
            cell,
            low: Vec2::new(patch.low[0], patch.low[1]),
            high: Vec2::new(patch.high[0], patch.high[1]),
            sky: [sky; 4],
        }
    }
}

fn sky_share(level: u8) -> f32 {
    (level as f32 / data::SKY_LEVELS as f32).max(data::CAVE_FLOOR)
}

struct Builder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    table: Arc<Blocks>,
}

impl Builder {
    fn quad(&mut self, corners: [Vec3; 4], normal: Vec3, art: FaceArt, light: [f32; 4]) {
        let uvs = [
            [art.low.x, art.high.y],
            [art.high.x, art.high.y],
            [art.high.x, art.low.y],
            [art.low.x, art.low.y],
        ];
        let cell = self.table.cell_index(art.cell);
        let base = self.vertices.len() as u32;
        for (index, corner) in corners.iter().enumerate() {
            let level = light[index];
            self.vertices.push(Vertex::with_all(
                *corner,
                normal,
                uvs[index],
                [cell, sky_share(art.sky[index])],
                [1.0, 0.0, 0.0, 1.0],
                [level, level, level, 1.0],
            ));
        }
        let flip = light[0] + light[2] < light[1] + light[3];
        let order: [u32; 6] = if flip {
            [1, 2, 3, 1, 3, 0]
        } else {
            [0, 1, 2, 0, 2, 3]
        };
        self.indices
            .extend(order.iter().map(|offset| base + offset));
    }
}

fn face_corners(face: Face, top_height: f32) -> [Vec3; 4] {
    let h = top_height;
    match face {
        Face::South => [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, h, 1.0),
            Vec3::new(0.0, h, 1.0),
        ],
        Face::North => [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, h, 0.0),
            Vec3::new(1.0, h, 0.0),
        ],
        Face::East => [
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, h, 0.0),
            Vec3::new(1.0, h, 1.0),
        ],
        Face::West => [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, h, 1.0),
            Vec3::new(0.0, h, 0.0),
        ],
        Face::Top => [
            Vec3::new(0.0, h, 1.0),
            Vec3::new(1.0, h, 1.0),
            Vec3::new(1.0, h, 0.0),
            Vec3::new(0.0, h, 0.0),
        ],
        Face::Bottom => [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
    }
}

fn face_cell(table: &Blocks, kind: u8, face: Face) -> Cell {
    let info = table.look(kind);
    match face {
        Face::Top => info.top,
        Face::Bottom => info.bottom,
        _ => info.side,
    }
}

fn occlusion(around: &Neighborhood, position: IVec3, face: Face, corner: Vec3) -> u8 {
    let normal = face.normal();
    let normal_axis = (0..3).find(|&axis| normal[axis] != 0).unwrap_or(1);
    let outside = position + IVec3::new(normal[0], normal[1], normal[2]);
    let mut sides = [IVec3::zeros(); 2];
    let mut count = 0;
    for axis in 0..3 {
        if axis == normal_axis {
            continue;
        }
        let sign = if corner[axis] > 0.5 { 1 } else { -1 };
        let mut step = IVec3::zeros();
        step[axis] = sign;
        sides[count] = step;
        count += 1;
    }
    let table = around.table.as_ref();
    let first = table.occludes(around.at(outside + sides[0]));
    let second = table.occludes(around.at(outside + sides[1]));
    let diagonal = table.occludes(around.at(outside + sides[0] + sides[1]));
    if first && second {
        return 0;
    }
    3 - (first as u8 + second as u8 + diagonal as u8)
}

fn corner_sky(around: &Neighborhood, position: IVec3, face: Face, corner: Vec3) -> u8 {
    let normal = face.normal();
    let normal_axis = (0..3).find(|&axis| normal[axis] != 0).unwrap_or(1);
    let outside = position + IVec3::new(normal[0], normal[1], normal[2]);
    let mut sides = [IVec3::zeros(); 2];
    let mut count = 0;
    for axis in 0..3 {
        if axis == normal_axis {
            continue;
        }
        let sign = if corner[axis] > 0.5 { 1 } else { -1 };
        let mut step = IVec3::zeros();
        step[axis] = sign;
        sides[count] = step;
        count += 1;
    }
    let table = around.table.as_ref();
    let first = table.occludes(around.at(outside + sides[0]));
    let second = table.occludes(around.at(outside + sides[1]));
    let mut total = around.sky_at(outside) as u32;
    let mut taken = 1;
    if !first {
        total += around.sky_at(outside + sides[0]) as u32;
        taken += 1;
    }
    if !second {
        total += around.sky_at(outside + sides[1]) as u32;
        taken += 1;
    }
    let diagonal = outside + sides[0] + sides[1];
    if !(first && second) && !table.occludes(around.at(diagonal)) {
        total += around.sky_at(diagonal) as u32;
        taken += 1;
    }
    (total / taken) as u8
}

fn face_visible(around: &Neighborhood, kind: u8, neighbor_position: IVec3) -> bool {
    let table = around.table.as_ref();
    let neighbor = around.at(neighbor_position);
    let info = table.look(kind);
    match info.shape {
        Shape::Liquid => match (info.fill, table.fill(neighbor)) {
            (Some(mine), Some(other)) if mine.liquid == other.liquid => neighbor != kind,
            (Some(mine), Some(other)) => mine.liquid < other.liquid,
            _ => !table.occludes(neighbor),
        },
        _ if info.cutout || info.transparent => neighbor != kind && !table.occludes(neighbor),
        _ => !table.occludes(neighbor),
    }
}

fn build_the_slab(around: &Neighborhood) -> Vec<(ChunkLayer, Mesh)> {
    let table = around.table.as_ref();
    let origin = around.origin + IVec3::new(1, 1, 1);
    let layers: Vec<ChunkLayer> = table.layers().collect();
    let mut builders: Vec<Builder> = layers
        .iter()
        .map(|_| Builder {
            vertices: Vec::new(),
            indices: Vec::new(),
            table: around.table.clone(),
        })
        .collect();
    for local_y in 0..data::SLAB_HIGH {
        for local_z in 0..data::SLAB_SPAN {
            for local_x in 0..data::SLAB_SPAN {
                let position = origin + IVec3::new(local_x, local_y, local_z);
                let kind = around.at(position);
                if kind == AIR {
                    continue;
                }
                let info = table.look(kind);
                let builder = &mut builders[layer_slot(table, kind)];
                let local = Vec3::new(local_x as f32, local_y as f32, local_z as f32);
                let sky = around.sky_at(position);
                match info.shape {
                    Shape::Cross => add_cross(builder, table, kind, local, sky),
                    Shape::Parts => add_parts(builder, around, kind, position, local, sky),
                    Shape::Cube | Shape::Liquid => {}
                }
            }
        }
    }
    for face in Face::ALL {
        add_merged_faces(&mut builders, around, origin, face);
    }
    layers
        .into_iter()
        .zip(builders)
        .map(|(layer, builder)| (layer, finish(builder)))
        .collect()
}

fn layer_slot(table: &Blocks, kind: u8) -> usize {
    let info = table.look(kind);
    match (info.shape, info.fill) {
        (Shape::Liquid, Some(fill)) => 3 + fill.liquid as usize,
        _ if info.cutout => 1,
        _ if info.transparent => 2,
        _ => 0,
    }
}

fn finish(builder: Builder) -> Mesh {
    with_bounds(Mesh::new(builder.vertices, builder.indices))
}

#[derive(Clone, Copy, PartialEq)]
struct FaceKey {
    kind: u8,
    light: [u8; 4],
    sky: [u8; 4],
    full: bool,
}

fn face_axes(face: Face) -> (usize, usize, usize) {
    match face {
        Face::North | Face::South => (2, 0, 1),
        Face::East | Face::West => (0, 2, 1),
        Face::Top | Face::Bottom => (1, 0, 2),
    }
}

fn liquid_top(table: &Blocks, kind: u8) -> f32 {
    match table.level(kind) {
        5 => 0.9,
        4 => 0.74,
        3 => 0.58,
        2 => 0.42,
        1 => 0.26,
        _ => 1.0,
    }
}

fn face_key(around: &Neighborhood, position: IVec3, face: Face) -> Option<FaceKey> {
    let table = around.table.as_ref();
    let kind = around.at(position);
    if kind == AIR {
        return None;
    }
    let info = table.look(kind);
    let liquid = info.shape == Shape::Liquid;
    if !liquid && info.shape != Shape::Cube {
        return None;
    }
    if liquid && face == Face::Bottom {
        return None;
    }
    let normal = face.normal();
    let neighbor = position + IVec3::new(normal[0], normal[1], normal[2]);
    if !face_visible(around, kind, neighbor) {
        return None;
    }
    let above = table.fill(around.at(position + IVec3::new(0, 1, 0)));
    let full = !liquid
        || above.is_some_and(|other| Some(other.liquid) == info.fill.map(|fill| fill.liquid));
    let corners = face_corners(face, 1.0);
    let light = if liquid {
        [3; 4]
    } else {
        corners.map(|corner| occlusion(around, position, face, corner))
    };
    let sky = corners.map(|corner| corner_sky(around, position, face, corner));
    Some(FaceKey {
        kind,
        light,
        sky,
        full,
    })
}

fn axis_extent(axis: usize) -> i32 {
    if axis == 1 {
        data::SLAB_HIGH
    } else {
        data::SLAB_SPAN
    }
}

fn add_merged_faces(builders: &mut [Builder], around: &Neighborhood, origin: IVec3, face: Face) {
    let table = around.table.as_ref();
    let (plane_axis, axis_a, axis_b) = face_axes(face);
    let slices = axis_extent(plane_axis);
    let across_span = axis_extent(axis_a);
    let down_span = axis_extent(axis_b);
    let mut mask = vec![None; (across_span * down_span) as usize];
    for slice in 0..slices {
        for down in 0..down_span {
            for across in 0..across_span {
                let mut local = IVec3::zeros();
                local[plane_axis] = slice;
                local[axis_a] = across;
                local[axis_b] = down;
                mask[(down * across_span + across) as usize] =
                    face_key(around, origin + local, face);
            }
        }
        for down in 0..down_span {
            let mut across = 0;
            while across < across_span {
                let Some(key) = mask[(down * across_span + across) as usize] else {
                    across += 1;
                    continue;
                };
                let mut width = 1;
                while across + width < across_span
                    && mask[(down * across_span + across + width) as usize] == Some(key)
                {
                    width += 1;
                }
                let mut height = 1;
                while down + height < down_span {
                    let row = down + height;
                    let filled = (0..width).all(|step| {
                        mask[(row * across_span + across + step) as usize] == Some(key)
                    });
                    if !filled {
                        break;
                    }
                    height += 1;
                }
                for row in 0..height {
                    for step in 0..width {
                        mask[((down + row) * across_span + across + step) as usize] = None;
                    }
                }
                add_face(
                    builders, table, face, slice, across, down, width, height, key,
                );
                across += width;
            }
        }
    }
}

fn add_face(
    builders: &mut [Builder],
    table: &Blocks,
    face: Face,
    slice: i32,
    across: i32,
    down: i32,
    width: i32,
    height: i32,
    key: FaceKey,
) {
    let top = if key.full {
        1.0
    } else {
        liquid_top(table, key.kind)
    };
    let plane = match face {
        Face::South | Face::East => (slice + 1) as f32,
        Face::North | Face::West | Face::Bottom => slice as f32,
        Face::Top => slice as f32 + top,
    };
    let (low_a, high_a) = (across as f32, (across + width) as f32);
    let low_b = down as f32;
    let high_b = match face {
        Face::Top | Face::Bottom => (down + height) as f32,
        _ => low_b + (height - 1) as f32 + top,
    };
    let corners = match face {
        Face::South => [
            Vec3::new(low_a, low_b, plane),
            Vec3::new(high_a, low_b, plane),
            Vec3::new(high_a, high_b, plane),
            Vec3::new(low_a, high_b, plane),
        ],
        Face::North => [
            Vec3::new(high_a, low_b, plane),
            Vec3::new(low_a, low_b, plane),
            Vec3::new(low_a, high_b, plane),
            Vec3::new(high_a, high_b, plane),
        ],
        Face::East => [
            Vec3::new(plane, low_b, high_a),
            Vec3::new(plane, low_b, low_a),
            Vec3::new(plane, high_b, low_a),
            Vec3::new(plane, high_b, high_a),
        ],
        Face::West => [
            Vec3::new(plane, low_b, low_a),
            Vec3::new(plane, low_b, high_a),
            Vec3::new(plane, high_b, high_a),
            Vec3::new(plane, high_b, low_a),
        ],
        Face::Top => [
            Vec3::new(low_a, plane, high_b),
            Vec3::new(high_a, plane, high_b),
            Vec3::new(high_a, plane, low_b),
            Vec3::new(low_a, plane, low_b),
        ],
        Face::Bottom => [
            Vec3::new(low_a, plane, low_b),
            Vec3::new(high_a, plane, low_b),
            Vec3::new(high_a, plane, high_b),
            Vec3::new(low_a, plane, high_b),
        ],
    };
    let normal = face.normal();
    let normal = Vec3::new(normal[0] as f32, normal[1] as f32, normal[2] as f32);
    let light = key.light.map(|level| data::AMBIENT_LEVELS[level as usize]);
    builders[layer_slot(table, key.kind)].quad(
        corners,
        normal,
        FaceArt {
            cell: face_cell(table, key.kind, face),
            low: Vec2::zeros(),
            high: Vec2::new(width as f32, height as f32),
            sky: key.sky,
        },
        light,
    );
}

fn add_cross(builder: &mut Builder, table: &Blocks, kind: u8, local: Vec3, sky: u8) {
    let art = FaceArt::patched(table.look(kind).side, Patch::FULL, sky);
    let (a, b) = (0.15, 0.85);
    let planes = [
        [
            Vec3::new(a, 0.0, a),
            Vec3::new(b, 0.0, b),
            Vec3::new(b, 1.0, b),
            Vec3::new(a, 1.0, a),
        ],
        [
            Vec3::new(a, 0.0, b),
            Vec3::new(b, 0.0, a),
            Vec3::new(b, 1.0, a),
            Vec3::new(a, 1.0, b),
        ],
    ];
    for plane in planes {
        let along = plane[1] - plane[0];
        let aside = Vec3::new(along.z, 0.0, -along.x).normalize() * data::CROSS_OFFSET;
        let front = plane.map(|corner| corner + local + aside);
        let back = [plane[1], plane[0], plane[3], plane[2]].map(|corner| corner + local - aside);
        builder.quad(front, Vec3::y(), art, [1.0; 4]);
        builder.quad(back, Vec3::y(), art, [1.0; 4]);
    }
}

fn add_parts(
    builder: &mut Builder,
    around: &Neighborhood,
    kind: u8,
    position: IVec3,
    local: Vec3,
    sky: u8,
) {
    let table = around.table.as_ref();
    let look = table.look(kind);
    let stacked = around.at(position - IVec3::new(0, 1, 0)) == kind;
    let cell = match look.stacked {
        Some(cell) if stacked => cell,
        _ => look.side,
    };
    for part in look.parts {
        let low = local + Vec3::from_column_slice(&part.low);
        let high = local + Vec3::from_column_slice(&part.high);
        let lift = Vec3::from_column_slice(&part.lift);
        for face in Face::ALL {
            let patch = match face {
                Face::Top => part.top,
                Face::Bottom => part.bottom,
                _ => part.side,
            };
            let normal = face.normal();
            let normal = Vec3::new(normal[0] as f32, normal[1] as f32, normal[2] as f32);
            let corners = box_face(face, low, high, lift);
            let art = FaceArt::patched(cell, patch, sky);
            builder.quad(corners, normal, art, [1.0; 4]);
        }
    }
}

fn box_face(face: Face, low: Vec3, high: Vec3, lift: Vec3) -> [Vec3; 4] {
    let (l, h) = (low, high);
    let corners = match face {
        Face::South => [
            Vec3::new(l.x, l.y, h.z),
            Vec3::new(h.x, l.y, h.z),
            Vec3::new(h.x, h.y, h.z),
            Vec3::new(l.x, h.y, h.z),
        ],
        Face::North => [
            Vec3::new(h.x, l.y, l.z),
            Vec3::new(l.x, l.y, l.z),
            Vec3::new(l.x, h.y, l.z),
            Vec3::new(h.x, h.y, l.z),
        ],
        Face::East => [
            Vec3::new(h.x, l.y, h.z),
            Vec3::new(h.x, l.y, l.z),
            Vec3::new(h.x, h.y, l.z),
            Vec3::new(h.x, h.y, h.z),
        ],
        Face::West => [
            Vec3::new(l.x, l.y, l.z),
            Vec3::new(l.x, l.y, h.z),
            Vec3::new(l.x, h.y, h.z),
            Vec3::new(l.x, h.y, l.z),
        ],
        Face::Top => [
            Vec3::new(l.x, h.y, h.z),
            Vec3::new(h.x, h.y, h.z),
            Vec3::new(h.x, h.y, l.z),
            Vec3::new(l.x, h.y, l.z),
        ],
        Face::Bottom => [
            Vec3::new(l.x, l.y, l.z),
            Vec3::new(h.x, l.y, l.z),
            Vec3::new(h.x, l.y, h.z),
            Vec3::new(l.x, l.y, h.z),
        ],
    };
    corners.map(|corner| {
        if corner.y == h.y {
            corner + lift
        } else {
            corner
        }
    })
}
