use crate::components::{
    Grounded, Motion, Pitch, Riding, Submerged, Velocity, WatchSpeed, Yaw, Zoom, ZoomWanted,
};
use crate::data::ask::Ask;
use crate::data::settings::Settings;
use crate::queries::player_box;
use crate::theme;
use blocker_blocks::data::items::Item;
use blocker_blocks::data::tools::ToolKind;
use blocker_blocks::queries::{box_in_liquid, is_solid_at, slide_box};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use nalgebra_glm::{IVec3, Vec2, Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_config::prelude::Config;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

type Walking<'world, 'item> = Query<
    'world,
    (
        &'item mut LocalTransform,
        &'item mut Velocity,
        &'item mut Yaw,
        &'item mut Pitch,
        &'item mut Motion,
        &'item mut Grounded,
        &'item mut Submerged,
        &'item mut WatchSpeed,
    ),
>;

type Looking<'world, 'item> = Query<
    'world,
    (
        &'item mut Eye,
        &'item mut Gaze,
        &'item mut Zoom,
        &'item mut ZoomWanted,
        Option<&'item Riding>,
    ),
>;

type Viewpoints<'world, 'item> = Query<'world, (&'item Camera, &'item mut LocalTransform)>;

nightshade::ecs::query_bundle! {
    pub struct Piloted<'world, 'item> {
        pub walking: Walking<'world, 'item>,
        pub looking: Looking<'world, 'item>,
        pub viewpoints: Viewpoints<'world, 'item>,
    }
}

pub fn control(
    world: Res<VoxelWorld>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    inventory: Res<Inventory>,
    time: Res<Time>,
    window: Res<Window>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut piloted: Piloted<'_, '_>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some((
        mut position,
        mut velocity,
        mut yaw,
        mut pitch,
        mut motion,
        mut grounded,
        mut submerged,
        mut watch_speed,
    )) = read_the_hero(
        &hero,
        &mut piloted.walking,
        |(transform, carried, turn, tilt, mode, down, wet, watching)| {
            (
                transform.translation,
                carried.0,
                turn.0,
                tilt.0,
                *mode,
                down.0,
                wet.0,
                watching.0,
            )
        },
    )
    else {
        return;
    };
    let Some((mut zoom, mut zoom_wanted, riding)) =
        read_the_hero(&hero, &mut piloted.looking, |(_, _, near, wanted, boat)| {
            (near.0, wanted.0, boat.is_some())
        })
    else {
        return;
    };

    let delta_time = time.delta_time.min(0.05);
    let mouse_sensitivity = config.number(Settings::MouseSensitivity);
    let pad_look_rate = config.number(Settings::PadLookRate);
    let zoom_smooth = config.number(Settings::ZoomSmooth);
    let fly_speed = config.number(Settings::FlySpeed);
    let swim_speed = config.number(Settings::SwimSpeed);
    let water_gravity = config.number(Settings::WaterGravity);
    let crouch_speed = config.number(Settings::CrouchSpeed);
    let sprint_speed = config.number(Settings::SprintSpeed);
    let walk_speed = config.number(Settings::WalkSpeed);
    let gravity = config.number(Settings::Gravity);
    let jump_speed = config.number(Settings::JumpSpeed);
    let glide_fall = config.number(Settings::GlideFall);
    let glide_speed = config.number(Settings::GlideSpeed);
    let locked = window.cursor_locked;
    let glider = matches!(
        selected_stack(&inventory).map(|stack| stack.item),
        Some(Item::Tool(ToolKind::Glider))
    );

    if locked {
        let look = input.mouse.raw_mouse_delta * mouse_sensitivity;
        yaw -= look.x;
        pitch = (pitch - look.y).clamp(-1.55, 1.55);
        let turn = actions.axis2(Ask::Look) * pad_look_rate * delta_time;
        yaw -= turn.x;
        pitch = (pitch + turn.y).clamp(-1.55, 1.55);
    }

    if riding {
        let rotation = quat_angle_axis(yaw, &Vec3::y()) * quat_angle_axis(pitch, &Vec3::x());
        let eye = position + Vec3::new(0.0, theme::EYE_HEIGHT * 0.8, 0.0);
        let gaze = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
        aim_the_camera(&mut piloted.viewpoints, eye, gaze, zoom, rotation, &world);
        write_the_body(
            &mut piloted,
            body,
            (
                position,
                velocity,
                yaw,
                pitch,
                motion,
                grounded,
                submerged,
                watch_speed,
            ),
            (eye, gaze, zoom, zoom_wanted),
        );
        return;
    }
    if locked
        && actions.just_pressed(Ask::Fly)
        && let Some(next) = motion.process_event(crate::data::motion::Event::Fly)
    {
        motion = next;
        velocity.y = 0.0;
    }
    if locked
        && actions.just_pressed(Ask::Watch)
        && let Some(next) = motion.process_event(crate::data::motion::Event::Watch)
    {
        motion = next;
        velocity = Vec3::zeros();
        watch_speed = theme::WATCH_SPEED;
    }
    let wheel = input.mouse.wheel_delta.y;
    if locked && motion != Motion::Watch && wheel.abs() > 0.01 {
        zoom_wanted = (zoom_wanted - wheel.signum() * theme::ZOOM_STEP)
            .clamp(theme::ZOOM_MIN, theme::ZOOM_MAX);
    }
    zoom += (zoom_wanted - zoom) * (zoom_smooth * delta_time).clamp(0.0, 1.0);

    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);
    let steer = if locked {
        actions.axis2(Ask::Move)
    } else {
        Vec2::zeros()
    };
    let mut wish = right * steer.x + forward * steer.y;
    if wish.norm_squared() > 0.0 {
        wish = wish.normalize();
    }
    let jump = locked && actions.pressed(Ask::Jump);
    let sink = locked && actions.pressed(Ask::Sink);
    let sprint = locked && actions.pressed(Ask::Sprint);
    let crouch = sink && grounded && motion != Motion::Fly && !submerged;

    let (min, max) = player_box(position);
    submerged = box_in_liquid(&world, min, max + Vec3::new(0.0, -0.9, 0.0))
        == Some(blocker_blocks::data::blocks::WATER);

    if motion == Motion::Watch {
        let wheel = input.mouse.wheel_delta.y;
        if locked && wheel.abs() > 0.01 {
            watch_speed = (watch_speed + wheel.signum() * theme::WATCH_SPEED_STEP)
                .clamp(theme::WATCH_SPEED_MIN, theme::WATCH_SPEED_MAX);
        }
        let speed = if sprint {
            watch_speed * 2.5
        } else {
            watch_speed
        };
        velocity = wish * speed;
        if jump {
            velocity.y += speed;
        }
        if sink {
            velocity.y -= speed;
        }
        position += velocity * delta_time;
        position.y = position.y.clamp(
            nightshade_voxels::data::FLOOR_Y as f32 + 1.0,
            nightshade_voxels::data::WORLD_Y as f32 - 1.0,
        );
        grounded = false;
        let rotation = quat_angle_axis(yaw, &Vec3::y()) * quat_angle_axis(pitch, &Vec3::x());
        let eye = position + Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
        let gaze = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
        aim_the_camera(&mut piloted.viewpoints, eye, gaze, zoom, rotation, &world);
        write_the_body(
            &mut piloted,
            body,
            (
                position,
                velocity,
                yaw,
                pitch,
                motion,
                grounded,
                submerged,
                watch_speed,
            ),
            (eye, gaze, zoom, zoom_wanted),
        );
        return;
    }

    if motion == Motion::Fly {
        let speed = if sprint { fly_speed * 2.0 } else { fly_speed };
        velocity = wish * speed;
        if jump {
            velocity.y += speed;
        }
        if sink {
            velocity.y -= speed;
        }
    } else if submerged {
        velocity.x = wish.x * swim_speed;
        velocity.z = wish.z * swim_speed;
        velocity.y -= water_gravity * delta_time;
        velocity.y = velocity.y.max(-3.0);
        if jump {
            velocity.y = 3.2;
        }
    } else {
        let speed = if crouch {
            crouch_speed
        } else if sprint {
            sprint_speed
        } else {
            walk_speed
        };
        velocity.x = wish.x * speed;
        velocity.z = wish.z * speed;
        velocity.y -= gravity * delta_time;
        if jump && grounded {
            velocity.y = jump_speed;
        }
        let wants_glide = glider && !grounded && !submerged && velocity.y < 0.0 && !sink;
        if wants_glide
            && motion == Motion::Walk
            && let Some(next) = motion.process_event(crate::data::motion::Event::Soar)
        {
            motion = next;
        } else if !wants_glide
            && motion == Motion::Glide
            && let Some(next) = motion.process_event(crate::data::motion::Event::Land)
        {
            motion = next;
        }
        if motion == Motion::Glide {
            velocity.y = velocity.y.max(-glide_fall);
            velocity.x = forward.x * glide_speed;
            velocity.z = forward.z * glide_speed;
        }
        if crouch {
            let ahead_x = position + Vec3::new(velocity.x * delta_time, 0.0, 0.0);
            if !supported(&world, ahead_x) {
                velocity.x = 0.0;
            }
            let ahead_z = position + Vec3::new(0.0, 0.0, velocity.z * delta_time);
            if !supported(&world, ahead_z) {
                velocity.z = 0.0;
            }
        }
    }

    let low = Vec3::new(-theme::PLAYER_HALF_WIDTH, 0.0, -theme::PLAYER_HALF_WIDTH);
    let high = Vec3::new(
        theme::PLAYER_HALF_WIDTH,
        theme::PLAYER_HEIGHT,
        theme::PLAYER_HALF_WIDTH,
    );
    let slide = slide_box(&world, position, low, high, velocity * delta_time);
    for axis in 0..3 {
        if slide.blocked[axis] {
            velocity[axis] = 0.0;
        }
    }
    position = slide.position;
    if position.y < 1.0 {
        position.y = nightshade_voxels::data::WORLD_Y as f32;
    }
    grounded = slide.on_ground;

    let rotation = quat_angle_axis(yaw, &Vec3::y()) * quat_angle_axis(pitch, &Vec3::x());
    let eye_height = if crouch {
        theme::CROUCH_EYE_HEIGHT
    } else {
        theme::EYE_HEIGHT
    };
    let eye = position + Vec3::new(0.0, eye_height, 0.0);
    let gaze = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    aim_the_camera(&mut piloted.viewpoints, eye, gaze, zoom, rotation, &world);
    write_the_body(
        &mut piloted,
        body,
        (
            position,
            velocity,
            yaw,
            pitch,
            motion,
            grounded,
            submerged,
            watch_speed,
        ),
        (eye, gaze, zoom, zoom_wanted),
    );
}

