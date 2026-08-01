use crate::commands::{attach_surface, lock_cursor};
use crate::components::SurfaceKind;
use crate::data::level::{HEIGHT, TILE, WIDTH};
use crate::data::rules::SATCHEL_SLOTS;
use crate::data::theme::*;
use crate::resources::{AshfallResources, Screen, UiHandles, WorldPhase};
use crate::systems::{screens, textures};
use crate::tags::{MenuCamera, Moon};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_platform::prelude::*;
use nightshade_text::prelude::{TextAlignment, VerticalAlignment};
use nightshade_ui::components::UiNodeShadow;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::ThemeState;
use nightshade_ui_menu::prelude::*;

pub fn initialize(
    mut resources: ResMut<AshfallResources>,
    mut settings: ResMut<RenderSettings>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
    mut textures_load: nightshade_graphics::resources::loading::TextureLoad,
) {
    apply_look(&mut settings);
    textures::load_textures(&mut textures_load);
    spawn_moon(&mut commands);
    spawn_ground(&mut commands);
    spawn_menu_camera(&mut commands);
    lock_cursor(&mut window, false);
    resources.screen = Screen::Title;
    resources.run.rng = fastrand::Rng::with_seed(0x5EED_1234);
    screens::refresh(&resources, &mut dirty, &mut commands);
}

fn apply_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.show_sky = false;
    settings.clear_color = [0.0, 0.0, 0.0, 1.0];
    settings.ibl_blend_factor = 0.0;
}

fn spawn_moon(commands: &mut Commands) {
    let moon = spawn_sun_without_shadows(commands);
    commands.add_tag_type::<Moon>(moon);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(moon) {
            light.color = MOON_COLOR;
            light.intensity = MOON_INTENSITY;
        }
    });
}

fn spawn_ground(commands: &mut Commands) {
    let half_width = WIDTH as f32 * TILE * 0.5 + 2.0;
    let half_depth = HEIGHT as f32 * TILE * 0.5 + 2.0;
    let center = Vec3::new(
        (WIDTH as f32 - 1.0) * TILE * 0.5,
        0.0,
        (HEIGHT as f32 - 1.0) * TILE * 0.5,
    );
    let entity = spawn_mesh_at(
        commands,
        "Plane",
        center,
        Vec3::new(half_width, 1.0, half_depth),
    );
    attach_surface(commands, entity, SurfaceKind::Street, WorldPhase::Fog);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(center.x, -0.05, center.z),
    );
    commands.set(
        entity,
        ColliderComponent::new_cuboid(half_width, 0.05, half_depth)
            .with_friction(0.9)
            .with_restitution(0.0),
    );
}

fn spawn_menu_camera(commands: &mut Commands) {
    let camera = spawn_camera(commands, MENU_CAMERA_EYE, "Menu Camera".to_string());
    commands.add_tag_type::<MenuCamera>(camera);
    commands.queue(move |storage, resources| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
            transform.rotation = camera_look_rotation(MENU_CAMERA_EYE, MENU_CAMERA_TARGET, 0.0);
        }
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
}

pub fn build_overlays(
    mut resources: ResMut<AshfallResources>,
    mut flash: ResMut<nightshade_ui_flash::FlashHandles>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    dress_the_interface(build.theme());
    let mut wash = nightshade_ui_flash::FlashHandles::default();
    let handles = {
        let mut tree = UiTreeBuilder::new(build.context());
        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();

        let handles = tree.in_parent(root, |tree| {
            let mut handles = UiHandles::default();
            wash = nightshade_ui_flash::commands::add_flash(tree);
            build_hud(tree, &mut handles);
            build_title(tree, &mut handles);
            build_pause(tree, &mut handles);
            build_note(tree, &mut handles);
            build_talk(tree, &mut handles);
            build_inventory(tree, &mut handles);
            build_town_map(tree, &mut handles);
            build_inspect(tree, &mut handles);
            build_dead(tree, &mut handles);
            build_escaped(tree, &mut handles);
            handles
        });

        tree.finish();
        handles
    };
    resources.ui = handles;
    *flash = wash;
}

