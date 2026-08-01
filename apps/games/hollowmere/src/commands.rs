use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
pub mod random;
use nightshade_render::data::material::solid_material;
pub mod audio;
pub mod chat;
pub mod creatures;
pub mod effects;
pub mod models;
pub mod places;
pub mod sky;
pub mod spells;

use crate::components::{
    Age, Block, Count, Experience, Fuse, Heading, Held, Level, Phase, Progress, Spark, Stage,
    Stride, Velocity, Watered,
};
use crate::contract::{Channel, ChatLine};
use crate::data::{
    ART_ITEMS, BlockKind, CHEST_ART, CropKind, Face, Item, ItemStack, PICKUPS, Recipe, SPAWNS,
    Shape, art_color, block_table,
};
use crate::queries::{
    art_mesh, can_craft, column_of, column_settled, crack_mesh, cube_mesh, extruded_cell_mesh,
    flat, ground_at, inventory_count, item_scale, level_threshold, liquid_near, ripple_pixels,
    room_wall, sheet_standing, stand_at,
};
use crate::resources::{
    Banner, Banners, ChatLog, Fireworks, Flash, Flashes, Hit, HitSource, Hits, Inventory,
    KillEvent, Scheduled, Sheet, Slain, Toast, VoxelWorld,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3, Vec4, vec2, vec4};
use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::material::{AlphaMode, Material, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::{GlobalTransform, Lifetime, LocalTransform, Name, Visibility};
use nightshade_sprite::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;
use nightshade_voxels::prelude::{Blocks, write_block};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Walk {
    Arrived,
    Moved,
    Blocked,
}

pub fn place_on_ground(
    world: &VoxelWorld,
    transform: &mut LocalTransform,
    heading: &Heading,
    stride: &Stride,
) {
    let bob = (stride.0 * 6.0).sin().abs() * 0.05;
    if let Some(floor) = stand_at(world, flat(transform.translation), transform.translation.y) {
        transform.translation.y = floor + bob;
    }
    transform.rotation = nalgebra_glm::quat_angle_axis(heading.0, &Vec3::y());
}

pub fn walk_toward(
    world: &VoxelWorld,
    transform: &mut LocalTransform,
    heading: &mut Heading,
    stride: &mut Stride,
    goal: Vec3,
    speed: f32,
    delta_time: f32,
) -> Walk {
    let here = transform.translation;
    let to_goal = Vec3::new(goal.x - here.x, 0.0, goal.z - here.z);
    let distance = to_goal.magnitude();
    if distance <= theme::ARRIVE_REACH {
        return Walk::Arrived;
    }
    let step = (speed * delta_time).min(distance);
    let straight = to_goal / distance;
    let turns = [0.0, 1.05, -1.05];
    for turn in turns {
        let direction = nalgebra_glm::quat_rotate_vec3(
            &nalgebra_glm::quat_angle_axis(turn, &Vec3::y()),
            &straight,
        );
        let next = here + direction * step;
        let (x, z) = (next.x.floor() as i32, next.z.floor() as i32);
        let floor = if column_settled(world, column_of(x, z)) {
            match ground_at(world, x, z, here.y, theme::NPC_RISE, theme::NPC_FALL) {
                Some(floor) => Some(floor),
                None => continue,
            }
        } else {
            None
        };
        transform.translation = Vec3::new(next.x, floor.unwrap_or(here.y), next.z);
        stride.0 += step;
        heading.0 = (-direction.x).atan2(-direction.z);
        place_on_ground(world, transform, heading, stride);
        return Walk::Moved;
    }
    heading.0 = (-straight.x).atan2(-straight.z);
    Walk::Blocked
}

pub fn teleport(
    world: &VoxelWorld,
    transform: &mut LocalTransform,
    heading: &Heading,
    stride: &Stride,
    spot: Vec3,
) {
    transform.translation = spot;
    place_on_ground(world, transform, heading, stride);
}

pub fn push_chat(chat: &mut ChatLog, channel: Channel, line: String) {
    chat.lines.push_back((channel, line));
    while chat.lines.len() > theme::CHAT_KEEP {
        chat.lines.pop_front();
    }
}

pub fn schedule_line(chat: &mut ChatLog, delay: f32, line: ChatLine) {
    chat.scheduled.push(Scheduled { delay, line });
}

pub fn record_hit(
    hits: &mut Hits,
    attacker: Entity,
    target: Entity,
    damage: f32,
    crit: bool,
    chill: bool,
    source: HitSource,
) {
    hits.pending.push(Hit {
        attacker,
        target,
        damage,
        crit,
        chill,
        source,
    });
}

pub fn record_kill(slain: &mut Slain, event: KillEvent) {
    slain.events.push(event);
}

pub fn flash(flashes: &mut Flashes, position: Vec3, text: String, color: Vec4, pop: f32) {
    flashes.pending.push(Flash {
        position,
        text,
        color,
        pop,
    });
}

pub fn banner(banners: &mut Banners, title: String, subtitle: String, color: Vec4) {
    banners.pending.push_back(Banner {
        title,
        subtitle,
        color,
    });
}

pub fn grant_experience(level: &mut Level, experience: &mut Experience, amount: u32) -> bool {
    experience.0 += amount;
    let mut leveled = false;
    loop {
        let threshold = level_threshold(level.0);
        if experience.0 < threshold {
            break;
        }
        experience.0 -= threshold;
        level.0 += 1;
        leveled = true;
    }
    leveled
}

pub fn spawn_rainfall(commands: &mut Commands, position: Vec3) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: theme::RAIN_REACH,
            },
            position,
            direction: Vec3::new(0.0, -1.0, 0.0),
            spawn_rate: theme::RAIN_RATE,
            burst_count: 0,
            particle_lifetime_min: 0.9,
            particle_lifetime_max: 1.2,
            initial_velocity_min: 13.0,
            initial_velocity_max: 16.0,
            velocity_spread: 0.03,
            gravity: Vec3::zeros(),
            drag: 0.0,
            size_start: 0.06,
            size_end: 0.06,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(0.72, 0.82, 1.0, 0.0)),
                    (0.1, vec4(0.72, 0.82, 1.0, 0.65)),
                    (0.9, vec4(0.72, 0.82, 1.0, 0.65)),
                    (1.0, vec4(0.72, 0.82, 1.0, 0.0)),
                ],
            },
            emissive_strength: 0.5,
            enabled: false,
            ..Default::default()
        },
    );
    entity
}