fn write_the_body(
    piloted: &mut Piloted<'_, '_>,
    body: Entity,
    (position, velocity, yaw, pitch, motion, grounded, submerged, watch_speed): (
        Vec3,
        Vec3,
        f32,
        f32,
        Motion,
        bool,
        bool,
        f32,
    ),
    (eye, gaze, zoom, zoom_wanted): (Vec3, Vec3, f32, f32),
) {
    piloted.walking.for_entity(
        body,
        |(transform, carried, turn, tilt, mode, down, wet, watching)| {
            transform.translation = position;
            transform.rotation = quat_angle_axis(yaw + std::f32::consts::PI, &Vec3::y());
            carried.0 = velocity;
            turn.0 = yaw;
            tilt.0 = pitch;
            *mode = motion;
            down.0 = grounded;
            wet.0 = submerged;
            watching.0 = watch_speed;
        },
    );
    piloted
        .looking
        .for_entity(body, |(sight, ahead, near, wanted, _)| {
            sight.0 = eye;
            ahead.0 = gaze;
            near.0 = zoom;
            wanted.0 = zoom_wanted;
        });
}

fn aim_the_camera(
    viewpoints: &mut Viewpoints<'_, '_>,
    eye: Vec3,
    gaze: Vec3,
    zoom: f32,
    rotation: nalgebra_glm::Quat,
    world: &VoxelWorld,
) {
    let place = eye + camera_offset(world, eye, gaze, zoom);
    viewpoints.for_each_mut(|_, (_, transform)| {
        transform.translation = place;
        transform.rotation = rotation;
    });
}

