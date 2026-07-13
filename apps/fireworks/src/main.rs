use nightshade::ecs::ui::layout_types::{FlowAlignment, FlowDirection, FlowLayout};
use nightshade::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FireworksPlugin)
        .run()
}

#[derive(Clone, Copy, PartialEq)]
enum ExplosionType {
    Peony,
    Chrysanthemum,
    Willow,
    Ring,
    MultiBreak,
    Glitter,
    Crackle,
    Palm,
    Crossette,
    Strobe,
    Kamuro,
    ShapeSmiley,
    ShapeHeart,
    ShapeStar,
    ShapeCircle,
    ShapeCross,
    ShapeDiamond,
    SplittingShell,
}

#[derive(Clone, Copy, PartialEq)]
enum ShellType {
    Standard,
    Comet,
}

struct FireworkShell {
    entity: Entity,
    velocity: Vec3,
    fuse_time: f32,
    color: Vec3,
    secondary_color: Option<Vec3>,
    explosion_type: ExplosionType,
    _shell_type: ShellType,
    exploded: bool,
}

struct FireEffect {
    entity: Entity,
    time_remaining: f32,
}

struct SmokeEffect {
    entity: Entity,
    time_remaining: f32,
}

struct SpriteEffect {
    entity: Entity,
    _label: &'static str,
}

struct SubShell {
    entity: Entity,
    position: Vec3,
    velocity: Vec3,
    fuse_time: f32,
    color: Vec3,
    secondary_color: Option<Vec3>,
    explosion_type: ExplosionType,
}

#[derive(Clone)]
enum ShowAction {
    Type(ExplosionType),
    TypeWithColor(ExplosionType, Vec3),
    Random,
    Salvo(u32),
    Symmetric(ExplosionType, u32),
}

#[derive(Clone)]
struct ShowEvent {
    time: f32,
    action: ShowAction,
}

#[derive(Default)]
struct FireworksDemo {
    shells: Vec<FireworkShell>,
    sub_shells: Vec<SubShell>,
    fire_effects: Vec<FireEffect>,
    smoke_effects: Vec<SmokeEffect>,
    sprite_effects: Vec<SpriteEffect>,
    next_launch_time: f32,
    auto_launch: bool,
    launch_interval: f32,
    show_fire: bool,
    show_smoke: bool,
    salvo_mode: bool,
    salvo_count: u32,
    show_active: bool,
    show_start_time: f32,
    show_events: Vec<ShowEvent>,
    show_event_index: usize,
    ui: UiHandles,
    type_buttons: HashMap<Entity, ExplosionType>,
    last_show_active: bool,
    last_salvo_mode: bool,
    last_stats: Option<Stats>,
}

#[derive(Default, Clone, Copy)]
struct UiHandles {
    auto_launch: Entity,
    launch_interval: Entity,
    salvo_mode: Entity,
    salvo_count_label: Entity,
    salvo_count: Entity,
    bloom_intensity: Entity,
    launch_single: Entity,
    launch_salvo: Entity,
    finale: Entity,
    show_play: Entity,
    show_status_label: Entity,
    shells_label: Entity,
    sub_shells_label: Entity,
    fire_label: Entity,
    sprite_label: Entity,
    smoke_label: Entity,
    emitters_label: Entity,
}

#[derive(Default, Clone, Copy, PartialEq)]
struct Stats {
    shells: usize,
    sub_shells: usize,
    fire: usize,
    sprite_total: usize,
    sprite_active: usize,
    smoke: usize,
    emitters: usize,
}

struct FireworksPlugin;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.window.title = "Fireworks".to_string();
        app.insert_resource(FireworksDemo::default());
        app.add_startup_system("fireworks_initialize", |world| {
            game_scope::<FireworksDemo, _>(world, initialize);
        });
        app.add_system(Stage::Update, "fireworks_update", |world| {
            game_scope::<FireworksDemo, _>(world, update);
        });
    }
}

