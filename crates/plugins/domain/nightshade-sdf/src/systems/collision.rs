use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_scene::contract::GlobalTransform;
use nightshade_worker::{workers_submit, workers_take};

use crate::contract::SdfCollisionFocus;
use crate::contract::field::uniform_scale;
use crate::contract::render_data::{SdfChunkKey, SdfEditInstance, SdfFrame};
use crate::data::collision::{CHUNK_CELLS, COLLISION_TAKEN};
use crate::data::surface::{chunk_corner, chunk_of};
use crate::resources::{SdfCollision, SurfaceJob};

pub type SdfFocus<'world, 'item> =
    Query<'world, (&'item SdfCollisionFocus, &'item GlobalTransform)>;

pub fn sdf_collision_system(
    frame: Res<SdfFrame>,
    mut collision: ResMut<SdfCollision>,
    focus: SdfFocus,
) {
    collision.ready.clear();
    collision.gone.clear();
    let mut foci: Vec<(f32, Vec3)> = Vec::new();
    focus.for_each(|_entity, (reach, global_transform)| {
        foci.push((reach.0, global_transform.0.column(3).xyz()));
    });
    if foci.is_empty() {
        let dropped: Vec<SdfChunkKey> = collision.built.drain().collect();
        collision.stale.clear();
        collision.pending.clear();
        collision.gone = dropped;
        return;
    }

    let cell = collision.cell;
    let span = CHUNK_CELLS as f32 * cell;
    let ground: Vec<SdfEditInstance> = frame
        .edits
        .iter()
        .filter(|edit| edit.collides)
        .cloned()
        .collect();
    if collision.edits.as_slice() != ground.as_slice() {
        let touched = touched_by(&collision.edits, &ground);
        let stale: Vec<SdfChunkKey> = match touched {
            None => collision.built.union(&collision.pending).copied().collect(),
            Some(spheres) => collision
                .built
                .union(&collision.pending)
                .copied()
                .filter(|key| reaches(&spheres, *key, span, cell))
                .collect(),
        };
        collision.stale.extend(stale);
        collision.edits = Arc::new(ground);
        collision.stamp += 1;
    }

    let mut nearest: HashMap<SdfChunkKey, f32> = HashMap::new();
    for (reach, middle) in &foci {
        let steps = (reach / span).ceil().max(1.0) as i32;
        let home = chunk_of(middle, cell);
        for x in -steps..=steps {
            for y in -steps..=steps {
                for z in -steps..=steps {
                    let key = [home[0] + x, home[1] + y, home[2] + z];
                    let gap = (nearest_in(key, span, cell, middle) - middle).magnitude();
                    if gap <= *reach {
                        let held = nearest.entry(key).or_insert(gap);
                        *held = held.min(gap);
                    }
                }
            }
        }
    }
    let mut wanted: Vec<(f32, SdfChunkKey)> =
        nearest.iter().map(|(key, gap)| (*gap, *key)).collect();
    wanted.sort_by(|one, two| one.0.total_cmp(&two.0));
    let held: HashSet<SdfChunkKey> = nearest.into_keys().collect();

    let dropped: Vec<SdfChunkKey> = collision
        .built
        .iter()
        .copied()
        .filter(|key| !held.contains(key))
        .collect();
    for key in &dropped {
        collision.built.remove(key);
        collision.stale.remove(key);
    }
    collision.pending.retain(|key| held.contains(key));
    collision.gone = dropped;

    let stamp = collision.stamp;
    let edits = Arc::clone(&collision.edits);
    for (gap, key) in &wanted {
        if collision.built.contains(key) && !collision.stale.contains(key) {
            continue;
        }
        let job = SurfaceJob {
            key: *key,
            cell,
            stamp,
            edits: Arc::clone(&edits),
        };
        if workers_submit(&mut collision.crew, job, *gap as i32) {
            collision.pending.insert(*key);
        }
    }

    for result in workers_take(&mut collision.crew, COLLISION_TAKEN) {
        collision.pending.remove(&result.key);
        if !held.contains(&result.key) {
            continue;
        }
        collision.built.insert(result.key);
        if result.stamp == stamp {
            collision.stale.remove(&result.key);
        }
        collision.ready.push((result.key, result.mesh));
    }
}

fn touched_by(before: &[SdfEditInstance], after: &[SdfEditInstance]) -> Option<Vec<(Vec3, f32)>> {
    let mut spheres = Vec::new();
    let room = |edit: &SdfEditInstance| -> Option<(Vec3, f32)> {
        let reach = edit.shape.reach()?;
        Some((
            edit.transform.column(3).xyz(),
            reach * uniform_scale(&edit.transform) + edit.blend.max(0.0),
        ))
    };
    let mut earlier: HashMap<Entity, &SdfEditInstance> =
        before.iter().map(|edit| (edit.entity, edit)).collect();
    for now in after {
        match earlier.remove(&now.entity) {
            Some(was) if was == now => {}
            Some(was) => {
                spheres.push(room(was)?);
                spheres.push(room(now)?);
            }
            None => spheres.push(room(now)?),
        }
    }
    for was in earlier.values() {
        spheres.push(room(was)?);
    }
    Some(spheres)
}

fn nearest_in(key: SdfChunkKey, span: f32, cell: f32, point: &Vec3) -> Vec3 {
    let corner = chunk_corner(key, cell);
    Vec3::new(
        point.x.clamp(corner.x, corner.x + span),
        point.y.clamp(corner.y, corner.y + span),
        point.z.clamp(corner.z, corner.z + span),
    )
}

fn reaches(spheres: &[(Vec3, f32)], key: SdfChunkKey, span: f32, cell: f32) -> bool {
    spheres.iter().any(|(middle, radius)| {
        (nearest_in(key, span, cell, middle) - middle).magnitude() <= *radius
    })
}
