use crate::components::{Marker, Statue};
use crate::data::{Bump, Chisel, SculptJob, Stage};
use crate::queries::{box_mesh, floor_mesh, stand_place, statue_name};
use crate::resources::Gallery;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_render::commands::mesh_cache::{
    mesh_cache_insert, mesh_cache_protect, mesh_cache_release,
};
use nightshade_render::commands::mesh_state::{
    mesh_state_mark_entity_added, mesh_state_mark_entity_removed,
};
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};
use nightshade_worker::{Workers, workers_cancel};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn keep_the_shapes(mesh_cache: &mut MeshCache) {
    mesh_cache_insert(mesh_cache, theme::FLOOR_MESH.to_string(), floor_mesh());
    mesh_cache_protect(mesh_cache, theme::FLOOR_MESH.to_string());
    mesh_cache_insert(
        mesh_cache,
        theme::STAND_MESH.to_string(),
        box_mesh(theme::STAND_WIDE, theme::STAND_TALL),
    );
    mesh_cache_protect(mesh_cache, theme::STAND_MESH.to_string());
    mesh_cache_insert(
        mesh_cache,
        theme::MARKER_MESH.to_string(),
        box_mesh(theme::MARKER_SIZE, theme::MARKER_SIZE),
    );
    mesh_cache_protect(mesh_cache, theme::MARKER_MESH.to_string());
}

pub fn spawn_the_floor(commands: &mut Commands) -> Entity {
    let reach = theme::ACROSS as f32 * theme::STAND_SPAN * 2.0;
    let extent = Vec3::new(reach, 1.0, reach);
    stand_it_up(
        commands,
        "Floor",
        Vec3::new(0.0, -theme::STAND_TALL, 0.0),
        theme::FLOOR_MESH,
        theme::FLOOR_MATERIAL,
        extent,
    )
}

pub fn spawn_a_stand(commands: &mut Commands, slot: usize) -> Entity {
    let extent = Vec3::new(theme::STAND_WIDE, theme::STAND_TALL, theme::STAND_WIDE);
    let entity = stand_it_up(
        commands,
        "Stand",
        stand_place(slot),
        theme::STAND_MESH,
        theme::STAND_MATERIAL,
        extent,
    );
    commands.set(entity, CastsShadow);
    entity
}

pub fn spawn_a_marker(commands: &mut Commands, slot: usize) -> Entity {
    let mut place = stand_place(slot);
    place.y = theme::MARKER_RIDE;
    let extent = Vec3::new(theme::MARKER_SIZE, theme::MARKER_SIZE, theme::MARKER_SIZE);
    let entity = stand_it_up(
        commands,
        "Marker",
        place,
        theme::MARKER_MESH,
        theme::MARKER_MATERIAL,
        extent,
    );
    commands.set(entity, Marker { slot });
    entity
}

pub fn stand_a_statue(
    mesh_cache: &mut MeshCache,
    commands: &mut Commands,
    slot: usize,
    mesh: Mesh,
) -> Entity {
    let name = statue_name(slot);
    mesh_cache_insert(mesh_cache, name.clone(), mesh);
    mesh_cache_protect(mesh_cache, name.clone());
    let mut place = stand_place(slot);
    place.y = theme::STATUE_RIDE;
    let reach = theme::STATUE_SIZE * 2.0;
    let extent = Vec3::new(reach, reach, reach);
    let entity = stand_it_up(
        commands,
        "Statue",
        place,
        &name,
        theme::STATUE_MATERIAL,
        extent,
    );
    commands.set(entity, Statue { slot });
    commands.set(entity, CastsShadow);
    entity
}

pub fn clear_the_gallery(
    gallery: &mut Gallery,
    mesh_cache: &mut MeshCache,
    commands: &mut Commands,
    standing: Vec<(Entity, usize)>,
) {
    for (entity, slot) in standing {
        mesh_cache_release(mesh_cache, &statue_name(slot));
        commands.queue(move |_, resources| {
            mesh_state_mark_entity_removed(resources.res_mut::<MeshRenderState>(), entity);
        });
        commands.despawn(entity);
    }
    for stage in gallery.stage.iter_mut() {
        *stage = Stage::Empty;
    }
    gallery.era += 1;
}

pub fn throw_the_work_away(gallery: &mut Gallery, crew: &mut Workers<SculptJob>) {
    for slot in 0..theme::STANDS {
        if workers_cancel(crew, slot) {
            gallery.stage[slot] = Stage::Empty;
            gallery.thrown += 1;
        }
    }
}

pub fn flip_the_mode(gallery: &mut Gallery) {
    gallery.on_the_frame = !gallery.on_the_frame;
}