pub fn spawn_snowfall(commands: &mut Commands, position: Vec3) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: theme::RAIN_REACH,
            },
            position,
            direction: Vec3::new(0.0, -1.0, 0.0),
            spawn_rate: theme::SNOW_RATE,
            burst_count: 0,
            particle_lifetime_min: 5.0,
            particle_lifetime_max: 7.0,
            initial_velocity_min: 1.4,
            initial_velocity_max: 2.4,
            velocity_spread: 0.35,
            gravity: Vec3::zeros(),
            drag: 0.1,
            size_start: 0.11,
            size_end: 0.11,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 1.0, 0.0)),
                    (0.15, vec4(1.0, 1.0, 1.0, 0.9)),
                    (0.85, vec4(1.0, 1.0, 1.0, 0.9)),
                    (1.0, vec4(1.0, 1.0, 1.0, 0.0)),
                ],
            },
            emissive_strength: 0.3,
            enabled: false,
            ..Default::default()
        },
    );
    entity
}

pub fn inventory_add(inventory: &mut Inventory, item: Item, count: u32) -> u32 {
    let mut remaining = count;
    for slot in inventory.slots.iter_mut().flatten() {
        if remaining == 0 {
            break;
        }
        if slot.item == item && slot.count < item.max_stack() {
            let room = item.max_stack() - slot.count;
            let taken = room.min(remaining);
            slot.count += taken;
            remaining -= taken;
        }
    }
    for slot in inventory.slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if slot.is_none() {
            let taken = item.max_stack().min(remaining);
            *slot = Some(ItemStack { item, count: taken });
            remaining -= taken;
        }
    }
    remaining
}

pub fn inventory_take(inventory: &mut Inventory, item: Item, count: u32) -> bool {
    if inventory_count(inventory, item) < count {
        return false;
    }
    let mut remaining = count;
    for slot in inventory.slots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if let Some(stack) = slot
            && stack.item == item
        {
            let taken = stack.count.min(remaining);
            stack.count -= taken;
            remaining -= taken;
            if stack.count == 0 {
                *slot = None;
            }
        }
    }
    true
}

pub fn inventory_take_selected(inventory: &mut Inventory, count: u32) -> Option<ItemStack> {
    let slot = &mut inventory.slots[inventory.selected];
    let stack = (*slot)?;
    let taken = stack.count.min(count);
    if stack.count > taken {
        if let Some(stack) = slot.as_mut() {
            stack.count -= taken;
        }
    } else {
        *slot = None;
    }
    Some(ItemStack {
        item: stack.item,
        count: taken,
    })
}

