use crate::commands::world::spawn_marked_blocks;
use crate::components::{Lit, Upright};
use crate::data::{Furnishing, HouseKind};
use crate::queries::interior::center;
use crate::queries::sheet::{standing, standing_wide};
use crate::resources::{Home, Sheet};
use crate::tags::Indoors;
use crate::theme;
use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{
    spawn_instanced_mesh_with_material, spawn_mesh_with_material,
};
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::mesh::InstanceTransform;
use nightshade_sprite::prelude::*;

pub fn spawn_room(commands: &mut Commands, sheet: &Sheet, home: &mut Home, kind: HouseKind) {
    let room = kind.room();
    let house = kind.house();
    let y = theme::INDOOR_Y;
    home.house = Some(kind);
    home.columns = room.columns;
    home.rows = room.rows;
    home.blocked = vec![false; (room.columns * room.rows) as usize];
    home.exit = Vec2::new(0.0, 0.5 - room.rows as f32 / 2.0);
    home.bed = None;
    home.counter = None;

    let mut tiles = Vec::new();
    for row in 0..room.rows {
        for column in 0..room.columns {
            let middle = center(home, column, row);
            tiles.push(InstanceTransform::new(
                Vec3::new(middle.x, y - theme::CAP_THICKNESS / 2.0, middle.y),
                Quat::identity(),
                Vec3::new(1.0, theme::CAP_THICKNESS, 1.0),
            ));
        }
    }
    let floor_material = room.floor;
    commands.queue(move |storage, resources| {
        let floor_material_id =
            nightshade_render::queries::material_registry::material_registry_resolve(
                resources
                    .res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                floor_material,
            )
            .unwrap_or_default();
        let floor = spawn_instanced_mesh_with_material(
            storage,
            resources,
            "Cube",
            tiles,
            floor_material_id,
        );
        storage.add_tag_type::<Indoors>(floor);
    });

    let half_width = room.columns as f32 / 2.0;
    let half_depth = room.rows as f32 / 2.0;
    let wall = house.wall.material();
    let mut blocks = Vec::new();
    for level in 0..theme::INDOOR_WALL_LEVELS {
        let middle_y = y + level as f32 + 0.5;
        for cell in 0..room.columns + 2 {
            blocks.push(InstanceTransform::new(
                Vec3::new(
                    cell as f32 + 0.5 - half_width - 1.0,
                    middle_y,
                    -half_depth - 0.5,
                ),
                Quat::identity(),
                Vec3::new(1.0, 1.0, 1.0),
            ));
        }
        for side in [-1.0, 1.0] {
            for cell in 0..room.rows {
                blocks.push(InstanceTransform::new(
                    Vec3::new(
                        side * (half_width + 0.5),
                        middle_y,
                        cell as f32 + 0.5 - half_depth,
                    ),
                    Quat::identity(),
                    Vec3::new(1.0, 1.0, 1.0),
                ));
            }
        }
    }
    spawn_marked_blocks::<Indoors>(commands, wall, blocks);

    let leaf = theme::INDOOR_DOOR_THICKNESS;
    spawn_marked_blocks::<Indoors>(
        commands,
        theme::DOOR_MATERIAL,
        vec![InstanceTransform::new(
            Vec3::new(
                0.0,
                y + theme::INDOOR_DOOR_HEIGHT / 2.0,
                leaf / 2.0 - half_depth,
            ),
            Quat::identity(),
            Vec3::new(theme::INDOOR_DOOR_WIDTH, theme::INDOOR_DOOR_HEIGHT, leaf),
        )],
    );

    let roof = spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(0.0, y + theme::HOLLOW_ROOF_HEIGHT, 0.0),
        Vec3::new(theme::HOLLOW_ROCK_SIZE, 0.4, theme::HOLLOW_ROCK_SIZE),
        theme::DUNGEON_ROCK_MATERIAL,
    );
    commands.add_tag_type::<Indoors>(roof);
    let ground = spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(0.0, y - 0.6, 0.0),
        Vec3::new(theme::HOLLOW_ROCK_SIZE, 0.8, theme::HOLLOW_ROCK_SIZE),
        theme::DUNGEON_ROCK_MATERIAL,
    );
    commands.add_tag_type::<Indoors>(ground);

    let lamp = spawn_light_entity(
        commands,
        Vec3::new(0.0, y + theme::INDOOR_ROOF_HEIGHT, 0.0),
        "Room lamp",
    );
    commands.set(
        lamp,
        Light {
            light_type: LightType::Point,
            color: theme::INDOOR_LAMP_COLOR,
            intensity: theme::INDOOR_LAMP_STRENGTH,
            range: theme::INDOOR_LAMP_RANGE,
            ..Default::default()
        },
    );
    commands.add_tag_type::<Indoors>(lamp);

    for piece in room.pieces {
        let span = piece.span.max(1) as f32;
        let middle = center(home, piece.column, piece.row) + Vec2::new((span - 1.0) / 2.0, 0.0);
        let spot = Vec3::new(middle.x, y, middle.y);
        let (region, height) = if piece.span > 1 {
            standing_wide(sheet, piece.tile)
        } else {
            standing(sheet, piece.tile)
        };
        let sprite = Sprite::new(theme::ATLAS, region)
            .with_size(span, height)
            .with_anchor(Vec2::new(0.5, 0.0))
            .with_layer(theme::LAYER_STANDING);
        let entity = spawn_sprite(commands, sprite, spot);
        commands.set(entity, Upright::default());
        commands.set(
            entity,
            Lit {
                base: [1.0, 1.0, 1.0, 1.0],
            },
        );
        commands.add_tag_type::<Indoors>(entity);
        for step in 0..piece.span.max(1) {
            let cell = (piece.row * room.columns + piece.column + step) as usize;
            if let Some(blocked) = home.blocked.get_mut(cell) {
                *blocked = true;
            }
        }
        match piece.role {
            Furnishing::Bed => home.bed = Some(middle),
            Furnishing::Counter => home.counter = Some(middle),
            Furnishing::Plain => {}
        }
    }
    if house.shop.is_none() {
        home.counter = None;
    }
}
