use crate::commands::{drop_the_entity, spawn_boulder, spawn_tile};
use crate::components::TileKey;
use crate::queries::{
    reach_of, scatter_boulders, tile_mesh, tile_mesh_name, tile_of, too_far, wanted_tiles,
};
use crate::resources::{Bench, Ground, Noises, Plan, Standing, TileJob, TileWork};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_render::commands::mesh_cache::{
    mesh_cache_insert, mesh_cache_protect, mesh_cache_release,
};
use nightshade_render::data::geometry::Mesh;
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_scene::prelude::LocalTransform;
use nightshade_worker::prelude::*;
use std::collections::HashSet;

impl Job for TileJob {
    type Key = [i32; 2];
    type Done = TileWork;
    type Tools = Bench;

    fn key(&self) -> Self::Key {
        self.key
    }

    fn run(self, tools: &mut Self::Tools) -> Self::Done {
        if tools.made.as_ref().map(|held| held.0) != Some(self.seed) {
            tools.made = Some((self.seed, Noises::new(self.seed)));
        }
        let Some((_, noises)) = tools.made.as_ref() else {
            return TileWork {
                key: self.key,
                mesh: Mesh::new(Vec::new(), Vec::new()),
                lowest: 0.0,
                highest: 0.0,
                boulders: Vec::new(),
            };
        };
        let (mesh, lowest, highest) = tile_mesh(noises, self.key, self.cells);
        let boulders = scatter_boulders(noises, self.key, self.boulders);
        TileWork {
            key: self.key,
            mesh,
            lowest,
            highest,
            boulders,
        }
    }
}

pub fn stream_the_ground(
    active: Res<ActiveCamera>,
    plan: Res<Plan>,
    mut ground: ResMut<Ground>,
    mut diggers: ResMut<Workers<TileJob>>,
    cameras: Query<(&LocalTransform, &Camera)>,
) {
    let watching = active.0;
    let mut standing = None;
    cameras.for_each(|entity, (transform, _camera)| {
        if Some(entity) == watching {
            standing = Some(transform.translation);
        }
    });
    let Some(place) = standing else {
        return;
    };

    let center = tile_of(place.x, place.z);
    if ground.center != Some(center) {
        ground.center = Some(center);
        let gone: Vec<[i32; 2]> = ground
            .standing
            .keys()
            .copied()
            .filter(|key| too_far(*key, center, plan.radius))
            .collect();
        ground.leaving = gone;
    }

    for key in wanted_tiles(center, plan.radius) {
        if workers_in_flight(&diggers) >= theme::TILES_IN_FLIGHT {
            break;
        }
        if ground.standing.contains_key(&key) || workers_holds(&diggers, key) {
            continue;
        }
        workers_submit(
            &mut diggers,
            TileJob {
                key,
                cells: plan.cells,
                boulders: plan.boulders,
                seed: plan.seed,
            },
            reach_of(key, center),
        );
    }
}

pub fn place_the_tiles(
    plan: Res<Plan>,
    mut ground: ResMut<Ground>,
    mut diggers: ResMut<Workers<TileJob>>,
    mut meshes: ResMut<MeshCache>,
    mut commands: Commands,
) {
    let finished = workers_take(&mut diggers, theme::TILES_TAKEN);
    if finished.is_empty() {
        return;
    }
    let Some(center) = ground.center else {
        return;
    };
    for work in finished {
        if work.mesh.vertices.is_empty()
            || too_far(work.key, center, plan.radius)
            || ground.standing.contains_key(&work.key)
        {
            continue;
        }
        let triangles = work.mesh.indices.len() / 3;
        let name = tile_mesh_name(work.key);
        mesh_cache_insert(&mut meshes, name.clone(), work.mesh);
        mesh_cache_protect(&mut meshes, name.clone());
        spawn_tile(&mut commands, work.key, &name, work.lowest, work.highest);
        for boulder in &work.boulders {
            spawn_boulder(&mut commands, work.key, *boulder);
        }
        ground.standing.insert(
            work.key,
            Standing {
                boulders: work.boulders.len(),
                triangles,
            },
        );
    }
}

pub fn clear_the_tiles(
    mut ground: ResMut<Ground>,
    mut diggers: ResMut<Workers<TileJob>>,
    mut meshes: ResMut<MeshCache>,
    mut commands: Commands,
    held: Query<(&TileKey,)>,
) {
    if ground.leaving.is_empty() {
        return;
    }
    let taken = theme::TILES_TAKEN.min(ground.leaving.len());
    let leaving: Vec<[i32; 2]> = ground.leaving.drain(..taken).collect();
    let going: HashSet<[i32; 2]> = leaving.iter().copied().collect();

    let mut doomed: Vec<Entity> = Vec::new();
    held.for_each(|entity, (key,)| {
        if going.contains(&key.0) {
            doomed.push(entity);
        }
    });
    for entity in doomed {
        drop_the_entity(&mut commands, entity);
    }

    for key in leaving {
        workers_cancel(&mut diggers, key);
        mesh_cache_release(&mut meshes, &tile_mesh_name(key));
        ground.standing.remove(&key);
    }
}
