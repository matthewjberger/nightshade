use crate::resources::Showroom;
use crate::theme;
use blocker_figure::commands::{dress_the_body, show_the_tree};
use blocker_figure::components::{Eye, Gait, Gaze, Look, Pace, Regard};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_hud::theme as hud;
use blocker_screen::data::screen::Screen;
use blocker_screens::theme as screens;
use nalgebra_glm::{Quat, Vec3};
use nightshade::ecs::HierarchyIndex;
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::prelude::spawn_mesh_at;
use nightshade_platform::prelude::{Input, Time, Window};
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};
use nightshade_state::prelude::State;

pub fn stand_the_showroom(
    window: Res<Window>,
    input: Res<Input>,
    time: Res<Time>,
    state: Res<State<Screen>>,
    hierarchy: Res<HierarchyIndex>,
    active: Res<ActiveCamera>,
    hero: Res<Hero>,
    mut showroom: ResMut<Showroom>,
    mut commands: Commands,
    mut seen: Query<(&Visibility,)>,
    mut placed: Query<(&GlobalTransform,)>,
    mut heroes: Query<(&Eye, &Gaze, &Look)>,
) {
    let Some((eye, gaze, look)) = read_the_hero(&hero, &mut heroes, |(eye, gaze, look)| {
        (eye.0, gaze.0, look.0)
    }) else {
        return;
    };
    let mut watching = eye;
    if let Some(camera) = active.0 {
        placed.for_entity(camera, |(transform,)| watching = transform.translation());
    }
    let standing = state.current == Screen::Character;
    let body = match showroom.body {
        Some(body) => body,
        None => {
            if !standing {
                return;
            }
            let body = commands.spawn((
                Name("Showroom".to_string()),
                LocalTransform::default(),
                GlobalTransform::default(),
                Visibility { visible: false },
            ));
            dress_the_body(
                &mut commands,
                body,
                look,
                blocker_figure::data::rig::FIGURE_SCALE,
            );
            commands.set(body, Pace(theme::SHOWROOM_IDLE_PACE));
            showroom.body = Some(body);
            body
        }
    };
    let wall = match showroom.wall {
        Some(wall) => wall,
        None => {
            let wall = spawn_mesh_at(
                &mut commands,
                "Cube",
                Vec3::zeros(),
                Vec3::new(1.0, 1.0, 1.0),
            );
            set_mesh_material(&mut commands, wall, theme::SHOWROOM_WALL_MATERIAL);
            commands.set(wall, Visibility { visible: false });
            showroom.wall = Some(wall);
            wall
        }
    };
    if standing {
        let stage = stage_the_figure(&window, watching, gaze);
        let clock = time.uptime_seconds as f32;
        let sway = (clock * theme::SHOWROOM_SWAY_RATE).sin() * theme::SHOWROOM_SWAY;
        let breath = (clock * theme::SHOWROOM_BREATH_RATE).sin() * theme::SHOWROOM_BREATH;
        commands.set(
            body,
            LocalTransform {
                translation: stage.place + stage.up * (breath * stage.scale),
                rotation: stage.facing * nalgebra_glm::quat_angle_axis(sway, &Vec3::y()),
                scale: Vec3::new(stage.scale, stage.scale, stage.scale),
            },
        );
        commands.set(
            wall,
            LocalTransform {
                translation: stage.wall,
                rotation: stage.facing,
                scale: stage.wall_size,
            },
        );
        commands.set(
            body,
            Gait(clock * theme::SHOWROOM_IDLE_RATE % std::f32::consts::TAU),
        );
        let moved = input.mouse.position_delta.magnitude() > 0.0;
        showroom.still = if moved {
            0.0
        } else {
            showroom.still + time.delta_time
        };
        if showroom.still < theme::SHOWROOM_GAZE_HOLD {
            showroom.gazing = true;
            commands.set(body, Regard(pointed_at(&window, &input, watching, gaze)));
        } else if showroom.gazing {
            showroom.gazing = false;
            commands.remove::<(Regard,)>(body);
        }
    }
    show_the_tree(&hierarchy, &mut commands, &mut seen, body, standing);
    show_the_tree(&hierarchy, &mut commands, &mut seen, wall, standing);
}