fn initialize(fireworks: &mut FireworksDemo, world: &mut World) {
    world.resources.render_settings.atmosphere = Atmosphere::Space;
    world.resources.debug_draw.show_grid = true;
    world.resources.render_settings.clear_color = [0.01, 0.01, 0.02, 1.0];
    world.resources.render_settings.bloom_enabled = true;
    world.resources.render_settings.bloom_intensity = 0.05;

    let camera = spawn_entities(
        world,
        LOCAL_TRANSFORM | GLOBAL_TRANSFORM | CAMERA | PAN_ORBIT_CAMERA,
        1,
    )[0];

    world.set(
        camera,
        LocalTransform {
            translation: Vec3::new(0.0, 50.0, 150.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set(camera, GlobalTransform::default());
    world.set(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_near: 0.1,
                z_far: Some(2000.0),
            }),
            smoothing: Some(Smoothing::default()),
        },
    );
    world.set(
        camera,
        PanOrbitCamera {
            focus: Vec3::new(0.0, 40.0, 0.0),
            target_focus: Vec3::new(0.0, 40.0, 0.0),
            radius: 150.0,
            target_radius: 150.0,
            pitch: 0.2,
            target_pitch: 0.2,
            yaw: 0.0,
            target_yaw: 0.0,
            ..Default::default()
        },
    );
    world.resources.active_camera = Some(camera);

    fireworks.auto_launch = true;
    fireworks.launch_interval = 0.3;
    fireworks.show_fire = true;
    fireworks.show_smoke = true;
    fireworks.next_launch_time = 0.0;
    fireworks.salvo_mode = false;
    fireworks.salvo_count = 5;

    if fireworks.show_fire {
        fireworks.spawn_fire_effect(world, Vec3::new(-35.0, 0.0, 0.0));
        fireworks.spawn_fire_effect(world, Vec3::new(35.0, 0.0, 0.0));
        fireworks.spawn_fire_effect(world, Vec3::new(-25.0, 0.0, 10.0));
        fireworks.spawn_fire_effect(world, Vec3::new(25.0, 0.0, 10.0));
    }

    if fireworks.show_smoke {
        fireworks.spawn_smoke_effect(world, Vec3::new(-30.0, 3.0, 5.0));
        fireworks.spawn_smoke_effect(world, Vec3::new(30.0, 3.0, 5.0));
    }

    FireworksDemo::load_all_particle_textures(world);
    fireworks.spawn_all_sprite_effects(world);

    spawn_ui_text_with_properties(
        world,
        "GPU Particle System - Fireworks\nMouse: Orbit | Scroll: Zoom | Space: Launch | 1-4: Salvo | ESC: Exit",
        Vec2::zeros(),
        TextProperties {
            font_size: 18.0,
            color: vec4(1.0, 1.0, 1.0, 0.9),
            alignment: TextAlignment::Center,
            outline_width: 0.01,
            outline_color: vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );

    fireworks.build_ui(world);
}

fn update(fireworks: &mut FireworksDemo, world: &mut World) {
    escape_key_exit_system(world);
    fireworks.poll_ui_events(world);
    fireworks.update_ui_state(world);

    camera_controllers_system(world);

    let delta_time = world.resources.window.timing.delta_time;
    let uptime = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    if fireworks.show_active {
        fireworks.update_fireworks_show(world, uptime);
    } else if fireworks.auto_launch && uptime >= fireworks.next_launch_time {
        if fireworks.salvo_mode {
            fireworks.launch_salvo(world);
        } else {
            fireworks.launch_firework(world);
        }
        fireworks.next_launch_time = uptime + fireworks.launch_interval;
    }

    fireworks.update_shells(world, delta_time);
    fireworks.update_sub_shells(world, delta_time);
    fireworks.cleanup_expired_effects(world, delta_time);

    let events = std::mem::take(&mut world.resources.input.events);
    for event in events {
        let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
        else {
            continue;
        };
        match key {
            KeyCode::Space => {
                fireworks.launch_firework(world);
            }
            KeyCode::Digit1 => {
                for _ in 0..3 {
                    fireworks.launch_firework(world);
                }
            }
            KeyCode::Digit2 => {
                for _ in 0..5 {
                    fireworks.launch_firework(world);
                }
            }
            KeyCode::Digit3 => {
                for _ in 0..10 {
                    fireworks.launch_firework(world);
                }
            }
            KeyCode::Digit4 => {
                fireworks.launch_finale(world);
            }
            KeyCode::Digit5 => {
                fireworks.salvo_mode = !fireworks.salvo_mode;
            }
            KeyCode::KeyA => {
                fireworks.auto_launch = !fireworks.auto_launch;
            }
            KeyCode::KeyF => {
                fireworks.show_fire = !fireworks.show_fire;
                if fireworks.show_fire {
                    fireworks.spawn_fire_effect(world, Vec3::new(-35.0, 0.0, 0.0));
                    fireworks.spawn_fire_effect(world, Vec3::new(35.0, 0.0, 0.0));
                }
            }
            KeyCode::KeyS => {
                fireworks.show_smoke = !fireworks.show_smoke;
                if fireworks.show_smoke {
                    fireworks.spawn_smoke_effect(world, Vec3::new(-30.0, 3.0, 5.0));
                    fireworks.spawn_smoke_effect(world, Vec3::new(30.0, 3.0, 5.0));
                }
            }
            _ => {}
        }
    }
}

impl FireworksDemo {
    fn build_ui(&mut self, world: &mut World) {
        let auto_launch_initial = self.auto_launch;
        let launch_interval_initial = self.launch_interval;
        let salvo_mode_initial = self.salvo_mode;
        let salvo_count_initial = self.salvo_count as f32;
        let bloom_intensity_initial = world.resources.render_settings.bloom_intensity;

        let mut tree = UiTreeBuilder::new(world);
        let panel = tree.add_docked_panel_left("fireworks", "Fireworks", 260.0);
        let content = panel_content(tree.world_mut(), panel);
        if let Some(node) = tree
            .world_mut()
            .get_mut::<nightshade::ecs::ui::components::UiLayoutNode>(content)
        {
            node.flow_layout = Some(FlowLayout {
                direction: FlowDirection::Vertical,
                padding: 0.0,
                spacing: 0.0,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Start,
                wrap: false,
            });
        }
        let mut auto_launch = Entity::default();
        let mut launch_interval = Entity::default();
        let mut salvo_mode = Entity::default();
        let mut salvo_count_label = Entity::default();
        let mut salvo_count = Entity::default();
        let mut bloom_intensity = Entity::default();
        let mut launch_single = Entity::default();
        let mut launch_salvo = Entity::default();
        let mut finale = Entity::default();
        let mut type_buttons: HashMap<Entity, ExplosionType> = HashMap::new();
        let mut show_status_label = Entity::default();
        let mut show_play = Entity::default();
        let mut shells_label = Entity::default();
        let mut sub_shells_label = Entity::default();
        let mut fire_label = Entity::default();
        let mut sprite_label = Entity::default();
        let mut smoke_label = Entity::default();
        let mut emitters_label = Entity::default();

        tree.in_parent(content, |tree| {
            let scroll = tree.add_scroll_area_fill(8.0, 6.0);
            let scroll_body = widget::<UiScrollAreaData>(tree.world_mut(), scroll)
                .map(|data| data.content_entity)
                .expect("scroll body");
            tree.in_parent(scroll_body, |tree| {
                auto_launch = tree.add_checkbox("Auto-launch", auto_launch_initial);
                tree.add_label("Launch interval (s)");
                launch_interval = tree.add_slider(0.1, 2.0, launch_interval_initial);

                salvo_mode = tree.add_checkbox("Salvo mode", salvo_mode_initial);
                salvo_count_label = tree.add_label("Salvo count");
                salvo_count = tree.add_slider(3.0, 15.0, salvo_count_initial);

                tree.add_label("Bloom intensity");
                bloom_intensity = tree.add_slider(0.0, 1.0, bloom_intensity_initial);

                launch_single = tree.add_button("Launch Single");
                launch_salvo = tree.add_button("Launch Salvo (5)");
                finale = tree.add_button("FINALE!");

                collapsing(tree, "Launch by Type", false, |tree| {
                    tree.add_label("Standard");
                    for (label, explosion_type) in [
                        ("Peony", ExplosionType::Peony),
                        ("Chrysanthemum", ExplosionType::Chrysanthemum),
                        ("Willow", ExplosionType::Willow),
                        ("Ring", ExplosionType::Ring),
                        ("MultiBreak", ExplosionType::MultiBreak),
                        ("Glitter", ExplosionType::Glitter),
                        ("Crackle", ExplosionType::Crackle),
                        ("Palm", ExplosionType::Palm),
                        ("Crossette", ExplosionType::Crossette),
                        ("Strobe", ExplosionType::Strobe),
                        ("Kamuro", ExplosionType::Kamuro),
                    ] {
                        type_buttons.insert(tree.add_button(label), explosion_type);
                    }

                    tree.add_label("Shapes");
                    for (label, explosion_type) in [
                        ("Smiley", ExplosionType::ShapeSmiley),
                        ("Heart", ExplosionType::ShapeHeart),
                        ("Star", ExplosionType::ShapeStar),
                        ("Circle", ExplosionType::ShapeCircle),
                        ("Cross", ExplosionType::ShapeCross),
                        ("Diamond", ExplosionType::ShapeDiamond),
                    ] {
                        type_buttons.insert(tree.add_button(label), explosion_type);
                    }

                    tree.add_label("Special");
                    type_buttons.insert(
                        tree.add_button("Splitting Shell"),
                        ExplosionType::SplittingShell,
                    );
                });

                show_status_label = tree.add_label("");
                show_play = tree.add_button("Play Fireworks Show");

                tree.add_label("");
                tree.add_label("Stats");
                shells_label = tree.add_label("Active shells: 0");
                sub_shells_label = tree.add_label("Sub-shells: 0");
                fire_label = tree.add_label("Fire effects: 0");
                sprite_label = tree.add_label("Sprite effects: 0 (0 entities)");
                smoke_label = tree.add_label("Smoke effects: 0");
                emitters_label = tree.add_label("Active emitters: 0");
            });
        });
        tree.finish();

        ui_set_visible(world, salvo_count_label, salvo_mode_initial);
        ui_set_visible(world, salvo_count, salvo_mode_initial);

        let _ = panel;
        self.ui = UiHandles {
            auto_launch,
            launch_interval,
            salvo_mode,
            salvo_count_label,
            salvo_count,
            bloom_intensity,
            launch_single,
            launch_salvo,
            finale,
            show_play,
            show_status_label,
            shells_label,
            sub_shells_label,
            fire_label,
            sprite_label,
            smoke_label,
            emitters_label,
        };
        self.type_buttons = type_buttons;
        self.last_show_active = !self.show_active;
        self.last_salvo_mode = !salvo_mode_initial;
        self.last_stats = None;
    }

    fn poll_ui_events(&mut self, world: &mut World) {
        let mut launch_single = false;
        let mut launch_salvo = false;
        let mut launch_finale = false;
        let mut show_clicked = false;
        let mut explosion_request: Option<ExplosionType> = None;

        for event in ui_events(world) {
            if let UiEvent::ButtonClicked(entity) = event {
                if *entity == self.ui.launch_single {
                    launch_single = true;
                } else if *entity == self.ui.launch_salvo {
                    launch_salvo = true;
                } else if *entity == self.ui.finale {
                    launch_finale = true;
                } else if *entity == self.ui.show_play {
                    show_clicked = true;
                } else if let Some(explosion_type) = self.type_buttons.get(entity).copied() {
                    explosion_request = Some(explosion_type);
                }
            }
        }

        if let Some(value) = ui_checkbox_value(world, self.ui.auto_launch) {
            self.auto_launch = value;
        }
        if let Some(value) = ui_slider_value(world, self.ui.launch_interval) {
            self.launch_interval = value;
        }
        if let Some(value) = ui_checkbox_value(world, self.ui.salvo_mode) {
            self.salvo_mode = value;
        }
        if let Some(value) = ui_slider_value(world, self.ui.salvo_count) {
            self.salvo_count = value.round().clamp(3.0, 15.0) as u32;
        }
        if let Some(value) = ui_slider_value(world, self.ui.bloom_intensity) {
            world.resources.render_settings.bloom_intensity = value;
        }

        if launch_single {
            self.launch_firework(world);
        }
        if launch_salvo {
            self.launch_salvo(world);
        }
        if launch_finale {
            self.launch_finale(world);
        }
        if let Some(explosion_type) = explosion_request {
            self.launch_specific_firework(world, explosion_type, None, None);
        }
        if show_clicked {
            if self.show_active {
                self.show_active = false;
            } else {
                let uptime = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
                self.start_fireworks_show(uptime);
            }
        }
    }

    fn update_ui_state(&mut self, world: &mut World) {
        if self.last_salvo_mode != self.salvo_mode {
            ui_set_visible(world, self.ui.salvo_count_label, self.salvo_mode);
            ui_set_visible(world, self.ui.salvo_count, self.salvo_mode);
            self.last_salvo_mode = self.salvo_mode;
        }

        if self.last_show_active != self.show_active {
            if self.show_active {
                ui_set_text(world, self.ui.show_status_label, "Show in progress...");
                ui_set_text(world, self.ui.show_play, "Stop Show");
            } else {
                ui_set_text(world, self.ui.show_status_label, "");
                ui_set_text(world, self.ui.show_play, "Play Fireworks Show");
            }
            self.last_show_active = self.show_active;
        }

        let sprite_active = self
            .sprite_effects
            .iter()
            .filter(|effect| {
                world
                    .get::<nightshade::render::particles::ParticleEmitter>(effect.entity)
                    .is_some_and(|emitter| emitter.enabled)
            })
            .count();
        let emitters = world.ecs.worlds[CORE]
            .query_entities(PARTICLE_EMITTER)
            .count();
        let stats = Stats {
            shells: self.shells.len(),
            sub_shells: self.sub_shells.len(),
            fire: self.fire_effects.len(),
            sprite_total: self.sprite_effects.len(),
            sprite_active,
            smoke: self.smoke_effects.len(),
            emitters,
        };
        if self.last_stats != Some(stats) {
            ui_set_text(
                world,
                self.ui.shells_label,
                &format!("Active shells: {}", stats.shells),
            );
            ui_set_text(
                world,
                self.ui.sub_shells_label,
                &format!("Sub-shells: {}", stats.sub_shells),
            );
            ui_set_text(
                world,
                self.ui.fire_label,
                &format!("Fire effects: {}", stats.fire),
            );
            ui_set_text(
                world,
                self.ui.sprite_label,
                &format!(
                    "Sprite effects: {} ({} entities)",
                    stats.sprite_total, stats.sprite_active
                ),
            );
            ui_set_text(
                world,
                self.ui.smoke_label,
                &format!("Smoke effects: {}", stats.smoke),
            );
            ui_set_text(
                world,
                self.ui.emitters_label,
                &format!("Active emitters: {}", stats.emitters),
            );
            self.last_stats = Some(stats);
        }
    }

    fn random_color() -> Vec3 {
        let mut rng = rand::rng();

        if rng.random::<f32>() < 0.3 {
            let hue = rng.random::<f32>() * 360.0;
            let saturation = 0.7 + rng.random::<f32>() * 0.3;
            let value = 0.9 + rng.random::<f32>() * 0.1;
            Self::hsv_to_rgb(hue, saturation, value)
        } else {
            let firework_colors = [
                Vec3::new(1.0, 0.1, 0.1),
                Vec3::new(1.0, 0.2, 0.2),
                Vec3::new(0.1, 1.0, 0.1),
                Vec3::new(0.2, 1.0, 0.4),
                Vec3::new(0.1, 0.3, 1.0),
                Vec3::new(0.3, 0.5, 1.0),
                Vec3::new(1.0, 1.0, 0.1),
                Vec3::new(1.0, 0.9, 0.3),
                Vec3::new(1.0, 0.1, 1.0),
                Vec3::new(1.0, 0.4, 0.9),
                Vec3::new(0.1, 1.0, 1.0),
                Vec3::new(0.4, 1.0, 0.9),
                Vec3::new(1.0, 0.5, 0.0),
                Vec3::new(1.0, 0.6, 0.2),
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(0.95, 0.95, 1.0),
                Vec3::new(1.0, 0.6, 0.8),
                Vec3::new(1.0, 0.4, 0.6),
                Vec3::new(0.5, 0.2, 1.0),
                Vec3::new(0.7, 0.3, 1.0),
                Vec3::new(1.0, 0.8, 0.3),
                Vec3::new(0.3, 1.0, 0.6),
                Vec3::new(1.0, 0.3, 0.5),
                Vec3::new(0.2, 0.8, 1.0),
                Vec3::new(1.0, 0.7, 0.0),
                Vec3::new(0.8, 1.0, 0.2),
                Vec3::new(1.0, 0.2, 0.6),
                Vec3::new(0.6, 0.9, 1.0),
                Vec3::new(1.0, 0.5, 0.3),
                Vec3::new(0.4, 0.6, 1.0),
            ];
            firework_colors[rng.random_range(0..firework_colors.len())]
        }
    }

    fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
        let c = value * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = value - c;

        let (r, g, b) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Vec3::new(r + m, g + m, b + m)
    }

    fn random_explosion_type() -> ExplosionType {
        let mut rng = rand::rng();
        let types = [
            ExplosionType::Peony,
            ExplosionType::Peony,
            ExplosionType::Peony,
            ExplosionType::Chrysanthemum,
            ExplosionType::Chrysanthemum,
            ExplosionType::Willow,
            ExplosionType::Ring,
            ExplosionType::MultiBreak,
            ExplosionType::MultiBreak,
            ExplosionType::Glitter,
            ExplosionType::Crackle,
            ExplosionType::Palm,
            ExplosionType::Palm,
            ExplosionType::Crossette,
            ExplosionType::Strobe,
            ExplosionType::Kamuro,
            ExplosionType::ShapeSmiley,
            ExplosionType::ShapeHeart,
            ExplosionType::ShapeStar,
            ExplosionType::ShapeCircle,
            ExplosionType::ShapeCross,
            ExplosionType::ShapeDiamond,
            ExplosionType::SplittingShell,
            ExplosionType::SplittingShell,
        ];
        types[rng.random_range(0..types.len())]
    }

    fn random_sub_explosion_type() -> ExplosionType {
        let mut rng = rand::rng();
        let types = [
            ExplosionType::Peony,
            ExplosionType::Chrysanthemum,
            ExplosionType::Willow,
            ExplosionType::Glitter,
            ExplosionType::Crackle,
        ];
        types[rng.random_range(0..types.len())]
    }

    fn generate_smiley_pattern() -> Vec<Vec2> {
        let mut points = Vec::new();

        for index in 0..60 {
            let angle = (index as f32) * std::f32::consts::TAU / 60.0;
            points.push(vec2(angle.cos(), angle.sin()));
        }

        for index in 0..16 {
            let angle = (index as f32) * std::f32::consts::TAU / 16.0;
            let radius = 0.15;
            points.push(vec2(
                -0.38 + angle.cos() * radius,
                0.32 + angle.sin() * radius,
            ));
        }

        for index in 0..16 {
            let angle = (index as f32) * std::f32::consts::TAU / 16.0;
            let radius = 0.15;
            points.push(vec2(
                0.38 + angle.cos() * radius,
                0.32 + angle.sin() * radius,
            ));
        }

        for index in 0..30 {
            let t = index as f32 / 29.0;
            let angle = std::f32::consts::PI * 0.2 + t * std::f32::consts::PI * 0.6;
            points.push(vec2(angle.cos() * 0.5, -angle.sin() * 0.5 + 0.05));
        }

        points
    }

    fn generate_heart_pattern() -> Vec<Vec2> {
        let mut points = Vec::new();
        for index in 0..100 {
            let t = (index as f32) * std::f32::consts::TAU / 100.0;
            let x = 16.0 * t.sin().powi(3);
            let y =
                13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
            points.push(vec2(x / 17.0, y / 17.0));
        }
        points
    }

    fn generate_star_pattern() -> Vec<Vec2> {
        let mut points = Vec::new();
        let outer_radius = 1.0;
        let inner_radius = 0.38;

        for point_index in 0..5 {
            let outer_angle =
                (point_index as f32) * std::f32::consts::TAU / 5.0 - std::f32::consts::FRAC_PI_2;
            let inner_angle = outer_angle + std::f32::consts::TAU / 10.0;

            let outer_point = vec2(
                outer_angle.cos() * outer_radius,
                outer_angle.sin() * outer_radius,
            );
            let inner_point = vec2(
                inner_angle.cos() * inner_radius,
                inner_angle.sin() * inner_radius,
            );

            let next_outer_angle = ((point_index + 1) as f32) * std::f32::consts::TAU / 5.0
                - std::f32::consts::FRAC_PI_2;
            let next_outer_point = vec2(
                next_outer_angle.cos() * outer_radius,
                next_outer_angle.sin() * outer_radius,
            );

            for step in 0..10 {
                let t = step as f32 / 10.0;
                points.push(vec2(
                    outer_point.x * (1.0 - t) + inner_point.x * t,
                    outer_point.y * (1.0 - t) + inner_point.y * t,
                ));
            }
            for step in 0..10 {
                let t = step as f32 / 10.0;
                points.push(vec2(
                    inner_point.x * (1.0 - t) + next_outer_point.x * t,
                    inner_point.y * (1.0 - t) + next_outer_point.y * t,
                ));
            }
        }

        points
    }

    fn generate_circle_pattern() -> Vec<Vec2> {
        let mut points = Vec::new();
        for index in 0..60 {
            let angle = (index as f32) * std::f32::consts::TAU / 60.0;
            points.push(vec2(angle.cos(), angle.sin()));
        }
        for index in 0..40 {
            let angle = (index as f32) * std::f32::consts::TAU / 40.0;
            points.push(vec2(angle.cos() * 0.5, angle.sin() * 0.5));
        }
        points
    }

    fn generate_cross_pattern() -> Vec<Vec2> {
        let mut points = Vec::new();
        let arm_length = 1.0;

        for index in 0..25 {
            let t = index as f32 / 24.0;
            let pos = -arm_length + t * 2.0 * arm_length;
            points.push(vec2(0.0, pos));
        }

        for index in 0..25 {
            let t = index as f32 / 24.0;
            let pos = -arm_length + t * 2.0 * arm_length;
            points.push(vec2(pos, 0.0));
        }

        points
    }

    fn generate_diamond_pattern() -> Vec<Vec2> {
        let mut points = Vec::new();
        let size = 1.0;

        let corners = [
            vec2(0.0, size),
            vec2(size, 0.0),
            vec2(0.0, -size),
            vec2(-size, 0.0),
        ];

        for edge in 0..4 {
            let start = corners[edge];
            let end = corners[(edge + 1) % 4];
            for step in 0..15 {
                let t = step as f32 / 15.0;
                points.push(vec2(
                    start.x * (1.0 - t) + end.x * t,
                    start.y * (1.0 - t) + end.y * t,
                ));
            }
        }

        for index in 0..12 {
            let t = index as f32 / 11.0;
            points.push(vec2(0.0, -size * 0.5 + t * size));
        }
        for index in 0..12 {
            let t = index as f32 / 11.0;
            points.push(vec2(-size * 0.5 + t * size, 0.0));
        }

        points
    }

    fn spawn_shaped_explosion(
        world: &mut World,
        pos: Vec3,
        color: Vec3,
        pattern: &[Vec2],
        scale: f32,
        particles_per_point: u32,
    ) {
        let mut rng = rand::rng();

        let max_emitters = 30;
        let step = (pattern.len() / max_emitters).max(1);

        for (index, point) in pattern.iter().enumerate() {
            if index % step != 0 {
                continue;
            }

            let point_magnitude = (point.x * point.x + point.y * point.y).sqrt();
            if point_magnitude < 0.001 {
                continue;
            }

            let direction = Vec3::new(point.x, point.y, 0.0).normalize();
            let speed = scale * point_magnitude * rng.random_range(0.9..1.1);

            let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];

            let mut emitter = ParticleEmitter::firework_explosion(pos, color, particles_per_point);
            emitter.burst_count = particles_per_point;
            emitter.spawn_rate = 0.0;
            emitter.one_shot = true;
            emitter.shape = nightshade::render::particles::EmitterShape::Point;
            emitter.direction = direction;
            emitter.initial_velocity_min = speed * 0.85;
            emitter.initial_velocity_max = speed * 1.15;
            emitter.velocity_spread = 0.15;
            emitter.particle_lifetime_min = 2.0;
            emitter.particle_lifetime_max = 3.0;
            emitter.size_start = 0.35;
            emitter.size_end = 0.08;
            emitter.drag = 0.25;
            emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
            emitter.emissive_strength = 12.0;

            world.set(entity, emitter);
        }
    }

    fn random_shell_type() -> ShellType {
        let mut rng = rand::rng();
        if rng.random::<f32>() < 0.3 {
            ShellType::Comet
        } else {
            ShellType::Standard
        }
    }

    fn spawn_explosion_for(
        world: &mut World,
        pos: Vec3,
        color: Vec3,
        secondary_color: Option<Vec3>,
        explosion_type: ExplosionType,
    ) {
        let mut rng = rand::rng();

        let flash_entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let flash_emitter = ParticleEmitter::flash_burst(pos);
        world.set(flash_entity, flash_emitter);

        match explosion_type {
            ExplosionType::Peony => {
                let particle_count: u32 = rng.random_range(500..900);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, particle_count);
                world.set(entity, emitter);

                if let Some(secondary) = secondary_color {
                    let entity2 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter2 =
                        ParticleEmitter::firework_explosion(pos, secondary, particle_count / 2);
                    emitter2.initial_velocity_min *= 0.7;
                    emitter2.initial_velocity_max *= 0.7;
                    emitter2.size_start *= 0.8;
                    world.set(entity2, emitter2);
                }

                let spark_count: u32 = rng.random_range(100..200);
                let spark_entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let spark_emitter = ParticleEmitter::firework_glitter(pos, spark_count);
                world.set(spark_entity, spark_emitter);
            }
            ExplosionType::Chrysanthemum => {
                let particle_count: u32 = rng.random_range(700..1200);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_chrysanthemum(pos, color, particle_count);
                world.set(entity, emitter);

                if let Some(secondary) = secondary_color {
                    let inner_count: u32 = rng.random_range(300..500);
                    let entity2 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter2 =
                        ParticleEmitter::firework_explosion(pos, secondary, inner_count);
                    emitter2.initial_velocity_min *= 0.5;
                    emitter2.initial_velocity_max *= 0.5;
                    world.set(entity2, emitter2);
                }
            }
            ExplosionType::Willow => {
                let particle_count: u32 = rng.random_range(600..1000);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_willow(pos, color, particle_count);
                world.set(entity, emitter);

                let tip_count: u32 = rng.random_range(150..300);
                let tip_entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let tip_emitter = ParticleEmitter::firework_glitter(pos, tip_count);
                world.set(tip_entity, tip_emitter);
            }
            ExplosionType::Ring => {
                let particle_count: u32 = rng.random_range(350..600);

                for angle_index in 0..4 {
                    let angle = (angle_index as f32) * std::f32::consts::TAU / 4.0;
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::firework_ring(pos, color, particle_count);
                    emitter.direction = Vec3::new(angle.sin(), 0.0, angle.cos());
                    world.set(entity, emitter);
                }

                if let Some(secondary) = secondary_color {
                    let center_count: u32 = rng.random_range(200..400);
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let emitter = ParticleEmitter::firework_explosion(pos, secondary, center_count);
                    world.set(entity, emitter);
                }
            }
            ExplosionType::MultiBreak => {
                let main_count: u32 = rng.random_range(400..600);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set(entity, emitter);

                for sub_index in 0..rng.random_range(4..8) {
                    let offset = Vec3::new(
                        rng.random_range(-10.0..10.0),
                        rng.random_range(-6.0..12.0),
                        rng.random_range(-10.0..10.0),
                    );
                    let sub_count: u32 = rng.random_range(180..350);
                    let sub_color = match sub_index % 3 {
                        0 => color,
                        1 => secondary_color.unwrap_or(Self::random_color()),
                        _ => Self::random_color(),
                    };
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter =
                        ParticleEmitter::firework_explosion(pos + offset, sub_color, sub_count);
                    emitter.initial_velocity_min *= 0.6;
                    emitter.initial_velocity_max *= 0.6;
                    emitter.size_start *= 0.7;
                    world.set(entity, emitter);
                }
            }
            ExplosionType::Glitter => {
                let main_count: u32 = rng.random_range(500..800);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set(entity, emitter);

                let glitter_count: u32 = rng.random_range(300..500);
                let entity2 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter2 = ParticleEmitter::firework_glitter(pos, glitter_count);
                world.set(entity2, emitter2);

                if let Some(secondary) = secondary_color {
                    let extra_glitter: u32 = rng.random_range(150..300);
                    let entity3 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter3 = ParticleEmitter::firework_glitter(pos, extra_glitter);
                    emitter3.color_gradient = ColorGradient::firework_explosion(secondary);
                    world.set(entity3, emitter3);
                }
            }
            ExplosionType::Crackle => {
                let main_count: u32 = rng.random_range(400..700);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set(entity, emitter);

                let crackle_count: u32 = rng.random_range(200..400);
                let entity2 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter2 = ParticleEmitter::firework_crackle(pos, crackle_count);
                world.set(entity2, emitter2);

                let extra_crackle: u32 = rng.random_range(100..200);
                let entity3 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let mut emitter3 = ParticleEmitter::firework_crackle(pos, extra_crackle);
                emitter3.particle_lifetime_min *= 1.5;
                emitter3.particle_lifetime_max *= 1.5;
                world.set(entity3, emitter3);
            }
            ExplosionType::Palm => {
                let trunk_count: u32 = rng.random_range(100..180);
                for frond_index in 0..rng.random_range(6..10) {
                    let angle = (frond_index as f32) * std::f32::consts::TAU / 8.0;
                    let tilt = rng.random_range(0.3..0.6);
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::palm_explosion(pos, color, trunk_count);
                    emitter.direction =
                        Vec3::new(angle.sin() * tilt, 1.0 - tilt * 0.5, angle.cos() * tilt)
                            .normalize();
                    world.set(entity, emitter);
                }

                if let Some(secondary) = secondary_color {
                    let tip_count: u32 = rng.random_range(200..400);
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::firework_glitter(pos, tip_count);
                    emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                    world.set(entity, emitter);
                }
            }
            ExplosionType::Crossette => {
                let main_count: u32 = rng.random_range(400..700);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set(entity, emitter);

                let split_count = rng.random_range(8..15);
                for _ in 0..split_count {
                    let offset = Vec3::new(
                        rng.random_range(-12.0..12.0),
                        rng.random_range(-8.0..12.0),
                        rng.random_range(-12.0..12.0),
                    );
                    let burst_count: u32 = rng.random_range(80..150);
                    let burst_color = if rng.random::<f32>() > 0.5 {
                        secondary_color.unwrap_or(color)
                    } else {
                        color
                    };
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let emitter =
                        ParticleEmitter::crossette_burst(pos + offset, burst_color, burst_count);
                    world.set(entity, emitter);
                }
            }
            ExplosionType::Strobe => {
                let main_count: u32 = rng.random_range(500..800);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set(entity, emitter);

                let strobe_count: u32 = rng.random_range(150..300);
                let entity2 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter2 = ParticleEmitter::strobe_effect(pos, strobe_count);
                world.set(entity2, emitter2);
            }
            ExplosionType::Kamuro => {
                let particle_count: u32 = rng.random_range(1000..1500);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let mut emitter = ParticleEmitter::firework_willow(pos, color, particle_count);
                emitter.initial_velocity_min = 20.0;
                emitter.initial_velocity_max = 35.0;
                emitter.particle_lifetime_min = 4.0;
                emitter.particle_lifetime_max = 7.0;
                emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
                emitter.drag = 0.08;
                emitter.size_start = 0.25;
                emitter.emissive_strength = 10.0;
                world.set(entity, emitter);

                if let Some(secondary) = secondary_color {
                    let inner_count: u32 = rng.random_range(400..600);
                    let entity2 = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter2 =
                        ParticleEmitter::firework_chrysanthemum(pos, secondary, inner_count);
                    emitter2.initial_velocity_min *= 0.5;
                    emitter2.initial_velocity_max *= 0.5;
                    world.set(entity2, emitter2);
                }
            }
            ExplosionType::ShapeSmiley => {
                let pattern = Self::generate_smiley_pattern();
                Self::spawn_shaped_explosion(world, pos, color, &pattern, 20.0, 8);
                if let Some(secondary) = secondary_color {
                    let glitter_count: u32 = rng.random_range(100..200);
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::firework_glitter(pos, glitter_count);
                    emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                    world.set(entity, emitter);
                }
            }
            ExplosionType::ShapeHeart => {
                let pattern = Self::generate_heart_pattern();
                Self::spawn_shaped_explosion(world, pos, color, &pattern, 22.0, 10);
                if let Some(secondary) = secondary_color {
                    let inner = Self::generate_heart_pattern();
                    Self::spawn_shaped_explosion(world, pos, secondary, &inner, 12.0, 5);
                }
            }
            ExplosionType::ShapeStar => {
                let pattern = Self::generate_star_pattern();
                Self::spawn_shaped_explosion(world, pos, color, &pattern, 20.0, 12);
                let glitter_count: u32 = rng.random_range(150..250);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_glitter(pos, glitter_count);
                world.set(entity, emitter);
            }
            ExplosionType::ShapeCircle => {
                let pattern = Self::generate_circle_pattern();
                Self::spawn_shaped_explosion(world, pos, color, &pattern, 22.0, 10);
                if let Some(secondary) = secondary_color {
                    let glitter_count: u32 = rng.random_range(150..250);
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::firework_glitter(pos, glitter_count);
                    emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                    world.set(entity, emitter);
                }
            }
            ExplosionType::ShapeCross => {
                let pattern = Self::generate_cross_pattern();
                Self::spawn_shaped_explosion(world, pos, color, &pattern, 20.0, 10);
                if let Some(secondary) = secondary_color {
                    let center_count: u32 = rng.random_range(100..200);
                    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let emitter = ParticleEmitter::firework_explosion(pos, secondary, center_count);
                    world.set(entity, emitter);
                }
            }
            ExplosionType::ShapeDiamond => {
                let pattern = Self::generate_diamond_pattern();
                Self::spawn_shaped_explosion(world, pos, color, &pattern, 22.0, 10);
                if let Some(secondary) = secondary_color {
                    let inner_pattern = Self::generate_diamond_pattern();
                    Self::spawn_shaped_explosion(world, pos, secondary, &inner_pattern, 12.0, 6);
                }
            }
            ExplosionType::SplittingShell => {
                let initial_burst: u32 = rng.random_range(200..400);
                let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, initial_burst);
                world.set(entity, emitter);
            }
        }
    }

    fn spawn_splitting_sub_shells(
        &mut self,
        world: &mut World,
        pos: Vec3,
        color: Vec3,
        secondary_color: Option<Vec3>,
    ) {
        let mut rng = rand::rng();
        let sub_count = rng.random_range(4..8);

        for _ in 0..sub_count {
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            let elevation = rng.random_range(0.2..0.6);
            let speed = rng.random_range(20.0..35.0);

            let velocity = Vec3::new(
                angle.cos() * (1.0 - elevation) * speed,
                elevation * speed,
                angle.sin() * (1.0 - elevation) * speed,
            );

            let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
            let trail_emitter = ParticleEmitter::firework_shell(pos, velocity);
            world.set(entity, trail_emitter);

            let sub_color = if rng.random::<f32>() > 0.5 {
                secondary_color.unwrap_or(Self::random_color())
            } else {
                color
            };

            self.sub_shells.push(SubShell {
                entity,
                position: pos,
                velocity,
                fuse_time: rng.random_range(0.8..1.4),
                color: sub_color,
                secondary_color: if rng.random::<f32>() > 0.6 {
                    Some(Self::random_color())
                } else {
                    None
                },
                explosion_type: Self::random_sub_explosion_type(),
            });
        }
    }

    fn update_sub_shells(&mut self, world: &mut World, delta_time: f32) {
        let mut explosions_to_spawn: Vec<(Vec3, Vec3, Option<Vec3>, ExplosionType, Entity)> =
            Vec::new();

        for sub_shell in self.sub_shells.iter_mut() {
            sub_shell.fuse_time -= delta_time;
            sub_shell.position += sub_shell.velocity * delta_time;
            sub_shell.velocity.y -= 9.81 * delta_time;

            if let Some(emitter) =
                world.get_mut::<nightshade::render::particles::ParticleEmitter>(sub_shell.entity)
            {
                emitter.position = sub_shell.position;
            }

            if sub_shell.fuse_time <= 0.0 {
                explosions_to_spawn.push((
                    sub_shell.position,
                    sub_shell.color,
                    sub_shell.secondary_color,
                    sub_shell.explosion_type,
                    sub_shell.entity,
                ));
            }
        }

        for (pos, color, secondary, explosion_type, entity) in explosions_to_spawn {
            Self::spawn_explosion_for(world, pos, color, secondary, explosion_type);
            if let Some(emitter) =
                world.get_mut::<nightshade::render::particles::ParticleEmitter>(entity)
            {
                emitter.enabled = false;
            }
        }

        self.sub_shells
            .retain(|sub_shell| sub_shell.fuse_time > 0.0);
    }

    fn launch_firework(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        let spread = 60.0;
        let x_offset: f32 = rng.random_range(-spread..spread);
        let z_offset: f32 = rng.random_range(-20.0..20.0);

        let launch_pos = Vec3::new(x_offset, 0.0, z_offset);
        let target_height: f32 = rng.random_range(55.0..95.0);

        let velocity = Vec3::new(
            rng.random_range(-4.0..4.0),
            rng.random_range(50.0..75.0),
            rng.random_range(-3.0..3.0),
        );

        let fuse_time = target_height / velocity.y;

        let color = Self::random_color();
        let secondary_color = if rng.random::<f32>() > 0.5 {
            Some(Self::random_color())
        } else {
            None
        };

        let explosion_type = Self::random_explosion_type();
        let shell_type = Self::random_shell_type();

        let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let trail_emitter = match shell_type {
            ShellType::Standard => ParticleEmitter::firework_shell(launch_pos, velocity),
            ShellType::Comet => ParticleEmitter::comet_shell(launch_pos, velocity),
        };
        world.set(entity, trail_emitter);

        self.shells.push(FireworkShell {
            entity,
            velocity,
            fuse_time,
            color,
            secondary_color,
            explosion_type,
            _shell_type: shell_type,
            exploded: false,
        });
    }

    fn launch_salvo(&mut self, world: &mut World) {
        for _ in 0..self.salvo_count {
            self.launch_firework(world);
        }
    }

    fn launch_finale(&mut self, world: &mut World) {
        for _ in 0..50 {
            self.launch_firework(world);
        }
    }

    fn launch_specific_firework(
        &mut self,
        world: &mut World,
        explosion_type: ExplosionType,
        color: Option<Vec3>,
        x_position: Option<f32>,
    ) {
        let mut rng = rand::rng();

        let x_offset = x_position.unwrap_or_else(|| rng.random_range(-60.0..60.0));
        let z_offset: f32 = rng.random_range(-20.0..20.0);

        let launch_pos = Vec3::new(x_offset, 0.0, z_offset);
        let target_height: f32 = rng.random_range(55.0..95.0);

        let velocity = Vec3::new(
            rng.random_range(-2.0..2.0),
            rng.random_range(50.0..75.0),
            rng.random_range(-2.0..2.0),
        );

        let fuse_time = target_height / velocity.y;

        let firework_color = color.unwrap_or_else(Self::random_color);
        let secondary_color = if rng.random::<f32>() > 0.4 {
            Some(Self::random_color())
        } else {
            None
        };

        let shell_type = Self::random_shell_type();

        let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let trail_emitter = match shell_type {
            ShellType::Standard => ParticleEmitter::firework_shell(launch_pos, velocity),
            ShellType::Comet => ParticleEmitter::comet_shell(launch_pos, velocity),
        };
        world.set(entity, trail_emitter);

        self.shells.push(FireworkShell {
            entity,
            velocity,
            fuse_time,
            color: firework_color,
            secondary_color,
            explosion_type,
            _shell_type: shell_type,
            exploded: false,
        });
    }

    fn launch_symmetric_fireworks(
        &mut self,
        world: &mut World,
        explosion_type: ExplosionType,
        count: u32,
    ) {
        let color = Self::random_color();
        let spacing = 100.0 / (count as f32);
        let start_x = -50.0 + spacing / 2.0;

        for index in 0..count {
            let x_pos = start_x + (index as f32) * spacing;
            self.launch_specific_firework(world, explosion_type, Some(color), Some(x_pos));
        }
    }

    fn create_show_events() -> Vec<ShowEvent> {
        let mut events = Vec::new();
        let mut time = 0.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::Peony),
        });
        time += 1.5;
        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::Chrysanthemum),
        });
        time += 1.5;
        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::Willow),
        });
        time += 2.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeHeart, 3),
        });
        time += 3.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::SplittingShell),
        });
        time += 2.5;
        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::SplittingShell),
        });
        time += 3.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeStar, 5),
        });
        time += 3.5;

        for _ in 0..3 {
            events.push(ShowEvent {
                time,
                action: ShowAction::Random,
            });
            time += 0.8;
        }
        time += 1.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeSmiley, 3),
        });
        time += 3.5;

        events.push(ShowEvent {
            time,
            action: ShowAction::Salvo(5),
        });
        time += 2.5;

        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::Kamuro),
        });
        time += 2.0;
        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::Kamuro),
        });
        time += 3.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeCross, 4),
        });
        time += 3.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::SplittingShell),
        });
        events.push(ShowEvent {
            time,
            action: ShowAction::Type(ExplosionType::SplittingShell),
        });
        time += 3.5;

        for _ in 0..5 {
            events.push(ShowEvent {
                time,
                action: ShowAction::Random,
            });
            time += 0.5;
        }
        time += 1.5;

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeDiamond, 5),
        });
        time += 3.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Salvo(8),
        });
        time += 2.0;

        for _ in 0..8 {
            events.push(ShowEvent {
                time,
                action: ShowAction::Random,
            });
            time += 0.4;
        }
        time += 2.0;

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeHeart, 7),
        });
        time += 4.0;

        time += 2.0;

        for burst in 0..15 {
            let count = 3 + burst / 3;
            events.push(ShowEvent {
                time,
                action: ShowAction::Salvo(count),
            });
            time += 0.3 - (burst as f32 * 0.01);
        }

        for _ in 0..30 {
            events.push(ShowEvent {
                time,
                action: ShowAction::Random,
            });
            time += 0.08;
        }

        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeHeart, 5),
        });
        events.push(ShowEvent {
            time,
            action: ShowAction::Symmetric(ExplosionType::ShapeStar, 5),
        });
        events.push(ShowEvent {
            time,
            action: ShowAction::TypeWithColor(ExplosionType::Kamuro, Vec3::new(1.0, 0.8, 0.2)),
        });
        events.push(ShowEvent {
            time,
            action: ShowAction::TypeWithColor(ExplosionType::Kamuro, Vec3::new(1.0, 0.3, 0.1)),
        });
        time += 0.5;

        for _ in 0..20 {
            events.push(ShowEvent {
                time,
                action: ShowAction::Random,
            });
            time += 0.05;
        }

        events
    }

    fn start_fireworks_show(&mut self, uptime: f32) {
        self.show_active = true;
        self.show_start_time = uptime;
        self.show_events = Self::create_show_events();
        self.show_event_index = 0;
        self.auto_launch = false;
    }

    fn update_fireworks_show(&mut self, world: &mut World, uptime: f32) {
        if !self.show_active {
            return;
        }

        let show_time = uptime - self.show_start_time;

        while self.show_event_index < self.show_events.len() {
            let event = &self.show_events[self.show_event_index];
            if show_time >= event.time {
                match &event.action {
                    ShowAction::Type(explosion_type) => {
                        self.launch_specific_firework(world, *explosion_type, None, None);
                    }
                    ShowAction::TypeWithColor(explosion_type, color) => {
                        self.launch_specific_firework(world, *explosion_type, Some(*color), None);
                    }
                    ShowAction::Random => {
                        self.launch_firework(world);
                    }
                    ShowAction::Salvo(count) => {
                        for _ in 0..*count {
                            self.launch_firework(world);
                        }
                    }
                    ShowAction::Symmetric(explosion_type, count) => {
                        self.launch_symmetric_fireworks(world, *explosion_type, *count);
                    }
                }
                self.show_event_index += 1;
            } else {
                break;
            }
        }

        if self.show_event_index >= self.show_events.len() {
            let last_event_time = self.show_events.last().map(|e| e.time).unwrap_or(0.0);
            if show_time > last_event_time + 5.0 {
                self.show_active = false;
            }
        }
    }

    fn update_shells(&mut self, world: &mut World, delta_time: f32) {
        let mut explosions_to_spawn: Vec<(Vec3, Vec3, Option<Vec3>, ExplosionType, Entity)> =
            Vec::new();

        for shell in self.shells.iter_mut() {
            if shell.exploded {
                continue;
            }

            shell.fuse_time -= delta_time;

            if let Some(emitter) =
                world.get_mut::<nightshade::render::particles::ParticleEmitter>(shell.entity)
            {
                emitter.position += shell.velocity * delta_time;
                shell.velocity.y -= 9.81 * delta_time;

                if shell.fuse_time <= 0.0 {
                    shell.exploded = true;
                    explosions_to_spawn.push((
                        emitter.position,
                        shell.color,
                        shell.secondary_color,
                        shell.explosion_type,
                        shell.entity,
                    ));
                }
            }
        }

        for (pos, color, secondary, explosion_type, entity) in explosions_to_spawn {
            if explosion_type == ExplosionType::SplittingShell {
                Self::spawn_explosion_for(world, pos, color, secondary, explosion_type);
                self.spawn_splitting_sub_shells(world, pos, color, secondary);
            } else {
                Self::spawn_explosion_for(world, pos, color, secondary, explosion_type);
            }
            if let Some(emitter) =
                world.get_mut::<nightshade::render::particles::ParticleEmitter>(entity)
            {
                emitter.enabled = false;
            }
        }

        self.shells.retain(|shell| {
            if shell.exploded {
                if let Some(emitter) =
                    world.get::<nightshade::render::particles::ParticleEmitter>(shell.entity)
                {
                    return emitter.enabled;
                }
                return false;
            }
            true
        });
    }

    fn spawn_fire_effect(&mut self, world: &mut World, position: Vec3) {
        let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let fire_emitter = ParticleEmitter::fire(position);
        world.set(entity, fire_emitter);

        self.fire_effects.push(FireEffect {
            entity,
            time_remaining: f32::INFINITY,
        });
    }

    fn spawn_smoke_effect(&mut self, world: &mut World, position: Vec3) {
        let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let smoke_emitter = ParticleEmitter::smoke(position);
        world.set(entity, smoke_emitter);

        self.smoke_effects.push(SmokeEffect {
            entity,
            time_remaining: f32::INFINITY,
        });
    }

    fn load_particle_texture(world: &mut World, slot: u32, png_bytes: &[u8]) {
        let img = nightshade::prelude::image::load_from_memory(png_bytes)
            .expect("Failed to decode particle texture")
            .to_rgba8();
        let (width, height) = img.dimensions();
        world
            .resources
            .loading
            .pending_particle_textures
            .push(ParticleTextureUpload {
                slot,
                rgba_data: img.into_raw(),
                width,
                height,
            });
    }

    fn load_all_particle_textures(world: &mut World) {
        let textures: &[(u32, &[u8])] = &[
            (1, include_bytes!("../../../assets/particles/flame_01.png")),
            (2, include_bytes!("../../../assets/particles/flame_02.png")),
            (3, include_bytes!("../../../assets/particles/flame_06.png")),
            (4, include_bytes!("../../../assets/particles/smoke_01.png")),
            (5, include_bytes!("../../../assets/particles/smoke_04.png")),
            (6, include_bytes!("../../../assets/particles/smoke_07.png")),
            (7, include_bytes!("../../../assets/particles/spark_01.png")),
            (8, include_bytes!("../../../assets/particles/spark_05.png")),
            (9, include_bytes!("../../../assets/particles/star_04.png")),
            (10, include_bytes!("../../../assets/particles/star_06.png")),
            (11, include_bytes!("../../../assets/particles/twirl_01.png")),
            (12, include_bytes!("../../../assets/particles/magic_01.png")),
            (13, include_bytes!("../../../assets/particles/magic_04.png")),
            (
                14,
                include_bytes!("../../../assets/particles/circle_01.png"),
            ),
            (
                15,
                include_bytes!("../../../assets/particles/circle_05.png"),
            ),
            (16, include_bytes!("../../../assets/particles/flare_01.png")),
            (17, include_bytes!("../../../assets/particles/light_01.png")),
            (18, include_bytes!("../../../assets/particles/fire_01.png")),
            (
                19,
                include_bytes!("../../../assets/particles/scorch_01.png"),
            ),
            (20, include_bytes!("../../../assets/particles/slash_01.png")),
            (
                21,
                include_bytes!("../../../assets/particles/symbol_01.png"),
            ),
            (22, include_bytes!("../../../assets/particles/trace_05.png")),
            (
                23,
                include_bytes!("../../../assets/particles/muzzle_03.png"),
            ),
            (
                24,
                include_bytes!("../../../assets/particles/window_01.png"),
            ),
        ];
        for &(slot, bytes) in textures {
            Self::load_particle_texture(world, slot, bytes);
        }
    }

    fn spawn_sprite_effect(
        &mut self,
        world: &mut World,
        emitter: ParticleEmitter,
        label: &'static str,
    ) {
        let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        world.set(entity, emitter);
        self.sprite_effects.push(SpriteEffect {
            entity,
            _label: label,
        });
    }

    fn spawn_sprite_label(world: &mut World, position: Vec3, text: &str) {
        spawn_3d_billboard_text_with_properties(
            world,
            text,
            position,
            TextProperties {
                font_size: 24.0,
                color: vec4(1.0, 1.0, 1.0, 0.9),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
    }

    fn spawn_all_sprite_effects(&mut self, world: &mut World) {
        let ground_z = 25.0;
        let spacing = 15.0;
        let start_x = -4.0 * spacing;
        let label_height = 12.0;

        Self::spawn_sprite_label(world, Vec3::new(start_x, label_height, ground_z), "star_04");
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Point,
                position: Vec3::new(start_x, 0.0, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 15.0,
                particle_lifetime_min: 0.8,
                particle_lifetime_max: 1.8,
                initial_velocity_min: 4.0,
                initial_velocity_max: 10.0,
                velocity_spread: 0.5,
                gravity: Vec3::new(0.0, -6.0, 0.0),
                drag: 0.1,
                size_start: 3.0,
                size_end: 0.5,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
                        (0.2, vec4(1.0, 0.95, 0.4, 1.0)),
                        (0.6, vec4(1.0, 0.7, 0.1, 0.9)),
                        (1.0, vec4(0.8, 0.3, 0.0, 0.0)),
                    ],
                },
                emissive_strength: 12.0,
                turbulence_strength: 0.0,
                turbulence_frequency: 0.0,
                texture_index: 9,
                ..Default::default()
            },
            "4-Point Stars",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + spacing, label_height, ground_z),
            "symbol_01",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Fire,
                shape: EmitterShape::Sphere { radius: 0.5 },
                position: Vec3::new(start_x + spacing, 0.5, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 8.0,
                particle_lifetime_min: 2.0,
                particle_lifetime_max: 4.0,
                initial_velocity_min: 0.5,
                initial_velocity_max: 2.0,
                velocity_spread: 0.8,
                gravity: Vec3::new(0.0, 1.5, 0.0),
                drag: 0.3,
                size_start: 2.5,
                size_end: 5.0,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(1.0, 0.3, 0.5, 0.9)),
                        (0.3, vec4(1.0, 0.2, 0.4, 0.8)),
                        (0.7, vec4(0.9, 0.1, 0.3, 0.5)),
                        (1.0, vec4(0.6, 0.0, 0.2, 0.0)),
                    ],
                },
                emissive_strength: 6.0,
                turbulence_strength: 0.8,
                turbulence_frequency: 0.8,
                texture_index: 21,
                ..Default::default()
            },
            "Rising Hearts",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 2.0 * spacing, label_height, ground_z),
            "magic_01",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Fire,
                shape: EmitterShape::Sphere { radius: 1.5 },
                position: Vec3::new(start_x + 2.0 * spacing, 2.0, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 6.0,
                particle_lifetime_min: 2.5,
                particle_lifetime_max: 5.0,
                initial_velocity_min: 0.3,
                initial_velocity_max: 1.5,
                velocity_spread: std::f32::consts::PI,
                gravity: Vec3::new(0.0, 0.3, 0.0),
                drag: 0.2,
                size_start: 3.0,
                size_end: 6.0,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(0.5, 0.2, 1.0, 0.8)),
                        (0.3, vec4(0.7, 0.3, 1.0, 0.7)),
                        (0.6, vec4(0.9, 0.5, 1.0, 0.4)),
                        (1.0, vec4(1.0, 0.8, 1.0, 0.0)),
                    ],
                },
                emissive_strength: 5.0,
                turbulence_strength: 1.0,
                turbulence_frequency: 0.5,
                texture_index: 12,
                ..Default::default()
            },
            "Magic Pentagons",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 3.0 * spacing, label_height, ground_z),
            "star_06",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Point,
                position: Vec3::new(start_x + 3.0 * spacing, 0.0, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 12.0,
                particle_lifetime_min: 1.0,
                particle_lifetime_max: 2.5,
                initial_velocity_min: 3.0,
                initial_velocity_max: 8.0,
                velocity_spread: 0.7,
                gravity: Vec3::new(0.0, -4.0, 0.0),
                drag: 0.15,
                size_start: 3.5,
                size_end: 1.0,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
                        (0.15, vec4(0.6, 0.8, 1.0, 1.0)),
                        (0.5, vec4(0.3, 0.5, 1.0, 0.8)),
                        (1.0, vec4(0.1, 0.2, 0.8, 0.0)),
                    ],
                },
                emissive_strength: 10.0,
                turbulence_strength: 0.0,
                turbulence_frequency: 0.0,
                texture_index: 10,
                ..Default::default()
            },
            "Diamond Stars",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 4.0 * spacing, label_height, ground_z),
            "circle_05",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Smoke,
                shape: EmitterShape::Sphere { radius: 1.0 },
                position: Vec3::new(start_x + 4.0 * spacing, 1.0, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 8.0,
                particle_lifetime_min: 3.0,
                particle_lifetime_max: 6.0,
                initial_velocity_min: 0.2,
                initial_velocity_max: 1.0,
                velocity_spread: std::f32::consts::PI,
                gravity: Vec3::new(0.0, 0.5, 0.0),
                drag: 0.1,
                size_start: 2.0,
                size_end: 5.0,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(0.2, 0.9, 1.0, 0.7)),
                        (0.3, vec4(0.3, 0.8, 1.0, 0.6)),
                        (0.6, vec4(0.1, 0.6, 0.9, 0.4)),
                        (1.0, vec4(0.05, 0.3, 0.7, 0.0)),
                    ],
                },
                emissive_strength: 3.0,
                turbulence_strength: 0.5,
                turbulence_frequency: 0.3,
                texture_index: 15,
                ..Default::default()
            },
            "Ring Bubbles",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 5.0 * spacing, label_height, ground_z),
            "scorch_01",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Fire,
                shape: EmitterShape::Sphere { radius: 0.5 },
                position: Vec3::new(start_x + 5.0 * spacing, 0.5, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 10.0,
                particle_lifetime_min: 0.8,
                particle_lifetime_max: 2.0,
                initial_velocity_min: 1.0,
                initial_velocity_max: 4.0,
                velocity_spread: std::f32::consts::PI,
                gravity: Vec3::new(0.0, -2.0, 0.0),
                drag: 0.2,
                size_start: 4.0,
                size_end: 1.5,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(1.0, 1.0, 0.9, 1.0)),
                        (0.2, vec4(1.0, 0.8, 0.3, 0.9)),
                        (0.5, vec4(1.0, 0.5, 0.1, 0.7)),
                        (1.0, vec4(0.6, 0.15, 0.0, 0.0)),
                    ],
                },
                emissive_strength: 8.0,
                turbulence_strength: 0.3,
                turbulence_frequency: 1.0,
                texture_index: 19,
                ..Default::default()
            },
            "Starburst",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 6.0 * spacing, label_height, ground_z),
            "twirl_01",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Fire,
                shape: EmitterShape::Sphere { radius: 0.5 },
                position: Vec3::new(start_x + 6.0 * spacing, 0.5, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 10.0,
                particle_lifetime_min: 2.0,
                particle_lifetime_max: 4.0,
                initial_velocity_min: 0.5,
                initial_velocity_max: 2.5,
                velocity_spread: std::f32::consts::PI,
                gravity: Vec3::new(0.0, 1.0, 0.0),
                drag: 0.25,
                size_start: 2.5,
                size_end: 4.5,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(0.1, 1.0, 0.6, 0.7)),
                        (0.3, vec4(0.2, 0.9, 0.8, 0.6)),
                        (0.6, vec4(0.3, 0.8, 1.0, 0.4)),
                        (1.0, vec4(0.5, 1.0, 0.9, 0.0)),
                    ],
                },
                emissive_strength: 4.0,
                turbulence_strength: 1.5,
                turbulence_frequency: 0.8,
                texture_index: 11,
                ..Default::default()
            },
            "Crescent Wisps",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 7.0 * spacing, label_height, ground_z),
            "muzzle_03",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Cone {
                    angle: 0.3,
                    height: 0.1,
                },
                position: Vec3::new(start_x + 7.0 * spacing, 0.0, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 12.0,
                particle_lifetime_min: 0.5,
                particle_lifetime_max: 1.5,
                initial_velocity_min: 5.0,
                initial_velocity_max: 12.0,
                velocity_spread: 0.4,
                gravity: Vec3::new(0.0, -8.0, 0.0),
                drag: 0.05,
                size_start: 3.5,
                size_end: 1.0,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
                        (0.15, vec4(1.0, 0.8, 0.2, 1.0)),
                        (0.4, vec4(1.0, 0.5, 0.05, 0.9)),
                        (0.7, vec4(1.0, 0.2, 0.0, 0.6)),
                        (1.0, vec4(0.5, 0.05, 0.0, 0.0)),
                    ],
                },
                emissive_strength: 10.0,
                turbulence_strength: 0.0,
                turbulence_frequency: 0.0,
                texture_index: 23,
                ..Default::default()
            },
            "Flame Tongues",
        );

        Self::spawn_sprite_label(
            world,
            Vec3::new(start_x + 8.0 * spacing, label_height, ground_z),
            "window_01",
        );
        self.spawn_sprite_effect(
            world,
            ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Sphere { radius: 0.3 },
                position: Vec3::new(start_x + 8.0 * spacing, 0.0, ground_z),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 6.0,
                particle_lifetime_min: 2.0,
                particle_lifetime_max: 4.0,
                initial_velocity_min: 2.0,
                initial_velocity_max: 5.0,
                velocity_spread: 0.6,
                gravity: Vec3::new(0.0, -3.0, 0.0),
                drag: 0.15,
                size_start: 4.0,
                size_end: 2.0,
                color_gradient: ColorGradient {
                    colors: vec![
                        (0.0, vec4(1.0, 1.0, 1.0, 0.9)),
                        (0.2, vec4(1.0, 0.9, 0.6, 0.8)),
                        (0.5, vec4(0.9, 0.8, 0.4, 0.6)),
                        (1.0, vec4(0.6, 0.5, 0.2, 0.0)),
                    ],
                },
                emissive_strength: 5.0,
                turbulence_strength: 0.5,
                turbulence_frequency: 0.5,
                texture_index: 24,
                ..Default::default()
            },
            "Window Panes",
        );
    }

    fn cleanup_expired_effects(&mut self, world: &mut World, delta_time: f32) {
        for effect in &mut self.fire_effects {
            if effect.time_remaining.is_finite() {
                effect.time_remaining -= delta_time;
            }
        }

        self.fire_effects.retain(|effect| {
            if !self.show_fire
                || (effect.time_remaining.is_finite() && effect.time_remaining <= 0.0)
            {
                if let Some(emitter) =
                    world.get_mut::<nightshade::render::particles::ParticleEmitter>(effect.entity)
                {
                    emitter.enabled = false;
                }
                return false;
            }
            true
        });

        for effect in &mut self.smoke_effects {
            if effect.time_remaining.is_finite() {
                effect.time_remaining -= delta_time;
            }
        }

        self.smoke_effects.retain(|effect| {
            if !self.show_smoke
                || (effect.time_remaining.is_finite() && effect.time_remaining <= 0.0)
            {
                if let Some(emitter) =
                    world.get_mut::<nightshade::render::particles::ParticleEmitter>(effect.entity)
                {
                    emitter.enabled = false;
                }
                return false;
            }
            true
        });
    }
}

fn panel_content(world: &mut World, panel: Entity) -> Entity {
    widget::<UiPanelData>(world, panel)
        .map(|data| data.content_entity)
        .expect("panel content missing")
}

fn collapsing<F>(tree: &mut UiTreeBuilder, label: &str, default_open: bool, f: F)
where
    F: FnOnce(&mut UiTreeBuilder),
{
    let header = tree.add_collapsing_header(label, default_open);
    let Some(content) =
        widget::<UiCollapsingHeaderData>(tree.world_mut(), header).map(|data| data.content_entity)
    else {
        return;
    };
    tree.in_parent(content, f);
}
