use crate::contract::{ChunkLayer, SlabKey};
use crate::data;
use crate::queries::{column_of, slab_of_column};
use crate::resources::{Meshes, VoxelSettings, VoxelViewer};
use nightshade::prelude::{Commands, Res, ResMut};
use nightshade_render::components::culling::CastsShadow;

pub fn cast_the_shadows(
    viewer: Res<VoxelViewer>,
    settings: Res<VoxelSettings>,
    mut meshes: ResMut<Meshes>,
    mut commands: Commands,
) {
    let eye = viewer.eye / settings.block_size;
    let center = column_of(eye.x.floor() as i32, eye.z.floor() as i32);
    let radius = settings.shadow_radius.div_euclid(data::SLAB) + 1;
    let levels = settings.shadow_levels;
    let standing = eye.y.floor() as i32;
    let middle = slab_of_column(center, standing.div_euclid(data::SLAB_HIGH));
    if meshes.center != Some(middle) {
        meshes.center = Some(middle);
        let mut wanted = Vec::new();
        for level in middle[1] - levels..=middle[1] + levels {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let key = [middle[0] + dx, level, middle[2] + dz];
                    if !meshes.casting.contains(&key) {
                        wanted.push(key);
                    }
                }
            }
        }
        let gone: Vec<SlabKey> = meshes
            .casting
            .iter()
            .filter(|key| {
                (key[0] - middle[0]).abs() > radius
                    || (key[2] - middle[2]).abs() > radius
                    || (key[1] - middle[1]).abs() > levels
            })
            .copied()
            .collect();
        for key in wanted {
            meshes.casting.insert(key);
            meshes.ring.push_back((key, true));
        }
        for key in gone {
            meshes.casting.remove(&key);
            meshes.ring.push_back((key, false));
        }
    }
    for _ in 0..data::SHADOW_STEP {
        let Some((key, casts)) = meshes.ring.pop_front() else {
            break;
        };
        if casts != meshes.casting.contains(&key) {
            continue;
        }
        for layer in [ChunkLayer::Solid, ChunkLayer::Cutout] {
            let Some(entity) = meshes.entities.get(&(key, layer)).copied() else {
                continue;
            };
            if casts {
                commands.set(entity, CastsShadow);
                continue;
            }
            commands.queue(move |storage, _| {
                storage.remove_components::<(CastsShadow,)>(entity);
            });
        }
    }
}
