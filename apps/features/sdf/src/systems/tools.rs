use crate::commands::{dress_edit, finish_edit, lay_capsule, placed, spawn_edit};
use crate::data::{Settings, Tool};
use crate::queries::shade;
use crate::resources::{Bite, SdfScene};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_camera::resources::ActiveCamera;
use nightshade_config::prelude::Config;
use nightshade_picking::queries::screen_ray;
use nightshade_platform::contract::input_capture::InputCapture;
use nightshade_platform::prelude::*;
use nightshade_platform::queries::input_capture::pointer_captured;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::*;
use nightshade_sdf::contract::field::sdf_field;
use nightshade_sdf::contract::render_data::{SdfEditInstance, SdfFrame};
use nightshade_sdf::contract::{SdfOperation, SdfShape};

pub fn select(input: Res<Input>, mut scene: ResMut<SdfScene>) {
    let picks = [
        (KeyCode::Digit1, Tool::Carve),
        (KeyCode::Digit2, Tool::Build),
        (KeyCode::Digit3, Tool::Draw),
        (KeyCode::Digit4, Tool::Hole),
    ];
    for (key, tool) in picks {
        if input.keyboard.just_pressed(key) {
            scene.tool = tool;
            let_go(&mut scene);
        }
    }
}

pub fn aim(
    input: Res<Input>,
    input_capture: Res<InputCapture>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    frame: Res<SdfFrame>,
    mut scene: ResMut<SdfScene>,
    storage: &mut Storage,
) {
    scene.aim = None;
    if !input.mouse.state.contains(MouseState::LEFT_CLICKED) || pointer_captured(&input_capture) {
        return;
    }
    let Some(ray) = screen_ray(
        &window,
        &viewport,
        &active_camera,
        storage.query_ref::<(&Camera, &GlobalTransform)>(),
        input.mouse.position,
    ) else {
        return;
    };
    let ignored: Vec<Entity> = match scene.tool {
        Tool::Hole => scene.hole.into_iter().collect(),
        Tool::Carve | Tool::Build => scene.bite.iter().map(|bite| bite.edit).collect(),
        Tool::Draw => scene.strokes.clone(),
    };
    let edits: Vec<SdfEditInstance> = frame
        .edits
        .iter()
        .filter(|edit| !ignored.contains(&edit.entity))
        .cloned()
        .collect();
    scene.eye = Some(ray.origin);
    scene.aim = march(&edits, ray.origin, ray.direction);
}

fn march(edits: &[SdfEditInstance], origin: Vec3, direction: Vec3) -> Option<Vec3> {
    let mut traveled = 0.0;
    for _ in 0..theme::AIM_STEPS {
        let point = origin + direction * traveled;
        let value = sdf_field(edits, &point, theme::AIM_TOUCH);
        if value < theme::AIM_TOUCH {
            return Some(point);
        }
        traveled += value;
        if traveled > theme::AIM_REACH {
            return None;
        }
    }
    None
}

fn let_go(scene: &mut SdfScene) {
    scene.bite = None;
    scene.strokes.clear();
    scene.stroke_end = None;
}

fn pressing(input: &Input, scene: &mut SdfScene, tool: Tool) -> Option<Vec3> {
    let held = input.mouse.state.contains(MouseState::LEFT_CLICKED);
    if !held || input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        let_go(scene);
    }
    if !held || scene.tool != tool {
        return None;
    }
    scene.aim
}

fn take_order(scene: &mut SdfScene) -> u32 {
    let order = scene.next_order.max(theme::FIRST_TOOL_ORDER);
    scene.next_order = order + 1;
    order
}

struct Reach {
    radius: f32,
    speed: f32,
    seed: f32,
    drift: f32,
}

fn reach_of(config: &Config<Settings>, radius: Settings, speed: Settings) -> Reach {
    Reach {
        radius: config.number(radius),
        speed: config.number(speed),
        seed: config.number(Settings::BiteSeed),
        drift: config.number(Settings::BiteDrift),
    }
}

fn grow(
    scene: &mut SdfScene,
    commands: &mut Commands,
    hit: Vec3,
    reach: &Reach,
    delta: f32,
) -> bool {
    let Some(bite) = scene.bite.as_mut() else {
        return false;
    };
    if nalgebra_glm::distance(&hit, &bite.center) > reach.radius * reach.drift {
        scene.bite = None;
        return false;
    }
    bite.radius = (bite.radius + reach.speed * delta).min(reach.radius);
    commands.set(
        bite.edit,
        SdfShape::Sphere {
            radius: bite.radius,
        },
    );
    if bite.radius >= reach.radius {
        scene.bite = None;
    }
    true
}

