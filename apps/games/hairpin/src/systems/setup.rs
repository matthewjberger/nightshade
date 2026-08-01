use crate::commands::set_screen;
use crate::data::*;
use crate::resources::{HairpinResources, Screen, Ui};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_gltf::Loading;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_load::{Source, load};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::material::Material;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::commands::add_menu_panel;
use nightshade_ui_menu::data::MenuStyle;
use std::collections::HashMap;

pub fn build_overlays(mut resources: ResMut<HairpinResources>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    let mut ui = Ui::default();

    tree.in_parent(root, |tree| {
        ui.hud = Some(readout(tree));

        let title = add_menu_panel(tree, &menu_style(), MENU_WIDTH, MENU_HEIGHT);
        ui.title = Some(title);
        tree.in_parent(title, |tree| {
            row(tree, "HAIRPIN", TITLE_SIZE);
            row(tree, "Hold the drift. Beat your best lap.", LABEL_SIZE);
            ui.start_button = Some(tree.add_button("Drive"));
            ui.exit_button = Some(tree.add_button("Exit"));
        });

        let pause = add_menu_panel(tree, &menu_style(), MENU_WIDTH, MENU_HEIGHT);
        ui.pause = Some(pause);
        tree.in_parent(pause, |tree| {
            row(tree, "Paused", HEADING_SIZE);
            ui.resume_button = Some(tree.add_button("Resume"));
            ui.restart_button = Some(tree.add_button("Restart"));
            ui.quit_button = Some(tree.add_button("Quit to title"));
        });

        for (index, (area, label)) in PAD_AREAS.iter().zip(PAD_LABELS).enumerate() {
            ui.pads[index] = Some(pad(tree, *area, label));
        }
        ui.pause_pad = Some(pad(tree, PAUSE_AREA, PAUSE_LABEL));
    });

    tree.finish();
    resources.ui = ui;
}

pub fn initialize(
    mut resources: ResMut<HairpinResources>,
    mut settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut registry: ResMut<MaterialRegistry>,
    mut loading: ResMut<Loading>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;
    spawn_sun(&mut commands);
    spawn_field(&mut registry, &mut commands);

    let eye = CAR_START - LINE_FORWARD * CAMERA_BACK + Vec3::y() * CAMERA_HEIGHT;
    let camera = spawn_camera(&mut commands, eye, "Chase Camera".to_string());
    active_camera.0 = Some(camera);
    resources.track.camera = Some(camera);

    resources.models.prefabs = vec![None; MODEL_COUNT];
    resources.models.handles = MODEL_FILES
        .iter()
        .map(|gltf| load(&mut loading, model_source(gltf)))
        .collect();

    set_screen(&mut resources, &mut dirty, &mut commands, Screen::Title);
}

fn spawn_field(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let entity = spawn_mesh_at(commands, "Plane", FIELD_CENTER, FIELD_SIZE);
    let material = insert_material(
        registry,
        FIELD_NAME,
        Material {
            base_color: FIELD_COLOR,
            roughness: 0.95,
            ..Default::default()
        },
    );
    commands.set(entity, MaterialRef::new(material));
}

fn model_source(gltf: &[u8]) -> Source {
    Source::Parts {
        head: gltf.to_vec(),
        files: HashMap::from([(COLORMAP_NAME.to_string(), COLORMAP.to_vec())]),
    }
}

fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BACKGROUND,
        border: PANEL_BORDER,
        corner_radius: 10.0,
        border_width: 1.0,
        padding: 14.0,
        spacing: 10.0,
        visible: false,
        ..Default::default()
    }
}

fn row(tree: &mut UiTreeBuilder, text: &str, size: f32) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + ROW_HEIGHT - LABEL_SIZE)))
        .with_text(text, size)
        .color_raw::<UiBase>(TEXT_COLOR);
}

fn pad(tree: &mut UiTreeBuilder, area: [f32; 4], label: &str) -> Entity {
    let entity = tree
        .add_node()
        .window(
            Rl(vec2(area[0] * 100.0, area[1] * 100.0)),
            Rl(vec2(area[2] * 100.0, area[3] * 100.0)),
            Anchor::TopLeft,
        )
        .with_rect(PAD_RADIUS, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PAD_BACKGROUND)
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .without_pointer_events()
        .with_visible(false)
        .entity();
    tree.in_parent(entity, |tree| {
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 100.0)))
            .with_text(label, HEADING_SIZE)
            .text_center()
            .with_text_outline(OUTLINE_COLOR, 1.0)
            .color_raw::<UiBase>(TEXT_COLOR)
            .without_pointer_events();
    });
    entity
}

fn readout(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .window(
            Rl(vec2(0.0, 0.0)) + Ab(vec2(HUD_MARGIN, HUD_MARGIN)),
            Ab(vec2(HUD_WIDTH, HUD_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_text("", LABEL_SIZE)
        .text_left()
        .with_text_outline(OUTLINE_COLOR, 1.0)
        .color_raw::<UiBase>(TEXT_COLOR)
        .without_pointer_events()
        .with_visible(false)
        .entity()
}