fn dress_the_interface(state: &mut ThemeState) {
    let theme = &mut state.current_theme;
    theme.name = "Ashfall".to_string();
    theme.dark_mode = true;
    theme.text_color = TEXT_PRIMARY;
    theme.text_color_disabled = TEXT_DIM;
    theme.text_color_accent = TEXT_RUST;
    theme.background_color = BUTTON_BG;
    theme.background_color_hovered = BUTTON_HOVER;
    theme.background_color_active = BUTTON_PRESSED;
    theme.panel_color = PANEL_BG;
    theme.panel_header_color = PANEL_BG;
    theme.border_color = PANEL_RULE;
    theme.border_color_focused = PANEL_BORDER;
    theme.accent_color = TEXT_RUST;
    theme.accent_color_hovered = BUTTON_HOVER;
    theme.accent_color_active = BUTTON_PRESSED;
    theme.selection_color = BUTTON_PRESSED;
    theme.input_background_color = BUTTON_BG;
    theme.input_background_focused = BUTTON_HOVER;
    theme.border_width = 1.0;
    theme.corner_radius = 0.0;
    theme.padding = 12.0;
    theme.spacing = 10.0;
    theme.font_size = LABEL_SIZE;
    theme.body_font_size = LABEL_SIZE;
    theme.heading_font_size = HEADING_SIZE;
    theme.caption_font_size = LABEL_SIZE - 2.0;
    theme.button_height = BUTTON_HEIGHT;
    theme.panel_shadow = UiNodeShadow::default();
    theme.button_shadow = UiNodeShadow::default();
    theme.button_shadow_hover = UiNodeShadow::default();
    theme.button_shadow_pressed = UiNodeShadow::default();
    theme.modal_shadow = UiNodeShadow::default();
    theme.focus_glow = None;
    state.presets.clear();
    state.selected_preset_index = None;
    state.preview_theme_index = None;
    state.transition_theme = None;
    state.transition_from = None;
    state.resolved_theme = None;
    state.generation += 1;
}

fn build_hud(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let hud = tree
        .add_node()
        .window(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
        .without_pointer_events()
        .entity();
    handles.hud = Some(hud);
    tree.in_parent(hud, |tree| {
        handles.condition = Some(hud_text(
            tree,
            Rl(vec2(0.0, 0.0)) + Ab(vec2(24.0, 20.0)),
            Anchor::TopLeft,
            TextAlignment::Left,
            HUD_SIZE,
        ));
        handles.weapon = Some(hud_text(
            tree,
            Rl(vec2(0.0, 100.0)) + Ab(vec2(24.0, -20.0)),
            Anchor::BottomLeft,
            TextAlignment::Left,
            HUD_SIZE,
        ));
        handles.keys = Some(hud_text(
            tree,
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-24.0, -20.0)),
            Anchor::BottomRight,
            TextAlignment::Right,
            LABEL_SIZE,
        ));
        handles.message = Some(hud_text(
            tree,
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 28.0)),
            Anchor::TopCenter,
            TextAlignment::Center,
            LABEL_SIZE,
        ));
        handles.prompt = Some(hud_text(
            tree,
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -70.0)),
            Anchor::BottomCenter,
            TextAlignment::Center,
            LABEL_SIZE,
        ));
        handles.radio = Some(hud_text(
            tree,
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-24.0, 20.0)),
            Anchor::TopRight,
            TextAlignment::Right,
            LABEL_SIZE,
        ));
        handles.stance = Some(hud_text(
            tree,
            Rl(vec2(0.0, 100.0)) + Ab(vec2(24.0, -48.0)),
            Anchor::BottomLeft,
            TextAlignment::Left,
            LABEL_SIZE,
        ));
        handles.battery = Some(hud_text(
            tree,
            Rl(vec2(0.0, 0.0)) + Ab(vec2(24.0, 44.0)),
            Anchor::TopLeft,
            TextAlignment::Left,
            LABEL_SIZE,
        ));
        tree.add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(20.0, 20.0)), Anchor::Center)
            .with_text("+", 14.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(TEXT_DIM)
            .without_pointer_events();
    });
}

fn hud_text(
    tree: &mut UiTreeBuilder,
    position: UiValue<Vec2>,
    anchor: Anchor,
    alignment: TextAlignment,
    size: f32,
) -> Entity {
    tree.add_node()
        .window(position, Ab(vec2(700.0, size + 12.0)), anchor)
        .with_text("", size)
        .with_text_alignment(alignment, VerticalAlignment::Middle)
        .with_text_outline(TEXT_OUTLINE, 1.2)
        .color_raw::<UiBase>(TEXT_PRIMARY)
        .without_pointer_events()
        .entity()
}