pub fn inventory_swap_selected(inventory: &mut Inventory, item: Item) {
    let slot = &mut inventory.slots[inventory.selected];
    let count = slot.map(|stack| stack.count).unwrap_or(0);
    if count > 1 {
        if let Some(stack) = slot.as_mut() {
            stack.count -= 1;
        }
        inventory_add(inventory, item, 1);
        return;
    }
    *slot = Some(ItemStack { item, count: 1 });
}

pub fn swap_slots(inventory: &mut Inventory, first: usize, second: usize) {
    if first == second || first >= inventory.slots.len() || second >= inventory.slots.len() {
        return;
    }
    inventory.slots.swap(first, second);
}

pub fn select_slot(inventory: &mut Inventory, index: usize) {
    inventory.selected = index.min(theme::HOTBAR_SLOTS - 1);
}

pub fn select_next_slot(inventory: &mut Inventory, steps: i32) {
    let slots = theme::HOTBAR_SLOTS as i32;
    inventory.selected = (inventory.selected as i32 + steps).rem_euclid(slots) as usize;
}

pub fn craft(inventory: &mut Inventory, recipe: &Recipe) -> bool {
    if !can_craft(inventory, recipe) {
        return false;
    }
    for (item, count) in recipe.inputs {
        inventory_take(inventory, *item, *count);
    }
    let leftover = inventory_add(inventory, recipe.output, recipe.count);
    if leftover > 0 {
        for (item, count) in recipe.inputs {
            inventory_add(inventory, *item, *count);
        }
        inventory_take(inventory, recipe.output, recipe.count - leftover);
        return false;
    }
    true
}

pub fn toast_show(toast: &mut Toast, text: impl Into<String>) {
    toast.text = text.into();
    toast.remaining = theme::TOAST_SECONDS;
}

pub fn register_item_meshes(cache: &mut MeshCache, pixels: &[u8], width: u32) {
    let table = block_table();
    for kind in crate::data::BLOCKS.map(|row| row.kind) {
        if matches!(kind, BlockKind::Air | BlockKind::Water) {
            continue;
        }
        let name = Item::Block(kind).mesh_name();
        let info = kind.info();
        let mesh = if info.shape == Shape::Cube {
            cube_mesh(
                &table,
                |face| match face {
                    Face::Top => info.top,
                    Face::Bottom => info.bottom,
                    _ => info.side,
                },
                -0.5,
                0.5,
            )
        } else {
            extruded_cell_mesh(pixels, width, kind.info().side)
        };
        mesh_cache_insert(cache, name.clone(), mesh);
        mesh_cache_protect(cache, name);
    }
    for item in ART_ITEMS {
        let Some(art) = item.art() else {
            continue;
        };
        let name = item.mesh_name();
        mesh_cache_insert(cache, name.clone(), art_mesh(art));
        mesh_cache_protect(cache, name);
    }
    mesh_cache_insert(cache, theme::CRACK_MESH.to_string(), crack_mesh(&table, 0));
    mesh_cache_protect(cache, theme::CRACK_MESH.to_string());
}

pub fn update_crack_mesh(cache: &mut MeshCache, blocks: &Blocks, stage: usize) {
    mesh_cache_insert(
        cache,
        theme::CRACK_MESH.to_string(),
        crack_mesh(blocks, stage),
    );
}

pub fn load_ripples(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Linear,
        min_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
    };
    queue_decoded_texture(
        load,
        theme::RIPPLE_TEXTURE.to_string(),
        ripple_pixels(),
        theme::RIPPLE_TEXELS,
        theme::RIPPLE_TEXELS,
        TextureUsage::Linear,
        sampler,
    );
    texture_cache_protect(load.cache, theme::RIPPLE_TEXTURE.to_string());
}

pub fn load_atlas(load: &mut TextureLoad<'_>, pixels: Vec<u8>, width: u32, height: u32) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::ClampToEdge,
        wrap_v: SamplerWrap::ClampToEdge,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    queue_decoded_texture(
        load,
        theme::ATLAS_TEXTURE.to_string(),
        pixels,
        width,
        height,
        TextureUsage::Color,
        sampler,
    );
    texture_cache_protect(load.cache, theme::ATLAS_TEXTURE.to_string());
}

