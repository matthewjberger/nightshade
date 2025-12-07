use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(FireworksDemo::default())
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

#[derive(Default)]
struct FireworksDemo {
    shells: Vec<FireworkShell>,
    fire_effects: Vec<FireEffect>,
    smoke_effects: Vec<SmokeEffect>,
    next_launch_time: f32,
    auto_launch: bool,
    launch_interval: f32,
    show_fire: bool,
    show_smoke: bool,
    particle_pass_configured: bool,
    salvo_mode: bool,
    salvo_count: u32,
}

impl State for FireworksDemo {
    fn title(&self) -> &str {
        "Fireworks Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_skybox = false;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.clear_color = [0.01, 0.01, 0.02, 1.0];

        let camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA | PAN_ORBIT_CAMERA,
            1,
        )[0];

        world.set_local_transform(
            camera,
            LocalTransform {
                translation: Vec3::new(0.0, 50.0, 150.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(camera, LocalTransformDirty);
        world.set_global_transform(camera, GlobalTransform::default());
        world.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 60.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(500.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        world.set_pan_orbit_camera(
            camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 40.0, 0.0),
                target_focus: Vec3::new(0.0, 40.0, 0.0),
                radius: 150.0,
                target_radius: 150.0,
                pitch: -0.2,
                target_pitch: -0.2,
                yaw: 0.0,
                target_yaw: 0.0,
                ..Default::default()
            },
        );
        world.resources.active_camera = Some(camera);

        self.auto_launch = true;
        self.launch_interval = 0.3;
        self.show_fire = true;
        self.show_smoke = true;
        self.next_launch_time = 0.0;
        self.salvo_mode = false;
        self.salvo_count = 5;

        if self.show_fire {
            self.spawn_fire_effect(world, Vec3::new(-35.0, 0.0, 0.0));
            self.spawn_fire_effect(world, Vec3::new(35.0, 0.0, 0.0));
            self.spawn_fire_effect(world, Vec3::new(-25.0, 0.0, 10.0));
            self.spawn_fire_effect(world, Vec3::new(25.0, 0.0, 10.0));
        }

        if self.show_smoke {
            self.spawn_smoke_effect(world, Vec3::new(-30.0, 3.0, 5.0));
            self.spawn_smoke_effect(world, Vec3::new(30.0, 3.0, 5.0));
        }

        spawn_hud_text_with_properties(
            world,
            "GPU Particle System - Fireworks Demo\nMouse: Orbit | Scroll: Zoom | Space: Launch | 1-4: Salvo | ESC: Exit",
            HudAnchor::TopCenter,
            Vec2::new(0.0, 20.0),
            TextProperties {
                font_size: 18.0,
                color: Vec4::new(1.0, 1.0, 1.0, 0.9),
                alignment: TextAlignment::Center,
                outline_width: 0.01,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);
        sync_text_meshes_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        let uptime = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        update_particle_emitters(world, delta_time);

        if self.auto_launch && uptime >= self.next_launch_time {
            if self.salvo_mode {
                self.launch_salvo(world);
            } else {
                self.launch_firework(world);
            }
            self.next_launch_time = uptime + self.launch_interval;
        }

        self.update_shells(world, delta_time);
        self.cleanup_expired_effects(world, delta_time);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 1.2);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .write("output", resources.swapchain);

        self.particle_pass_configured = true;
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::Space => {
                self.launch_firework(world);
            }
            KeyCode::Digit1 => {
                for _ in 0..3 {
                    self.launch_firework(world);
                }
            }
            KeyCode::Digit2 => {
                for _ in 0..5 {
                    self.launch_firework(world);
                }
            }
            KeyCode::Digit3 => {
                for _ in 0..10 {
                    self.launch_firework(world);
                }
            }
            KeyCode::Digit4 => {
                self.launch_finale(world);
            }
            KeyCode::Digit5 => {
                self.salvo_mode = !self.salvo_mode;
            }
            KeyCode::KeyA => {
                self.auto_launch = !self.auto_launch;
            }
            KeyCode::KeyF => {
                self.show_fire = !self.show_fire;
                if self.show_fire {
                    self.spawn_fire_effect(world, Vec3::new(-35.0, 0.0, 0.0));
                    self.spawn_fire_effect(world, Vec3::new(35.0, 0.0, 0.0));
                }
            }
            KeyCode::KeyS => {
                self.show_smoke = !self.show_smoke;
                if self.show_smoke {
                    self.spawn_smoke_effect(world, Vec3::new(-30.0, 3.0, 5.0));
                    self.spawn_smoke_effect(world, Vec3::new(30.0, 3.0, 5.0));
                }
            }
            _ => {}
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Fireworks Controls")
            .default_pos([10.0, 60.0])
            .show(ui_context, |ui| {
                ui.checkbox(&mut self.auto_launch, "Auto-launch");
                ui.add(
                    egui::Slider::new(&mut self.launch_interval, 0.1..=2.0)
                        .text("Launch interval (s)"),
                );

                ui.checkbox(&mut self.salvo_mode, "Salvo mode");
                if self.salvo_mode {
                    ui.add(egui::Slider::new(&mut self.salvo_count, 3..=15).text("Salvo count"));
                }

                ui.separator();

                if ui.button("Launch Single").clicked() {
                    self.launch_firework(world);
                }
                if ui.button("Launch Salvo (5)").clicked() {
                    self.launch_salvo(world);
                }
                if ui.button("FINALE!").clicked() {
                    self.launch_finale(world);
                }

                ui.separator();

                ui.label(format!("Active shells: {}", self.shells.len()));
                ui.label(format!("Fire effects: {}", self.fire_effects.len()));
                ui.label(format!("Smoke effects: {}", self.smoke_effects.len()));

                let emitter_count = world.query_entities(PARTICLE_EMITTER).count();
                ui.label(format!("Active emitters: {}", emitter_count));
            });
    }
}

impl FireworksDemo {
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
        ];
        types[rng.random_range(0..types.len())]
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