fn build_title(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let title = add_menu_panel(tree, &menu_style(), TITLE_WIDTH, TITLE_HEIGHT);
    handles.title = Some(title);
    tree.in_parent(title, |tree| {
        text_row(tree, "A S H F A L L", TITLE_SIZE, TEXT_RUST);
        rule(tree);
        text_row(
            tree,
            "The fog came in on a Tuesday and it never left.",
            LABEL_SIZE,
            TEXT_PRIMARY,
        );
        text_row(
            tree,
            "Three keys stand between you and the church gate.",
            LABEL_SIZE,
            TEXT_PRIMARY,
        );
        rule(tree);
        text_row(
            tree,
            "Nobody drove out. The road is still there. It does not go anywhere.",
            LABEL_SIZE,
            TEXT_DIM,
        );
        handles.play_button = Some(tree.add_button("Enter the fog"));
        handles.quit_button = Some(tree.add_button("Quit"));
    });
}

fn build_pause(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let pause = add_menu_panel(tree, &menu_style(), MENU_WIDTH, MENU_HEIGHT);
    handles.pause = Some(pause);
    tree.in_parent(pause, |tree| {
        text_row(tree, "The town waits", HEADING_SIZE, TEXT_RUST);
        rule(tree);
        handles.resume_button = Some(tree.add_button("Resume"));
        handles.restart_button = Some(tree.add_button("Restart"));
        handles.pause_title_button = Some(tree.add_button("Quit to title"));
    });
}

fn build_note(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let note = add_menu_panel(tree, &menu_style(), NOTE_WIDTH, NOTE_HEIGHT);
    handles.note = Some(note);
    tree.in_parent(note, |tree| {
        handles.note_title = Some(text_row(tree, "", HEADING_SIZE, TEXT_RUST));
        rule(tree);
        handles.note_body = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 180.0)))
                .with_text("", NOTE_SIZE)
                .with_text_wrap()
                .text_left()
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .entity(),
        );
        handles.note_close_button = Some(tree.add_button("Put it back"));
    });
}

fn build_talk(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let talk = add_menu_panel(tree, &menu_style(), NOTE_WIDTH, MENU_HEIGHT + 60.0);
    handles.talk = Some(talk);
    tree.in_parent(talk, |tree| {
        handles.talk_name = Some(text_row(tree, "", HEADING_SIZE, TEXT_RUST));
        rule(tree);
        handles.talk_line = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 110.0)))
                .with_text("", NOTE_SIZE)
                .with_text_wrap()
                .text_left()
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .entity(),
        );
        handles.talk_button = Some(tree.add_button("Go on"));
    });
}

fn build_inventory(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let inventory = add_menu_panel(tree, &menu_style(), INVENTORY_WIDTH, INVENTORY_HEIGHT);
    handles.inventory = Some(inventory);
    tree.in_parent(inventory, |tree| {
        text_row(tree, "WHAT YOU CARRY", HEADING_SIZE, TEXT_RUST);
        rule(tree);
        for index in 0..SATCHEL_SLOTS {
            handles.inventory_slots[index] = Some(
                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
                    .with_text("", NOTE_SIZE)
                    .text_left()
                    .color_raw::<UiBase>(TEXT_PRIMARY)
                    .entity(),
            );
        }
        rule(tree);
        handles.inventory_actions = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
                .with_text("", LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(TEXT_RUST)
                .entity(),
        );
        handles.inventory_carried = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
                .with_text("", LABEL_SIZE)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity(),
        );
        text_row(tree, "PAPERS YOU KEPT", LABEL_SIZE, TEXT_DIM);
        handles.inventory_notes = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 160.0)))
                .with_text("", LABEL_SIZE)
                .with_text_wrap()
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity(),
        );
        handles.inventory_close_button = Some(tree.add_button("Close"));
    });
}

