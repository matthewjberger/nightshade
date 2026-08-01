use crate::commands::attach_material;
use crate::components::{Col, ItemKind, Lit, RoomIndex, Row, Upper};
use crate::data::ask::Ask;
use crate::data::level::*;
use crate::data::rooms::{ROOMS, Room};
use crate::data::theme;
use crate::queries::{moved_across, moved_down, room_at, tile_of, tile_on};
use crate::resources::{AshfallResources, Screen};
use crate::systems::textures::solid;
use crate::tags::ChartMarker;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_input::Actions;
use nightshade_scene::prelude::*;
use nightshade_ui::prelude::ui_set_text;

pub fn build(mut resources: ResMut<AshfallResources>, mut commands: Commands) {
    let resources = &mut *resources;
    let camera = commands.spawn((
        Name("Chart Camera".to_string()),
        LocalTransform {
            translation: theme::CHART_ORIGIN + Vec3::new(0.0, theme::CHART_HEIGHT, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
            ..Default::default()
        },
        GlobalTransform::default(),
        Camera {
            projection: Projection::Orthographic(OrthographicCamera {
                x_mag: None,
                y_mag: theme::CHART_ZOOM,
                z_far: 1000.0,
                z_near: 0.01,
            }),
            smoothing: None,
        },
    ));
    resources.chart.camera = Some(camera);

    for (index, room) in ROOMS.iter().enumerate() {
        for lit in [false, true] {
            let color = if lit {
                theme::chart_room_color(room.kind)
            } else {
                theme::CHART_UNKNOWN
            };
            let piece = plate(
                &mut commands,
                spot(
                    (room.col0 + room.col1) as f32 * 0.5,
                    (room.row0 + room.row1) as f32 * 0.5,
                    0.0,
                ),
                Vec3::new(
                    (room.col1 - room.col0 + 1) as f32 - theme::CHART_GAP,
                    theme::CHART_PLATE,
                    (room.row1 - room.row0 + 1) as f32 - theme::CHART_GAP,
                ),
                color,
            );
            commands.set(piece, RoomIndex(index));
            commands.set(piece, Lit(lit));
        }
    }

    for row in 0..HEIGHT as i32 {
        for col in 0..WIDTH as i32 {
            for upper in [false, true] {
                let Some(color) = mark_color(tile_on(col, row, upper)) else {
                    continue;
                };
                let piece = plate(
                    &mut commands,
                    spot(col as f32, row as f32, theme::CHART_MARK_LIFT),
                    Vec3::new(theme::CHART_MARK, theme::CHART_PLATE, theme::CHART_MARK),
                    color,
                );
                commands.set(piece, Col(col));
                commands.set(piece, Row(row));
                commands.set(piece, Upper(upper));
            }
        }
    }

    let marker = plate(
        &mut commands,
        spot(0.0, 0.0, theme::CHART_MARKER_LIFT),
        Vec3::new(theme::CHART_MARKER, theme::CHART_PLATE, theme::CHART_MARKER),
        theme::CHART_HERE,
    );
    commands.add_tag_type::<ChartMarker>(marker);
}

pub fn update(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<AshfallResources>,
    mut commands: Commands,
    mut rooms: Query<(&RoomIndex, &Lit, &mut Visibility)>,
    mut marks: Query<(&Col, &Row, &Upper, &mut Visibility)>,
    mut markers: Query<(&mut LocalTransform, &mut Visibility), WithTag<ChartMarker>>,
    mut pickups: Query<(&ItemKind, &LocalTransform)>,
) {
    let resources = &mut *resources;
    let open = resources.screen == Screen::TownMap;
    if open != resources.chart.showing {
        resources.chart.showing = open;
        if open {
            resources.chart.floor = resources.player.upstairs;
        }
        let camera = if open {
            resources.chart.camera
        } else {
            resources.player.eyes
        };
        if let Some(camera) = camera {
            commands.queue(move |_, resources| {
                resources.res_mut::<ActiveCamera>().0 = Some(camera);
            });
        }
    }
    if !open {
        return;
    }

    if moved_across(&actions) != 0 || moved_down(&actions) != 0 {
        resources.chart.floor = !resources.chart.floor;
    }
    let upper = resources.chart.floor;
    let mut loot = vec![false; ROOMS.len()];
    pickups.for_each_mut(|_entity, (_, transform)| {
        let (col, row) = tile_of(&transform.translation);
        let high = transform.translation.y > theme::CHART_UPPER_LEVEL;
        if let Some(index) = room_at(col, row, high) {
            loot[index] = true;
        }
    });

    rooms.for_each_mut(|_entity, (index, lit, visibility)| {
        let room = &ROOMS[index.0];
        let known = explored(resources, room);
        let show = room.upper == upper && lit.0 == known;
        if visibility.visible != show {
            visibility.visible = show;
        }
    });
    marks.for_each_mut(|_entity, (col, row, above, visibility)| {
        let show = above.0 == upper && walked(resources, col.0, row.0, above.0);
        if visibility.visible != show {
            visibility.visible = show;
        }
    });
    let standing_here = upper == resources.player.upstairs;
    markers.for_each_mut(|_entity, (transform, visibility)| {
        transform.translation = spot(
            resources.run.here.0 as f32,
            resources.run.here.1 as f32,
            theme::CHART_MARKER_LIFT,
        );
        if visibility.visible != standing_here {
            visibility.visible = standing_here;
        }
    });

    let handles = resources.ui;
    if let Some(title) = handles.town_map_title {
        let kept = resources.run.found_notes.iter().filter(|got| **got).count();
        let street = if upper {
            " STREET LEVEL "
        } else {
            "[STREET LEVEL]"
        };
        let above = if upper {
            "[UPPER FLOOR]"
        } else {
            " UPPER FLOOR "
        };
        let text = format!("ASHFALL   {street}  {above}   {kept} papers kept");
        ui_set_text(&mut commands, title, &text);
    }
    if let Some(label) = handles.town_map_room {
        let here =
            room_at(resources.run.here.0, resources.run.here.1, upper).filter(|_| standing_here);
        let name = here.map(|index| ROOMS[index].name).unwrap_or("The fog");
        let left = here.is_some_and(|index| loot[index]);
        let text = if left {
            format!("{name}   something is still on the floor here")
        } else {
            name.to_string()
        };
        ui_set_text(&mut commands, label, &text);
    }
    resources.run.map_dirty = false;
}

fn walked(resources: &AshfallResources, col: i32, row: i32, upper: bool) -> bool {
    let seen = if upper {
        &resources.run.explored_upper
    } else {
        &resources.run.explored
    };
    let index = row as usize * WIDTH + col as usize;
    seen.get(index).copied().unwrap_or(false)
}

fn explored(resources: &AshfallResources, room: &Room) -> bool {
    let seen = if room.upper {
        &resources.run.explored_upper
    } else {
        &resources.run.explored
    };
    for row in room.row0..=room.row1 {
        for col in room.col0..=room.col1 {
            let index = row as usize * WIDTH + col as usize;
            if seen.get(index).copied().unwrap_or(false) {
                return true;
            }
        }
    }
    false
}

fn spot(col: f32, row: f32, lift: f32) -> Vec3 {
    theme::CHART_ORIGIN + Vec3::new(col - WIDTH as f32 * 0.5, lift, row - HEIGHT as f32 * 0.5)
}

fn plate(commands: &mut Commands, position: Vec3, scale: Vec3, color: [f32; 4]) -> Entity {
    let entity = spawn_mesh_at(commands, "Cube", position, scale);
    attach_material(
        commands,
        entity,
        format!(
            "ashfall_chart_{:0>3}_{:0>3}_{:0>3}",
            (color[0] * 255.0) as u32,
            (color[1] * 255.0) as u32,
            (color[2] * 255.0) as u32
        ),
        solid(color, [color[0] * 0.9, color[1] * 0.9, color[2] * 0.9]),
    );
    commands.set(entity, Visibility { visible: false });
    entity
}

fn mark_color(tile: u8) -> Option<[f32; 4]> {
    match tile {
        DOOR => Some(theme::CHART_DOOR),
        BRASS_DOOR | WARD_DOOR | BONE_DOOR => Some(theme::CHART_LOCKED),
        BOLTED => Some(theme::CHART_BOLTED),
        STAIR => Some(theme::CHART_STAIR),
        EXIT => Some(theme::CHART_GATE),
        _ => None,
    }
}
