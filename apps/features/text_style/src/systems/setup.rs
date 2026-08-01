use crate::components::{BaseScale, FollowsCamera};
use crate::resources::TextStyleResources;
use crate::systems::panel;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_text::commands::{
    spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties,
};
use nightshade_text::contract::{TextAlignment, TextProperties, TextState, VerticalAlignment};

pub fn initialize(
    resources: Res<TextStyleResources>,
    mut debug_draw: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    debug_draw.show_grid = true;
    settings.atmosphere = Atmosphere::Sky;

    spawn_sun(&mut commands);

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        theme::CAMERA_TARGET,
        theme::CAMERA_DISTANCE,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        "Text style camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let style = resources.style;
    let properties = |font_size: f32| TextProperties {
        font_size,
        color: theme::TEXT_COLOR,
        alignment: TextAlignment::Center,
        vertical_alignment: VerticalAlignment::Middle,
        outline_width: style.outline_width,
        outline_color: style.outline_color,
        smoothing: style.smoothing,
        ..Default::default()
    };

    let title = spawn_3d_billboard_text_with_properties(
        &mut text_state,
        &mut commands,
        "Nightshade",
        Vec3::new(0.0, 2.2, 0.0),
        properties(96.0),
    );
    commands.set(title, BaseScale(1.0));
    commands.set(title, FollowsCamera(true));

    let standing = spawn_3d_text_with_properties(
        &mut text_state,
        &mut commands,
        "drawn from a distance field",
        Vec3::new(0.0, 1.2, 0.0),
        properties(40.0),
    );
    commands.set(standing, BaseScale(1.0));
    commands.set(standing, FollowsCamera(false));

    let enlarged = spawn_3d_text_with_properties(
        &mut text_state,
        &mut commands,
        "16 px, scaled by three",
        Vec3::new(0.0, 0.4, 0.0),
        properties(16.0),
    );
    commands.set(enlarged, BaseScale(3.0));
    commands.set(enlarged, FollowsCamera(false));
}

pub fn build_panel(
    mut resources: ResMut<TextStyleResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let style = resources.style;
    resources.handles = panel::build(build.context(), style);
}