fn bite_at(
    scene: &mut SdfScene,
    commands: &mut Commands,
    hit: Vec3,
    reach: &Reach,
    name: &str,
) -> Entity {
    let radius = reach.radius * reach.seed;
    let edit = spawn_edit(commands, name, placed(hit), SdfShape::Sphere { radius });
    scene.bite = Some(Bite {
        edit,
        center: hit,
        radius,
    });
    edit
}

pub fn carve(
    input: Res<Input>,
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut scene: ResMut<SdfScene>,
    mut commands: Commands,
) {
    let Some(hit) = pressing(&input, &mut scene, Tool::Carve) else {
        return;
    };
    let reach = reach_of(&config, Settings::CarveRadius, Settings::CarveSpeed);
    if grow(&mut scene, &mut commands, hit, &reach, time.delta_time) {
        return;
    }
    let order = take_order(&mut scene);
    let cut = bite_at(&mut scene, &mut commands, hit, &reach, "Carve");
    dress_edit(
        &mut commands,
        cut,
        SdfOperation::Subtract,
        config.number(Settings::CarveBlend),
        shade(&config, Settings::GroundTint),
        order,
    );
}

pub fn build(
    input: Res<Input>,
    time: Res<Time>,
    config: Res<Config<Settings>>,
    mut scene: ResMut<SdfScene>,
    mut commands: Commands,
) {
    let Some(hit) = pressing(&input, &mut scene, Tool::Build) else {
        return;
    };
    let reach = reach_of(&config, Settings::BuildRadius, Settings::BuildSpeed);
    if grow(&mut scene, &mut commands, hit, &reach, time.delta_time) {
        return;
    }
    let Some(eye) = scene.eye else {
        return;
    };
    if nalgebra_glm::distance(&eye, &hit) < reach.radius + config.number(Settings::BuildClearance) {
        return;
    }
    let order = take_order(&mut scene);
    let lump = bite_at(&mut scene, &mut commands, hit, &reach, "Build");
    dress_edit(
        &mut commands,
        lump,
        SdfOperation::Union,
        config.number(Settings::BuildBlend),
        shade(&config, Settings::BuildTint),
        order,
    );
    finish_edit(
        &mut commands,
        lump,
        config.number(Settings::BuildRoughness),
        0.0,
    );
}

fn next_segment(scene: &mut SdfScene, hit: Vec3, spacing: f32, jump: f32) -> Option<(Vec3, Vec3)> {
    let Some(last) = scene.stroke_end else {
        scene.stroke_end = Some(hit);
        return Some((hit, hit));
    };
    let apart = nalgebra_glm::distance(&last, &hit);
    if apart < spacing {
        return None;
    }
    scene.stroke_end = Some(hit);
    if apart > jump {
        return Some((hit, hit));
    }
    Some((last, hit))
}

pub fn draw(
    input: Res<Input>,
    config: Res<Config<Settings>>,
    mut scene: ResMut<SdfScene>,
    mut commands: Commands,
) {
    let Some(hit) = pressing(&input, &mut scene, Tool::Draw) else {
        return;
    };
    let Some((from, to)) = next_segment(
        &mut scene,
        hit,
        config.number(Settings::StrokeSpacing),
        config.number(Settings::StrokeJump),
    ) else {
        return;
    };
    let order = take_order(&mut scene);
    let stroke = lay_capsule(
        &mut commands,
        "Stroke",
        from,
        to,
        config.number(Settings::StrokeRadius),
    );
    scene.strokes.push(stroke);
    dress_edit(
        &mut commands,
        stroke,
        SdfOperation::Union,
        config.number(Settings::StrokeBlend),
        shade(&config, Settings::StrokeTint),
        order,
    );
    finish_edit(
        &mut commands,
        stroke,
        config.number(Settings::StrokeRoughness),
        0.0,
    );
}

pub fn drag(
    input: Res<Input>,
    config: Res<Config<Settings>>,
    mut scene: ResMut<SdfScene>,
    mut holes: Query<(&mut LocalTransform,)>,
) {
    let Some(hit) = pressing(&input, &mut scene, Tool::Hole) else {
        return;
    };
    let Some(hole) = scene.hole else {
        return;
    };
    let depth = config.number(Settings::HoleDepth);
    holes.for_entity(hole, |(transform,)| {
        transform.translation = Vec3::new(hit.x, hit.y - depth * 0.5 + 1.0, hit.z);
    });
}