struct Stage {
    place: Vec3,
    up: Vec3,
    facing: Quat,
    scale: f32,
    wall: Vec3,
    wall_size: Vec3,
}

struct Frame {
    wide: f32,
    tall: f32,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    half_tall: f32,
    half_wide: f32,
}

fn frame_the_view(window: &Window, gaze: Vec3) -> Frame {
    let (wide, tall) = window
        .cached_viewport_size
        .map(|(wide, tall)| (wide as f32, tall as f32))
        .filter(|(wide, tall)| *wide > 1.0 && *tall > 1.0)
        .unwrap_or((1920.0, 1080.0));
    let forward = if gaze.magnitude() > 0.0001 {
        gaze.normalize()
    } else {
        Vec3::z()
    };
    let right = forward.cross(&Vec3::y());
    let right = if right.magnitude() > 0.0001 {
        right.normalize()
    } else {
        Vec3::x()
    };
    let up = right.cross(&forward).normalize();
    let half_tall = theme::SHOWROOM_DISTANCE
        * (blocker_screen::theme::CAMERA_FOV_DEGREES.to_radians() * 0.5).tan();
    let half_wide = half_tall * wide / tall;
    Frame {
        wide,
        tall,
        forward,
        right,
        up,
        half_tall,
        half_wide,
    }
}

fn pointed_at(window: &Window, input: &Input, watching: Vec3, gaze: Vec3) -> Vec3 {
    let frame = frame_the_view(window, gaze);
    let across = (input.mouse.position.x / frame.wide) * 2.0 - 1.0;
    let lift = 1.0 - (input.mouse.position.y / frame.tall) * 2.0;
    watching
        + frame.forward * theme::SHOWROOM_DISTANCE
        + frame.right * (across * frame.half_wide)
        + frame.up * (lift * frame.half_tall)
}

fn stage_the_figure(window: &Window, watching: Vec3, gaze: Vec3) -> Stage {
    let Frame {
        wide,
        tall,
        forward,
        right,
        up,
        half_tall,
        half_wide,
    } = frame_the_view(window, gaze);
    let column = screens::CHARACTER_HEIGHT - hud::TITLE_HEIGHT - hud::FOOTER_HEIGHT;
    let stand_tall = column - 20.0;
    let panel_top = tall * 0.5 - screens::CHARACTER_HEIGHT * 0.5;
    let panel_left = wide * 0.5 - screens::CHARACTER_WIDTH * 0.5;
    let stand_top = panel_top + hud::TITLE_HEIGHT + hud::BODY_PADDING;
    let center_x = panel_left + hud::BODY_PADDING + screens::CHARACTER_STAND_WIDTH * 0.5;
    let stand_center_y = stand_top + stand_tall * 0.5;
    let figure_center_y = stand_top + (stand_tall - screens::CHARACTER_CAPTION) * 0.5;
    let at = |x: f32, y: f32, distance: f32| {
        let across = (x / wide) * 2.0 - 1.0;
        let lift = 1.0 - (y / tall) * 2.0;
        let reach = distance / theme::SHOWROOM_DISTANCE;
        watching
            + forward * distance
            + right * (across * half_wide * reach)
            + up * (lift * half_tall * reach)
    };
    let wanted = (stand_tall - screens::CHARACTER_CAPTION - 40.0) / tall
        * 2.0
        * half_tall
        * theme::SHOWROOM_FILL;
    let scale = wanted / (theme::SHOWROOM_FIGURE_HEIGHT * blocker_figure::data::rig::FIGURE_SCALE);
    let middle = at(center_x, figure_center_y, theme::SHOWROOM_DISTANCE);
    let place = middle - up * (wanted * 0.5);
    let facing =
        nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[right, up, -forward]));
    let wall_distance = theme::SHOWROOM_DISTANCE + theme::SHOWROOM_WALL_BEHIND;
    let reach = wall_distance / theme::SHOWROOM_DISTANCE;
    let wall = at(center_x, stand_center_y, wall_distance);
    let wall_size = Vec3::new(
        screens::CHARACTER_STAND_WIDTH / tall * 2.0 * half_tall * reach,
        stand_tall / tall * 2.0 * half_tall * reach,
        0.02,
    );
    Stage {
        place,
        up,
        facing,
        scale,
        wall,
        wall_size,
    }
}