fn glowing(color: [f32; 4], strength: f32) -> Material {
    Material {
        base_color: color,
        emissive_factor: [
            color[0] * strength,
            color[1] * strength,
            color[2] * strength,
        ],
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn mob_material(index: usize) -> String {
    format!("hollowmere_mob_{index}")
}

pub fn pickup_material(index: usize) -> String {
    format!("hollowmere_pickup_{index}")
}

pub const STONE: &str = "hollowmere_stone";

pub fn insert_materials(registry: &mut MaterialRegistry) {
    insert_material(
        registry,
        theme::SOLID_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            atlas_cells: theme::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::CUTOUT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            atlas_cells: theme::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Mask,
            alpha_cutoff: 0.15,
            double_sided: false,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::TRANSPARENT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            atlas_cells: theme::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: false,
            roughness: theme::GLASS_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::WATER_MATERIAL,
        Material {
            base_color: theme::WATER_COLOR,
            base_texture: Some(theme::ATLAS_TEXTURE.to_string()),
            base_texture_transform: TextureTransform {
                drift: theme::WATER_DRIFT,
                ..Default::default()
            },
            atlas_cells: theme::ATLAS_CELLS,
            normal_texture: Some(theme::RIPPLE_TEXTURE.to_string()),
            normal_texture_transform: TextureTransform {
                scale: [theme::RIPPLE_SCALE, theme::RIPPLE_SCALE],
                drift: theme::RIPPLE_DRIFT,
                ..Default::default()
            },
            normal_scale: theme::RIPPLE_STRENGTH,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            roughness: theme::WATER_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::FLAT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.8,
            metallic: 0.1,
            ..Default::default()
        },
    );
    insert_material(registry, spells::FIRE, glowing(theme::FIRE_COLOR, 4.0));
    insert_material(registry, spells::FROST, glowing(theme::FROST_COLOR, 3.0));
    insert_material(registry, spells::HOLY, glowing(theme::HOLY_COLOR, 5.0));
    insert_material(registry, spells::LIFE, glowing(theme::LIFE_COLOR, 4.0));
    insert_material(
        registry,
        spells::ICE,
        Material {
            base_color: theme::ICE_COLOR,
            emissive_factor: [0.25, 0.4, 0.6],
            roughness: 0.2,
            metallic: 0.1,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        spells::SOOT,
        solid_material(theme::SOOT_COLOR, 1.0, 0.0),
    );
    insert_material(registry, spells::VIOLET, glowing(theme::VIOLET_COLOR, 5.0));
    insert_material(
        registry,
        STONE,
        solid_material(theme::STONE_COLOR, 0.9, 0.0),
    );
    insert_material(
        registry,
        sky::CLOUD,
        solid_material(theme::CLOUD_COLOR, 1.0, 0.0),
    );
    insert_material(
        registry,
        sky::FEATHER,
        solid_material(theme::FEATHER_COLOR, 0.9, 0.0),
    );
    for (index, spawn) in SPAWNS.iter().enumerate() {
        insert_material(
            registry,
            mob_material(index),
            solid_material(spawn.color, 0.85, 0.0),
        );
    }
    for (index, pickup) in PICKUPS.iter().enumerate() {
        insert_material(registry, pickup_material(index), glowing(pickup.color, 0.8));
    }
}

pub fn launch_firework(commands: &mut Commands, fireworks: &mut Fireworks, launch: Vec3) {
    let rng = &mut fireworks.rng;
    let velocity = Vec3::new(
        (rng.f32() - 0.5) * 2.5,
        theme::FIREWORK_SPEED + rng.f32() * 4.0,
        (rng.f32() - 0.5) * 2.5,
    );
    let color = theme::FIREWORK_COLORS[rng.usize(0..theme::FIREWORK_COLORS.len())];
    let entity = commands.spawn(());
    let mut trail = ParticleEmitter::firework_shell(launch, velocity);
    trail.size_start = 0.08;
    trail.size_end = 0.0;
    commands.set(entity, trail);
    commands.set(entity, Lifetime { remaining: 8.0 });
    commands.set(entity, Velocity(velocity));
    commands.set(
        entity,
        Fuse(theme::FIREWORK_FUSE + fireworks.rng.f32() * 0.4),
    );
    commands.set(entity, Spark(color));
}

pub fn plant(
    commands: &mut Commands,
    world: &VoxelWorld,
    sheet: &Sheet,
    kind: CropKind,
    block: IVec3,
) -> Entity {
    let (region, height) = sheet_standing(sheet, kind.info().stages[0]);
    let sprite = Sprite::new(theme::SPRITE_ATLAS, region)
        .with_size(0.9, height * 0.9)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::SPRITE_LAYER)
        .as_billboard();
    let position = Vec3::new(
        block.x as f32 + 0.5,
        block.y as f32 + 1.0,
        block.z as f32 + 0.5,
    );
    let entity = spawn_sprite(commands, sprite, position);
    commands.set(entity, kind);
    commands.set(entity, Block(block));
    commands.set(entity, Progress(0.0));
    commands.set(entity, Stage(0));
    commands.set(
        entity,
        Watered(liquid_near(world, block, theme::WATER_REACH, theme::WATER)),
    );
    entity
}

pub fn spawn_drop(
    commands: &mut Commands,
    item: Item,
    count: u32,
    position: Vec3,
    velocity: Vec3,
    phase: f32,
) -> Entity {
    let transform = LocalTransform {
        translation: position,
        scale: Vec3::repeat(item_scale(item)),
        ..Default::default()
    };
    let entity = commands.spawn((
        Name(format!("Dropped {}", item.name())),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(item.mesh_name()),
        MaterialRef::default(),
        CastsShadow,
        Visibility { visible: true },
        Held(item),
        Count(count),
        Velocity(velocity),
        Age(0.0),
        Phase(phase),
    ));
    set_mesh_material(commands, entity, item.material_name());
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

pub fn spawn_outline(commands: &mut Commands) -> Entity {
    let corners = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let edges = [
        (corners(0.0, 0.0, 0.0), corners(1.0, 0.0, 0.0)),
        (corners(1.0, 0.0, 0.0), corners(1.0, 0.0, 1.0)),
        (corners(1.0, 0.0, 1.0), corners(0.0, 0.0, 1.0)),
        (corners(0.0, 0.0, 1.0), corners(0.0, 0.0, 0.0)),
        (corners(0.0, 1.0, 0.0), corners(1.0, 1.0, 0.0)),
        (corners(1.0, 1.0, 0.0), corners(1.0, 1.0, 1.0)),
        (corners(1.0, 1.0, 1.0), corners(0.0, 1.0, 1.0)),
        (corners(0.0, 1.0, 1.0), corners(0.0, 1.0, 0.0)),
        (corners(0.0, 0.0, 0.0), corners(0.0, 1.0, 0.0)),
        (corners(1.0, 0.0, 0.0), corners(1.0, 1.0, 0.0)),
        (corners(1.0, 0.0, 1.0), corners(1.0, 1.0, 1.0)),
        (corners(0.0, 0.0, 1.0), corners(0.0, 1.0, 1.0)),
    ];
    let inflate = 0.003;
    let lines = edges
        .iter()
        .map(|(start, end)| Line {
            start: start.map(|value| value * (1.0 + 2.0 * inflate) - inflate),
            end: end.map(|value| value * (1.0 + 2.0 * inflate) - inflate),
            color: theme::OUTLINE_COLOR,
        })
        .collect();
    commands.spawn((
        Name("Block Outline".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Lines::new(lines),
        Visibility { visible: false },
    ))
}

pub fn spawn_crack(commands: &mut Commands) -> Entity {
    let entity = commands.spawn((
        Name("Block Crack".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        RenderMesh::new(theme::CRACK_MESH),
        MaterialRef::default(),
        Visibility { visible: false },
    ));
    set_mesh_material(commands, entity, theme::CUTOUT_MATERIAL);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}

pub fn pack_the_sheet(atlases: &mut SpriteAtlases) -> Sheet {
    sprite_atlas_create(
        atlases,
        theme::SPRITE_ATLAS,
        theme::SPRITE_ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    let decoded = decode_to_rgba8(theme::SHEET_BYTES).expect("the sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    let empty_rows_below = empty_rows_below(&decoded.rgba, width);
    let tiles = sprite_atlas_insert(
        atlases,
        theme::SPRITE_ATLAS,
        theme::SPRITE_SHEET,
        &decoded.rgba,
        width,
        height,
    )
    .expect("the sheet fits the atlas");
    Sheet {
        tiles,
        empty_rows_below,
    }
}

fn empty_rows_below(pixels: &[u8], width: u32) -> [u8; theme::SHEET_TILES] {
    let mut empty = [0u8; theme::SHEET_TILES];
    for (index, slot) in empty.iter_mut().enumerate() {
        let column = index as u32 % theme::SHEET_COLUMNS;
        let row = index as u32 / theme::SHEET_COLUMNS;
        let mut count = 0u8;
        for texel_row in (0..theme::TILE_TEXELS).rev() {
            let y = row * theme::TILE_TEXELS + texel_row;
            let clear = (0..theme::TILE_TEXELS).all(|texel_column| {
                let x = column * theme::TILE_TEXELS + texel_column;
                pixels[((y * width + x) * 4 + 3) as usize] == 0
            });
            if !clear {
                break;
            }
            count += 1;
        }
        *slot = count.min(15);
    }
    empty
}

pub fn build_room(
    world: &mut VoxelWorld,
    floor: IVec3,
    inside: (i32, i32),
    span: i32,
    chest: Option<IVec3>,
) {
    let wall = room_wall(inside, span);
    for step in 0..=theme::DUNGEON_TALL {
        let kind = if step == 0 || step == theme::DUNGEON_TALL || wall {
            BlockKind::Cobblestone
        } else {
            BlockKind::Air
        };
        write_block(
            world,
            IVec3::new(floor.x, floor.y + step, floor.z),
            kind.index(),
        );
    }
    if wall {
        return;
    }
    let lamp = inside.0 % theme::DUNGEON_ROOM == 4 && inside.1 % theme::DUNGEON_ROOM == 4;
    if lamp {
        let under_the_ceiling = floor.y + theme::DUNGEON_TALL - 1;
        write_block(
            world,
            IVec3::new(floor.x, under_the_ceiling, floor.z),
            BlockKind::Glowstone.index(),
        );
    }
    let stand = IVec3::new(floor.x, floor.y + 1, floor.z);
    if chest == Some(stand) {
        write_block(world, stand, BlockKind::Chest.index());
    }
}

pub fn frame(tree: &mut UiTreeBuilder, layer: Option<u32>, tint: Vec4, corner: f32) {
    let Some(layer) = layer else {
        return;
    };
    let band = 1.0 / theme::FRAME_BANDS;
    for row in 0..3 {
        for column in 0..3 {
            if row == 1 && column == 1 {
                continue;
            }
            let position = Rl(vec2(
                if column == 2 { 100.0 } else { 0.0 },
                if row == 2 { 100.0 } else { 0.0 },
            )) + Ab(vec2(
                match column {
                    0 => 0.0,
                    1 => corner,
                    _ => -corner,
                },
                match row {
                    0 => 0.0,
                    1 => corner,
                    _ => -corner,
                },
            ));
            let size = Rl(vec2(
                if column == 1 { 100.0 } else { 0.0 },
                if row == 1 { 100.0 } else { 0.0 },
            )) + Ab(vec2(
                if column == 1 { -2.0 * corner } else { corner },
                if row == 1 { -2.0 * corner } else { corner },
            ));
            tree.add_node()
                .window(position, size, Anchor::TopLeft)
                .with_image_uv(
                    layer,
                    vec2(column as f32 * band, row as f32 * band),
                    vec2((column + 1) as f32 * band, (row + 1) as f32 * band),
                )
                .color_raw::<UiBase>(tint)
                .without_pointer_events()
                .entity();
        }
    }
}

pub fn decode_atlas() -> (Vec<u8>, u32, u32) {
    let decoded = decode_to_rgba8(theme::ATLAS_BYTES).expect("the block atlas decodes");
    let (width, height) = (decoded.width, decoded.height);
    let mut pixels = decoded.rgba;
    for (cell, art) in CHEST_ART {
        paint_cell(&mut pixels, width, cell, art);
    }
    (pixels, width, height)
}

pub fn paint_cell(pixels: &mut [u8], width: u32, cell: (u32, u32), art: [&str; 16]) {
    for (row, line) in art.iter().enumerate() {
        for (column, mark) in line.chars().enumerate() {
            let Some(color) = art_color(mark) else {
                continue;
            };
            let x = cell.0 * theme::TILE_TEXELS + column as u32;
            let y = cell.1 * theme::TILE_TEXELS + row as u32;
            let offset = ((y * width + x) * 4) as usize;
            for channel in 0..3 {
                pixels[offset + channel] = (color[channel] * 255.0).round() as u8;
            }
            pixels[offset + 3] = 255;
        }
    }
}
