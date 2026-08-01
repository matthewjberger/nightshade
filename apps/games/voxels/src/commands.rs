use crate::components::{
    Age, Block, Boat, Count, Held, Home, NpcName, Phase, Progress, Reach, Rest, Roll, Speed, Stage,
    Target, Velocity, Watered,
};
use crate::data::art_color;
use crate::data::{
    ART_ITEMS, AfterTalk, BlockKind, CropKind, Face, Item, ItemStack, NpcRole, Recipe, Shape,
    block_table,
};
use crate::data::{CHEST_ART, CLASSIC_ART, Palette};
use crate::queries::empty_rows_below;
use crate::queries::{
    art_mesh, can_craft, crack_mesh, cube_mesh, extruded_cell_mesh, inventory_count, item_scale,
    liquid_near, reserved, ripple_pixels, sheet_standing, villages_near, walkable,
};
use crate::resources::{
    Dialogue, Fireworks, Generation, Inventory, Player, Sheet, Shell, Toast, VoxelWorld,
};
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{IVec3, Vec2, Vec3, vec4};
use nightshade::ecs::{ChildOf, Entity};
use nightshade::prelude::Commands;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::material::{AlphaMode, Material, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::{GlobalTransform, Lifetime, LocalTransform, Name, Visibility};
use nightshade_sprite::prelude::*;
use nightshade_text::prelude::*;
use nightshade_voxels::prelude::Blocks;

pub fn spawn_a_boat(commands: &mut Commands, place: Vec3, yaw: f32) -> Entity {
    let mut transform = LocalTransform::from_translation(place);
    transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let extent = Vec3::new(1.6, 0.8, 3.2);
    let entity = commands.spawn((
        Name("Boat".to_string()),
        Boat { yaw },
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(theme::BOAT_MESH),
        MaterialRef::default(),
        Visibility { visible: true },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(Vec3::zeros(), extent),
            extent.norm(),
        ),
    ));
    set_mesh_material(commands, entity, theme::FLAT_MATERIAL);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
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

pub fn set_watching(player: &mut Player, watching: bool) {
    player.watching = watching;
    player.watched = None;
    player.velocity = Vec3::zeros();
    player.watch_speed = theme::WATCH_SPEED;
    if watching {
        player.flying = true;
    }
}

pub fn toast_show(toast: &mut Toast, text: impl Into<String>) {
    toast.text = text.into();
    toast.remaining = theme::TOAST_SECONDS;
}

pub fn dialogue_start(
    dialogue: &mut Dialogue,
    speaker: &str,
    lines: Vec<String>,
    after: AfterTalk,
) {
    dialogue.speaker = speaker.to_string();
    dialogue.lines = lines;
    dialogue.line = 0;
    dialogue.revealed = 0.0;
    dialogue.after = after;
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
}

pub fn launch_firework(commands: &mut Commands, fireworks: &mut Fireworks, launch: Vec3) {
    let rng = &mut fireworks.rng;
    let velocity = Vec3::new(
        (rng.f32() - 0.5) * 2.5,
        theme::FIREWORK_SPEED + rng.f32() * 4.0,
        (rng.f32() - 0.5) * 2.5,
    );
    let color = theme::FIREWORK_COLORS
        [(rng.f32() * theme::FIREWORK_COLORS.len() as f32) as usize % theme::FIREWORK_COLORS.len()];
    let entity = commands.spawn(());
    let mut trail = ParticleEmitter::firework_shell(launch, velocity);
    trail.size_start = 0.08;
    trail.size_end = 0.0;
    commands.set(entity, trail);
    commands.set(entity, Lifetime { remaining: 8.0 });
    fireworks.shells.push(Shell {
        entity,
        position: launch,
        velocity,
        fuse: theme::FIREWORK_FUSE + fireworks.rng.f32() * 0.4,
        color,
    });
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
    let (tiles, empty_rows_below) = add_sheet(atlases, theme::SPRITE_SHEET, theme::SHEET_BYTES);
    let (dungeon_tiles, dungeon_rows) =
        add_sheet(atlases, theme::DUNGEON_SHEET, theme::DUNGEON_SHEET_BYTES);
    Sheet {
        tiles,
        empty_rows_below,
        dungeon_tiles,
        dungeon_rows,
    }
}

pub fn add_sheet(
    atlases: &mut SpriteAtlases,
    name: &str,
    bytes: &[u8],
) -> (SpriteRegion, [u8; theme::SHEET_TILES]) {
    let decoded = decode_to_rgba8(bytes).expect("the sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    let rows = empty_rows_below(&decoded.rgba, width);
    let tiles = sprite_atlas_insert(
        atlases,
        theme::SPRITE_ATLAS,
        name,
        &decoded.rgba,
        width,
        height,
    )
    .expect("the sheet fits the atlas");
    (tiles, rows)
}

pub fn populate_now(
    world: &VoxelWorld,
    generation: &Generation,
    sheet: &Sheet,
    text_state: &mut TextState,
    commands: &mut Commands,
) {
    let spawn = generation.spawn.unwrap_or((0, 0));
    let home = villages_near(generation, spawn.0, spawn.1)
        .min_by_key(|village| {
            (village.square.0 - spawn.0).abs() + (village.square.1 - spawn.1).abs()
        })
        .map(|village| (village.square, village.shop_door));
    let square = home.map(|(square, _)| square).unwrap_or(spawn);
    let shop_door = home.and_then(|(_, door)| door).unwrap_or(square);
    let mut rng = Rng::with_seed(4242_u64);

    if generation.plan.villages {
        if let Some(spot) = open_spot(world, generation, shop_door, 3, &mut rng) {
            let keeper = spawn_standing(commands, sheet, theme::SHOPKEEPER, spot, None);
            commands.set(keeper, NpcName("Bram"));
            commands.set(keeper, NpcRole::Shopkeeper);
            commands.set(keeper, Home(Vec2::new(spot.x, spot.z)));
            commands.set(keeper, Target(Vec2::new(spot.x, spot.z)));
            commands.set(keeper, Reach(1.5));
            commands.set(keeper, Speed(0.6));
            commands.set(keeper, Rest(4.0));
            commands.set(keeper, Roll(Rng::with_seed(91_u64)));
            spawn_label(text_state, commands, keeper, "Bram");
        }

        for (index, name) in ["Mira", "Tobin"].iter().enumerate() {
            let Some(spot) = open_spot(world, generation, square, 6, &mut rng) else {
                continue;
            };
            let villager = spawn_standing(
                commands,
                sheet,
                theme::VILLAGER,
                spot,
                Some(theme::VILLAGER_TINT),
            );
            commands.set(villager, NpcName(name));
            commands.set(villager, NpcRole::Villager);
            commands.set(villager, Home(Vec2::new(spot.x, spot.z)));
            commands.set(villager, Target(Vec2::new(spot.x, spot.z)));
            commands.set(villager, Reach(5.0));
            commands.set(villager, Speed(0.9));
            commands.set(villager, Rest(2.0 + index as f32));
            commands.set(villager, Roll(Rng::with_seed((500 + index as u32) as u64)));
            spawn_label(text_state, commands, villager, name);
        }
    }

    let herd = [
        (theme::COW, 0.9, 2.5),
        (theme::SHEEP, 0.8, 2.0),
        (theme::COW, 0.9, 2.5),
        (theme::GOAT, 1.1, 3.0),
        (theme::CHICKEN, 1.4, 3.0),
        (theme::CHICKEN, 1.4, 3.0),
        (theme::CHICKEN, 1.4, 3.0),
        (theme::SHEEP, 0.8, 2.0),
    ];
    for (index, (tile, speed, reach)) in herd.iter().enumerate() {
        let Some(spot) = open_spot(world, generation, square, 14, &mut rng) else {
            continue;
        };
        let animal = spawn_standing(commands, sheet, *tile, spot, None);
        commands.set(animal, Home(Vec2::new(spot.x, spot.z)));
        commands.set(animal, Target(Vec2::new(spot.x, spot.z)));
        commands.set(animal, Reach(*reach));
        commands.set(animal, Speed(*speed));
        commands.set(animal, Rest(1.0 + index as f32 * 0.6));
        commands.set(animal, Roll(Rng::with_seed((300 + index as u32) as u64)));
    }
}

pub fn open_spot(
    world: &VoxelWorld,
    generation: &Generation,
    around: (i32, i32),
    spread: i32,
    rng: &mut Rng,
) -> Option<Vec3> {
    for _ in 0..60 {
        let x = around.0 + ((rng.f32() - 0.5) * 2.0 * spread as f32) as i32;
        let z = around.1 + ((rng.f32() - 0.5) * 2.0 * spread as f32) as i32;
        if let Some(ground) = walkable(world, x, z)
            && ground > theme::SEA_LEVEL
            && !reserved(generation, x, z)
        {
            return Some(Vec3::new(
                x as f32 + 0.5,
                ground as f32 + 1.0,
                z as f32 + 0.5,
            ));
        }
    }
    None
}

pub fn spawn_standing(
    commands: &mut Commands,
    sheet: &Sheet,
    tile: u32,
    position: Vec3,
    tint: Option<[f32; 4]>,
) -> Entity {
    let (region, height) = sheet_standing(sheet, tile);
    let mut sprite = Sprite::new(theme::SPRITE_ATLAS, region)
        .with_size(1.0, height)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::SPRITE_LAYER)
        .as_billboard();
    if let Some(tint) = tint {
        sprite = sprite.with_color(tint);
    }
    spawn_sprite(commands, sprite, position)
}

pub fn spawn_label(
    text_state: &mut TextState,
    commands: &mut Commands,
    parent: Entity,
    text: &str,
) -> Entity {
    let label = spawn_3d_billboard_text_with_properties(
        text_state,
        commands,
        text,
        Vec3::new(0.0, theme::LABEL_HEIGHT, 0.0),
        TextProperties {
            font_size: 22.0,
            color: vec4(1.0, 1.0, 1.0, 0.95),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 1.5,
            outline_color: vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );
    commands.set(label, ChildOf(parent));
    label
}

pub fn clear_cell(pixels: &mut [u8], width: u32, cell: (u32, u32)) {
    for row in 0..theme::TILE_TEXELS {
        for column in 0..theme::TILE_TEXELS {
            let x = cell.0 * theme::TILE_TEXELS + column;
            let y = cell.1 * theme::TILE_TEXELS + row;
            let offset = ((y * width + x) * 4) as usize;
            pixels[offset..offset + 4].fill(0);
        }
    }
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

pub fn decode_atlas(palette: Palette) -> (Vec<u8>, u32, u32) {
    let decoded = decode_to_rgba8(palette.bytes()).expect("the block atlas decodes");
    let (width, height) = (decoded.width, decoded.height);
    let mut pixels = decoded.rgba;
    for (cell, tint) in palette.tinted_cells().iter().copied() {
        for row in 0..theme::TILE_TEXELS {
            for column in 0..theme::TILE_TEXELS {
                let x = cell.0 * theme::TILE_TEXELS + column;
                let y = cell.1 * theme::TILE_TEXELS + row;
                let offset = ((y * width + x) * 4) as usize;
                for channel in 0..3 {
                    let value = pixels[offset + channel] as f32 * tint[channel];
                    pixels[offset + channel] = value.round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    for (cell, art) in CHEST_ART {
        paint_cell(&mut pixels, width, cell, art);
    }
    if palette == Palette::Classic {
        for (cell, art) in CLASSIC_ART {
            clear_cell(&mut pixels, width, cell);
            paint_cell(&mut pixels, width, cell, art);
        }
    }
    (pixels, width, height)
}