        let flash_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let flash_emitter = ParticleEmitter::flash_burst(pos);
        world.set_particle_emitter(flash_entity, flash_emitter);

        match explosion_type {
            ExplosionType::Peony => {
                let particle_count: u32 = rng.random_range(500..900);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, particle_count);
                world.set_particle_emitter(entity, emitter);

                if let Some(secondary) = secondary_color {
                    let entity2 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter2 =
                        ParticleEmitter::firework_explosion(pos, secondary, particle_count / 2);
                    emitter2.initial_velocity_min *= 0.7;
                    emitter2.initial_velocity_max *= 0.7;
                    emitter2.size_start *= 0.8;
                    world.set_particle_emitter(entity2, emitter2);
                }

                let spark_count: u32 = rng.random_range(100..200);
                let spark_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let spark_emitter = ParticleEmitter::firework_glitter(pos, spark_count);
                world.set_particle_emitter(spark_entity, spark_emitter);
            }
            ExplosionType::Chrysanthemum => {
                let particle_count: u32 = rng.random_range(700..1200);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_chrysanthemum(pos, color, particle_count);
                world.set_particle_emitter(entity, emitter);

                if let Some(secondary) = secondary_color {
                    let inner_count: u32 = rng.random_range(300..500);
                    let entity2 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter2 =
                        ParticleEmitter::firework_explosion(pos, secondary, inner_count);
                    emitter2.initial_velocity_min *= 0.5;
                    emitter2.initial_velocity_max *= 0.5;
                    world.set_particle_emitter(entity2, emitter2);
                }
            }
            ExplosionType::Willow => {
                let particle_count: u32 = rng.random_range(600..1000);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_willow(pos, color, particle_count);
                world.set_particle_emitter(entity, emitter);

                let tip_count: u32 = rng.random_range(150..300);
                let tip_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let tip_emitter = ParticleEmitter::firework_glitter(pos, tip_count);
                world.set_particle_emitter(tip_entity, tip_emitter);
            }
            ExplosionType::Ring => {
                let particle_count: u32 = rng.random_range(350..600);

                for angle_index in 0..4 {
                    let angle = (angle_index as f32) * std::f32::consts::TAU / 4.0;
                    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::firework_ring(pos, color, particle_count);
                    emitter.direction = Vec3::new(angle.sin(), 0.0, angle.cos());
                    world.set_particle_emitter(entity, emitter);
                }

                if let Some(secondary) = secondary_color {
                    let center_count: u32 = rng.random_range(200..400);
                    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let emitter = ParticleEmitter::firework_explosion(pos, secondary, center_count);
                    world.set_particle_emitter(entity, emitter);
                }
            }
            ExplosionType::MultiBreak => {
                let main_count: u32 = rng.random_range(400..600);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set_particle_emitter(entity, emitter);

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
                    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter =
                        ParticleEmitter::firework_explosion(pos + offset, sub_color, sub_count);
                    emitter.initial_velocity_min *= 0.6;
                    emitter.initial_velocity_max *= 0.6;
                    emitter.size_start *= 0.7;
                    world.set_particle_emitter(entity, emitter);
                }
            }
            ExplosionType::Glitter => {
                let main_count: u32 = rng.random_range(500..800);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set_particle_emitter(entity, emitter);

                let glitter_count: u32 = rng.random_range(300..500);
                let entity2 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter2 = ParticleEmitter::firework_glitter(pos, glitter_count);
                world.set_particle_emitter(entity2, emitter2);

                if let Some(secondary) = secondary_color {
                    let extra_glitter: u32 = rng.random_range(150..300);
                    let entity3 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter3 = ParticleEmitter::firework_glitter(pos, extra_glitter);
                    emitter3.color_gradient = ColorGradient::firework_explosion(secondary);
                    world.set_particle_emitter(entity3, emitter3);
                }
            }
            ExplosionType::Crackle => {
                let main_count: u32 = rng.random_range(400..700);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set_particle_emitter(entity, emitter);

                let crackle_count: u32 = rng.random_range(200..400);
                let entity2 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter2 = ParticleEmitter::firework_crackle(pos, crackle_count);
                world.set_particle_emitter(entity2, emitter2);

                let extra_crackle: u32 = rng.random_range(100..200);
                let entity3 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let mut emitter3 = ParticleEmitter::firework_crackle(pos, extra_crackle);
                emitter3.particle_lifetime_min *= 1.5;
                emitter3.particle_lifetime_max *= 1.5;
                world.set_particle_emitter(entity3, emitter3);
            }
            ExplosionType::Palm => {
                let trunk_count: u32 = rng.random_range(100..180);
                for frond_index in 0..rng.random_range(6..10) {
                    let angle = (frond_index as f32) * std::f32::consts::TAU / 8.0;
                    let tilt = rng.random_range(0.3..0.6);
                    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::palm_explosion(pos, color, trunk_count);
                    emitter.direction =
                        Vec3::new(angle.sin() * tilt, 1.0 - tilt * 0.5, angle.cos() * tilt)
                            .normalize();
                    world.set_particle_emitter(entity, emitter);
                }

                if let Some(secondary) = secondary_color {
                    let tip_count: u32 = rng.random_range(200..400);
                    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter = ParticleEmitter::firework_glitter(pos, tip_count);
                    emitter.color_gradient = ColorGradient::firework_explosion(secondary);
                    world.set_particle_emitter(entity, emitter);
                }
            }
            ExplosionType::Crossette => {
                let main_count: u32 = rng.random_range(400..700);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set_particle_emitter(entity, emitter);

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
                    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let emitter =
                        ParticleEmitter::crossette_burst(pos + offset, burst_color, burst_count);
                    world.set_particle_emitter(entity, emitter);
                }
            }
            ExplosionType::Strobe => {
                let main_count: u32 = rng.random_range(500..800);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter = ParticleEmitter::firework_explosion(pos, color, main_count);
                world.set_particle_emitter(entity, emitter);

                let strobe_count: u32 = rng.random_range(150..300);
                let entity2 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let emitter2 = ParticleEmitter::strobe_effect(pos, strobe_count);
                world.set_particle_emitter(entity2, emitter2);
            }
            ExplosionType::Kamuro => {
                let particle_count: u32 = rng.random_range(1000..1500);
                let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                let mut emitter = ParticleEmitter::firework_willow(pos, color, particle_count);
                emitter.initial_velocity_min = 20.0;
                emitter.initial_velocity_max = 35.0;
                emitter.particle_lifetime_min = 4.0;
                emitter.particle_lifetime_max = 7.0;
                emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
                emitter.drag = 0.08;
                emitter.size_start = 0.25;
                emitter.emissive_strength = 10.0;
                world.set_particle_emitter(entity, emitter);

                if let Some(secondary) = secondary_color {
                    let inner_count: u32 = rng.random_range(400..600);
                    let entity2 = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
                    let mut emitter2 =
                        ParticleEmitter::firework_chrysanthemum(pos, secondary, inner_count);
                    emitter2.initial_velocity_min *= 0.5;
                    emitter2.initial_velocity_max *= 0.5;
                    world.set_particle_emitter(entity2, emitter2);
                }
            }
        }
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

        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let trail_emitter = match shell_type {
            ShellType::Standard => ParticleEmitter::firework_shell(launch_pos, velocity),
            ShellType::Comet => ParticleEmitter::comet_shell(launch_pos, velocity),
        };
        world.set_particle_emitter(entity, trail_emitter);

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

    fn update_shells(&mut self, world: &mut World, delta_time: f32) {
        let mut explosions_to_spawn: Vec<(Vec3, Vec3, Option<Vec3>, ExplosionType, Entity)> =
            Vec::new();

        for shell in self.shells.iter_mut() {
            if shell.exploded {
                continue;
            }

            shell.fuse_time -= delta_time;

            if let Some(emitter) = world.get_particle_emitter_mut(shell.entity) {
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
            Self::spawn_explosion_for(world, pos, color, secondary, explosion_type);
            if let Some(emitter) = world.get_particle_emitter_mut(entity) {
                emitter.enabled = false;
            }
        }

        self.shells.retain(|shell| {
            if shell.exploded {
                if let Some(emitter) = world.get_particle_emitter(shell.entity) {
                    return emitter.enabled;
                }
                return false;
            }
            true
        });
    }

    fn spawn_fire_effect(&mut self, world: &mut World, position: Vec3) {
        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let fire_emitter = ParticleEmitter::fire(position);
        world.set_particle_emitter(entity, fire_emitter);

        self.fire_effects.push(FireEffect {
            entity,
            time_remaining: f32::INFINITY,
        });
    }

    fn spawn_smoke_effect(&mut self, world: &mut World, position: Vec3) {
        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let smoke_emitter = ParticleEmitter::smoke(position);
        world.set_particle_emitter(entity, smoke_emitter);

        self.smoke_effects.push(SmokeEffect {
            entity,
            time_remaining: f32::INFINITY,
        });
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
                if let Some(emitter) = world.get_particle_emitter_mut(effect.entity) {
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
                if let Some(emitter) = world.get_particle_emitter_mut(effect.entity) {
                    emitter.enabled = false;
                }
                return false;
            }
            true
        });
    }
}