fn build_inspect(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let inspect = tree
        .add_node()
        .window(Rl(vec2(50.0, 50.0)), Rl(vec2(100.0, 100.0)), Anchor::Center)
        .with_rect(2.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(MAP_WASH)
        .entity();
    handles.inspect = Some(inspect);
    tree.in_parent(inspect, |tree| {
        handles.inspect_name = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(28.0, 24.0)),
                    Rl(vec2(60.0, 0.0)) + Ab(vec2(0.0, HEADING_SIZE + 8.0)),
                    Anchor::TopLeft,
                )
                .with_text("", HEADING_SIZE)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .color_raw::<UiBase>(TEXT_RUST)
                .entity(),
        );
        handles.inspect_body = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(28.0, -74.0)),
                    Rl(vec2(70.0, 0.0)) + Ab(vec2(0.0, 44.0)),
                    Anchor::BottomLeft,
                )
                .with_text("", LABEL_SIZE)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .entity(),
        );
        handles.inspect_hint = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(28.0, -26.0)),
                    Rl(vec2(70.0, 0.0)) + Ab(vec2(0.0, LABEL_SIZE + 8.0)),
                    Anchor::BottomLeft,
                )
                .with_text("", LABEL_SIZE)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .color_raw::<UiBase>(TEXT_DIM)
                .entity(),
        );
    });
}

fn build_town_map(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let map = tree
        .add_node()
        .window(Rl(vec2(50.0, 50.0)), Rl(vec2(100.0, 100.0)), Anchor::Center)
        .with_rect(2.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(MAP_WASH)
        .entity();
    handles.town_map = Some(map);
    tree.in_parent(map, |tree| {
        handles.town_map_title = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(24.0, 22.0)),
                    Rl(vec2(60.0, 0.0)) + Ab(vec2(0.0, HEADING_SIZE + 8.0)),
                    Anchor::TopLeft,
                )
                .with_text("", LABEL_SIZE)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .color_raw::<UiBase>(TEXT_RUST)
                .entity(),
        );
        handles.town_map_room = Some(
            tree.add_node()
                .window(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(24.0, MAP_TOP)),
                    Rl(vec2(60.0, 0.0)) + Ab(vec2(0.0, HEADING_SIZE + 8.0)),
                    Anchor::TopLeft,
                )
                .with_text("", LABEL_SIZE)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .color_raw::<UiBase>(MAP_WALL)
                .entity(),
        );
        tree.add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(24.0, -22.0)),
                Rl(vec2(60.0, 0.0)) + Ab(vec2(0.0, LABEL_SIZE + 8.0)),
                Anchor::BottomLeft,
            )
            .with_text(
                "orange doors want a key, red doors are bolted, blue is a stair",
                LABEL_SIZE,
            )
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .color_raw::<UiBase>(TEXT_DIM);
    });
}

fn build_dead(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let dead = add_menu_panel(tree, &menu_style(), ENDING_WIDTH, MENU_HEIGHT);
    handles.dead = Some(dead);
    tree.in_parent(dead, |tree| {
        text_row(tree, "The fog keeps you.", HEADING_SIZE, TEXT_RUST);
        rule(tree);
        text_row(
            tree,
            "Nobody drove out. Nobody walked out. Nobody will come looking.",
            LABEL_SIZE,
            TEXT_DIM,
        );
        handles.dead_again_button = Some(tree.add_button("Wake up again"));
        handles.dead_title_button = Some(tree.add_button("Quit to title"));
    });
}

fn build_escaped(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let escaped = add_menu_panel(tree, &menu_style(), ENDING_WIDTH, ENDING_HEIGHT);
    handles.escaped = Some(escaped);
    tree.in_parent(escaped, |tree| {
        text_row(tree, "The bell stops.", HEADING_SIZE, TEXT_RUST);
        rule(tree);
        handles.escaped_body = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 110.0)))
                .with_text("", NOTE_SIZE)
                .with_text_wrap()
                .text_left()
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .entity(),
        );
        handles.escaped_again_button = Some(tree.add_button("Go back in"));
        handles.escaped_title_button = Some(tree.add_button("Quit to title"));
    });
}

fn rule(tree: &mut UiTreeBuilder) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 2.0)))
        .with_rect(0.0, 0.0, TRANSPARENT)
        .color_raw::<UiBase>(PANEL_RULE);
}

fn text_row(tree: &mut UiTreeBuilder, text: &str, size: f32, color: Vec4) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + ROW_HEIGHT - LABEL_SIZE)))
        .with_text(text, size)
        .color_raw::<UiBase>(color)
        .entity()
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BG,
        border: PANEL_BORDER,
        corner_radius: 2.0,
        border_width: 1.0,
        padding: 16.0,
        spacing: 8.0,
        visible: true,
        ..Default::default()
    }
}