pub fn step_the_hands(gallery: &mut Gallery) {
    if gallery.ladder.is_empty() {
        return;
    }
    let step = gallery
        .ladder
        .iter()
        .position(|hands| *hands == gallery.hands)
        .unwrap_or(0);
    gallery.hands = gallery.ladder[(step + 1) % gallery.ladder.len()];
}

pub fn halt_the_crew(gallery: &mut Gallery) {
    gallery.paused = !gallery.paused;
}

fn stand_it_up(
    commands: &mut Commands,
    name: &str,
    place: Vec3,
    mesh: &str,
    material: &str,
    extent: Vec3,
) -> Entity {
    let transform = LocalTransform::from_translation(place);
    let entity = commands.spawn((
        Name(name.to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(mesh),
        MaterialRef::default(),
        Visibility { visible: true },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(Vec3::zeros(), extent),
            extent.norm(),
        ),
    ));
    set_mesh_material(commands, entity, material);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

pub fn sculpt(seed: u64, tint: usize, chisel: &mut Chisel) -> Mesh {
    let mut bumps = std::mem::take(&mut chisel.bumps);
    let mut reach = std::mem::take(&mut chisel.reach);
    let mut place = std::mem::take(&mut chisel.place);
    fill_the_bumps(seed, &mut bumps);
    let rings = theme::RINGS;
    let segments = theme::SEGMENTS;
    reach.clear();
    place.clear();
    for ring in 0..=rings {
        let down = std::f32::consts::PI * ring as f32 / rings as f32;
        let flat = down.sin();
        let high = down.cos();
        for segment in 0..=segments {
            let around = std::f32::consts::TAU * segment as f32 / segments as f32;
            let toward = Vec3::new(flat * around.cos(), high, flat * around.sin());
            let mut out = 1.0;
            for bump in bumps.iter() {
                let lean = toward.dot(&bump.toward) * theme::BUMP_TIGHT - theme::BUMP_TIGHT + 1.0;
                if lean > 0.0 {
                    out += bump.weight * lean * lean * lean;
                }
            }
            reach.push(out);
            place.push(toward * (theme::STATUE_SIZE * out));
        }
    }
    let mut vertices = Vec::with_capacity(place.len());
    for ring in 0..=rings {
        for segment in 0..=segments {
            let here = ring * (segments + 1) + segment;
            let left = ring * (segments + 1) + (segment + segments - 1) % segments;
            let right = ring * (segments + 1) + (segment + 1) % segments;
            let above = here.saturating_sub(segments + 1);
            let below = (here + segments + 1).min(place.len() - 1);
            let along = place[right] - place[left];
            let down = place[below] - place[above];
            let mut normal = along.cross(&down);
            if normal.norm() < 1.0e-6 {
                normal = place[here];
            }
            let mut vertex = Vertex::new(place[here], normal.normalize());
            vertex.color = stone_tint(reach[here], tint);
            vertices.push(vertex);
        }
    }
    let mut indices = Vec::with_capacity(rings * segments * 6);
    for ring in 0..rings {
        for segment in 0..segments {
            let first = (ring * (segments + 1) + segment) as u32;
            let second = first + 1;
            let third = first + (segments + 1) as u32;
            let fourth = third + 1;
            indices.extend([first, third, fourth, first, fourth, second]);
        }
    }
    chisel.bumps = bumps;
    chisel.reach = reach;
    chisel.place = place;
    Mesh::new(vertices, indices)
}

fn fill_the_bumps(seed: u64, bumps: &mut Vec<Bump>) {
    let mut roll = StdRng::seed_from_u64(seed);
    bumps.clear();
    bumps.reserve(theme::BUMPS);
    for _ in 0..theme::BUMPS {
        let high = roll.random_range(-1.0f32..1.0);
        let around = roll.random_range(0.0f32..std::f32::consts::TAU);
        let flat = (1.0 - high * high).max(0.0).sqrt();
        bumps.push(Bump {
            toward: Vec3::new(flat * around.cos(), high, flat * around.sin()),
            weight: theme::BUMP_WEIGHT * roll.random_range(-1.0f32..1.0),
        });
    }
}

fn stone_tint(reach: f32, tint: usize) -> [f32; 4] {
    let share = ((reach - 0.8) * 1.6).clamp(0.0, 1.0);
    let stone = theme::LOW_STONE * (1.0 - share) + theme::HIGH_STONE * share;
    let color = stone.component_mul(&theme::TINTS[tint % theme::TINTS.len()]);
    [color.x, color.y, color.z, 1.0]
}