fn supported(world: &VoxelWorld, position: Vec3) -> bool {
    let level = (position.y - 0.1).floor() as i32;
    for offset_x in [-theme::PLAYER_HALF_WIDTH, theme::PLAYER_HALF_WIDTH] {
        for offset_z in [-theme::PLAYER_HALF_WIDTH, theme::PLAYER_HALF_WIDTH] {
            let block = IVec3::new(
                (position.x + offset_x).floor() as i32,
                level,
                (position.z + offset_z).floor() as i32,
            );
            if is_solid_at(world, block) {
                return true;
            }
        }
    }
    false
}

fn camera_offset(world: &VoxelWorld, eye: Vec3, forward: Vec3, zoom: f32) -> Vec3 {
    let back = -forward;
    if zoom <= 0.01 {
        return Vec3::zeros();
    }
    let mut distance = zoom;
    let mut travelled = theme::CAMERA_COLLIDE_STEP;
    while travelled <= zoom {
        let point = eye + back * travelled;
        let block = IVec3::new(
            point.x.floor() as i32,
            point.y.floor() as i32,
            point.z.floor() as i32,
        );
        if is_solid_at(world, block) {
            distance = (travelled - theme::CAMERA_COLLIDE_MARGIN).max(0.0);
            break;
        }
        travelled += theme::CAMERA_COLLIDE_STEP;
    }
    back * distance
}
