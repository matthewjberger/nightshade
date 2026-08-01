use crate::commands::set_block;
use crate::contract::{AIR, Blocks, Face};
use crate::data;
use crate::queries::{block_at, chunk_of, cube_mesh, is_solid_at};
use crate::resources::{Fall, Falling, VoxelSettings, VoxelWorld};
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::{Commands, Res, ResMut};
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_platform::contract::time::Time;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_render::queries::material_registry::material_registry_resolve;
use nightshade_scene::contract::primitives::{Name, Visibility};
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

pub fn settle_the_loose_blocks(
    settings: Res<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut falling: ResMut<Falling>,
    mut cache: ResMut<MeshCache>,
    materials: Res<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut commands: Commands,
) {
    if world.updates.is_empty() {
        return;
    }
    let mut pending = std::mem::take(&mut world.updates);
    if pending.len() > data::FALLS_PER_FRAME {
        world.updates = pending.split_off(data::FALLS_PER_FRAME);
    }
    let table = world.blocks.clone();
    let up = IVec3::new(0, 1, 0);
    for position in pending {
        let block = block_at(&world, position);
        if block == AIR {
            continue;
        }
        if table.falls(block) && table.replaceable(block_at(&world, position - up)) {
            if falling.blocks.len() >= data::FALLING_LIMIT {
                world.updates.push(position);
                continue;
            }
            set_block(&mut world, position, AIR);
            let entity = spawn_falling_block(
                &settings,
                &materials,
                &table,
                &mut falling,
                &mut cache,
                &mut commands,
                block,
                position,
            );
            falling.blocks.push(Fall {
                block,
                position: position.cast::<f32>(),
                speed: 0.0,
                entity,
            });
        } else if table.tall(block) {
            let below = block_at(&world, position - up);
            let above = block_at(&world, position + up);
            let held = below == block || (above == block && is_solid_at(&world, position - up));
            if !held {
                set_block(&mut world, position, AIR);
            }
        } else if let Some(support) = table.support(block)
            && !is_solid_at(
                &world,
                position + IVec3::new(support[0], support[1], support[2]),
            )
        {
            set_block(&mut world, position, AIR);
            world.fallen.push((position, block));
        }
    }
}

pub fn drop_the_falling(
    time: Res<Time>,
    settings: Res<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut falling: ResMut<Falling>,
    mut commands: Commands,
) {
    if falling.blocks.is_empty() {
        return;
    }
    let delta_time = time.delta_time.min(data::FALL_STEP_LIMIT);
    let gain = data::FALL_ACCELERATION / settings.block_size * delta_time;
    let terminal = data::FALL_TERMINAL / settings.block_size;
    let table = world.blocks.clone();
    let mut gone: Vec<Entity> = Vec::new();
    let mut blocks = std::mem::take(&mut falling.blocks);
    blocks.retain_mut(|fall| {
        fall.speed = (fall.speed + gain).min(terminal);
        let from = fall.position.y.floor() as i32;
        let next = fall.position.y - fall.speed * delta_time;
        let to = next.floor() as i32;
        let x = fall.position.x as i32;
        let z = fall.position.z as i32;
        for cell in (to..from).rev() {
            let at = IVec3::new(x, cell, z);
            if cell < data::FLOOR_Y || !world.chunks.contains_key(&chunk_of(at)) {
                gone.push(fall.entity);
                return false;
            }
            if !table.replaceable(block_at(&world, at)) {
                land(&mut world, &table, fall.block, IVec3::new(x, cell + 1, z));
                gone.push(fall.entity);
                return false;
            }
        }
        fall.position.y = next;
        commands.set(
            fall.entity,
            LocalTransform {
                translation: fall.position * settings.block_size,
                scale: Vec3::repeat(settings.block_size),
                ..Default::default()
            },
        );
        true
    });
    falling.blocks = blocks;
    if !gone.is_empty() {
        commands.queue(move |storage, resources| {
            despawn_entities_with_cache_cleanup(storage, resources, &gone);
        });
    }
}

fn land(world: &mut VoxelWorld, table: &Blocks, block: u8, position: IVec3) {
    let mut at = position;
    for _ in 0..data::FALL_SEARCH {
        if table.replaceable(block_at(world, at)) {
            set_block(world, at, block);
            return;
        }
        at.y += 1;
    }
    world.fallen.push((at, block));
}

fn spawn_falling_block(
    settings: &VoxelSettings,
    materials: &nightshade_render::resources::material_registry::MaterialRegistry,
    blocks: &Blocks,
    falling: &mut Falling,
    cache: &mut MeshCache,
    commands: &mut Commands,
    block: u8,
    position: IVec3,
) -> Entity {
    let name = fall_mesh_name(block);
    let look = blocks.look(block);
    if falling.meshed.insert(block) {
        let mesh = cube_mesh(
            blocks,
            |face| match face {
                Face::Top => look.top,
                Face::Bottom => look.bottom,
                _ => look.side,
            },
            0.0,
            1.0,
        );
        mesh_cache_insert(cache, name.clone(), mesh);
        mesh_cache_protect(cache, name.clone());
    }
    let material_name = if look.cutout {
        &settings.cutout_material
    } else if look.transparent {
        &settings.transparent_material
    } else {
        &settings.solid_material
    };
    let material = material_registry_resolve(materials, material_name).unwrap_or_default();
    let mut transform =
        LocalTransform::from_translation(position.cast::<f32>() * settings.block_size);
    transform.scale = Vec3::repeat(settings.block_size);
    let entity = commands.spawn((
        Name(name.clone()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(&name),
        MaterialRef::new(material),
        Visibility { visible: true },
    ));
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

fn fall_mesh_name(block: u8) -> String {
    format!("voxels_fall_{block}")
}
