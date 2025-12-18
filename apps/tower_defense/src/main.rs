mod ecs;
mod systems;

use ecs::{GameState, GameWorld, TowerType, UiHandles};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;
use systems::{
    create_path, enemy_movement_system, initialize_grid, input_system, placement_preview_system,
    projectile_movement_system, range_indicator_system, spawn_grid_tiles, tile_hover_system,
    tower_shooting_system, tower_targeting_system, ui_update_system, update_money_popups,
    update_visual_effects, wave_spawning_system,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TowerDefenseECS::new())
}

struct TowerDefenseECS {
    game_world: GameWorld,
}

impl TowerDefenseECS {
    fn new() -> Self {
        Self {
            game_world: GameWorld::default(),
        }
    }
}

impl State for TowerDefenseECS {
    fn title(&self) -> &str {
        "Tower Defense ECS"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;
        world.resources.graphics.show_grid = false;

        spawn_sun(world);

        let main_camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA | PAN_ORBIT_CAMERA,
            1,
        )[0];

        let camera_radius = 16.1;
        let camera_pitch = 0.52;
        world.set_local_transform(
            main_camera,
            LocalTransform {
                translation: Vec3::new(0.0, 8.0, 14.0),
                rotation: nalgebra_glm::quat_angle_axis(-0.6, &nalgebra_glm::vec3(1.0, 0.0, 0.0)),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(main_camera, LocalTransformDirty);
        world.set_global_transform(main_camera, GlobalTransform::default());
        world.set_camera(
            main_camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(500.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        world.set_pan_orbit_camera(
            main_camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 0.0, 0.0),
                target_focus: Vec3::new(0.0, 0.0, 0.0),
                radius: camera_radius,
                target_radius: camera_radius,
                pitch: camera_pitch,
                target_pitch: camera_pitch,
                yaw: 0.0,
                target_yaw: 0.0,
                ..Default::default()
            },
        );
        world.resources.active_camera = Some(main_camera);

        self.game_world.resources.camera_entity = main_camera;

        self.game_world.resources.money = 200;
        self.game_world.resources.lives = 20;
        self.game_world.resources.wave = 0;
        self.game_world.resources.game_state = GameState::WaitingForWave;
        self.game_world.resources.selected_tower_type = TowerType::Basic;
        self.game_world.resources.spawn_timer = 0.0;
        self.game_world.resources.wave_delay = 3.0;
        self.game_world.resources.wave_announce_timer = 0.0;
        self.game_world.resources.game_speed = 1.0;
        self.game_world.resources.current_hp = 20;
        self.game_world.resources.max_hp = 20;
        self.game_world.resources.auto_start_waves = false;

        initialize_grid(&mut self.game_world);
        create_path(&mut self.game_world, world);
        spawn_grid_tiles(&mut self.game_world, world);

        let mut ui_handles = UiHandles {
            money_text: Some(spawn_3d_text_with_properties(
                world,
                &format!("Money: ${}", self.game_world.resources.money),
                nalgebra_glm::vec3(-10.0, 2.0, -8.0),
                TextProperties {
                    font_size: 80.0,
                    color: nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
                    alignment: TextAlignment::Left,
                    outline_width: 0.1,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            lives_text: Some(spawn_3d_text_with_properties(
                world,
                &format!("Lives: {}", self.game_world.resources.lives),
                nalgebra_glm::vec3(-1.0, 2.0, -8.0),
                TextProperties {
                    font_size: 60.0,
                    color: nalgebra_glm::vec4(1.0, 0.2, 0.2, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.1,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            hp_text: Some(spawn_3d_text_with_properties(
                world,
                &format!(
                    "HP: {}/{}",
                    self.game_world.resources.current_hp, self.game_world.resources.max_hp
                ),
                nalgebra_glm::vec3(3.0, 1.0, -8.0),
                TextProperties {
                    font_size: 50.0,
                    color: nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
                    alignment: TextAlignment::Left,
                    outline_width: 0.08,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            wave_text: Some(spawn_3d_text_with_properties(
                world,
                &format!("Wave: {}", self.game_world.resources.wave),
                nalgebra_glm::vec3(10.0, 2.0, -8.0),
                TextProperties {
                    font_size: 80.0,
                    color: nalgebra_glm::vec4(0.2, 0.8, 1.0, 1.0),
                    alignment: TextAlignment::Right,
                    outline_width: 0.1,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            status_text: Some(spawn_3d_text_with_properties(
                world,
                "",
                nalgebra_glm::vec3(0.0, 4.0, 0.0),
                TextProperties {
                    font_size: 160.0,
                    color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.12,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            wave_announce_text: Some(spawn_3d_text_with_properties(
                world,
                "",
                nalgebra_glm::vec3(0.0, 3.0, 0.0),
                TextProperties {
                    font_size: 120.0,
                    color: nalgebra_glm::vec4(1.0, 0.8, 0.0, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.15,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            lives_bar: None,
            lives_bar_bg: None,
            tower_select_texts: Vec::new(),
        };

        for (index, tower_type) in TowerType::all().iter().enumerate() {
            let is_selected = *tower_type == self.game_world.resources.selected_tower_type;
            let base_color = tower_type.color();
            let text_color = if is_selected {
                nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
            } else {
                nalgebra_glm::vec4(base_color.x, base_color.y, base_color.z, 0.7)
            };

            let text = format!(
                "[{}] {} - ${}",
                index + 1,
                tower_type.name(),
                tower_type.cost()
            );

            let entity = spawn_hud_text_with_properties(
                world,
                &text,
                HudAnchor::TopLeft,
                nalgebra_glm::vec2(15.0, 15.0 + (index as f32 * 28.0)),
                TextProperties {
                    font_size: 22.0,
                    color: text_color,
                    alignment: TextAlignment::Left,
                    outline_width: 0.02,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
            ui_handles.tower_select_texts.push(entity);
        }

        if let Some(status_text) = ui_handles.status_text
            && let Some(visibility) = world.get_visibility_mut(status_text)
        {
            visibility.visible = false;
        }

        if let Some(wave_announce_text) = ui_handles.wave_announce_text
            && let Some(visibility) = world.get_visibility_mut(wave_announce_text)
        {
            visibility.visible = false;
        }

        ui_handles.lives_bar_bg = Some(spawn_mesh(
            world,
            "Cube",
            nalgebra_glm::vec3(-1.0, 1.0, -8.0),
            nalgebra_glm::vec3(6.0, 0.3, 0.1),
        ));

        if let Some(bg_entity) = ui_handles.lives_bar_bg {
            let material_name = format!("LivesBarBg_{}", bg_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.2, 0.2, 0.2, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(bg_entity, MaterialRef::new(material_name));
        }

        let total_hp = (self.game_world.resources.lives - 1) * self.game_world.resources.max_hp
            + self.game_world.resources.current_hp;
        let max_total_hp = self.game_world.resources.lives * self.game_world.resources.max_hp;
        let health_percentage = total_hp as f32 / max_total_hp as f32;
        let bar_width = 5.8 * health_percentage;

        ui_handles.lives_bar = Some(spawn_mesh(
            world,
            "Cube",
            nalgebra_glm::vec3(-3.9 + bar_width / 2.0, 1.0, -7.9),
            nalgebra_glm::vec3(bar_width, 0.25, 0.1),
        ));

        if let Some(bar_entity) = ui_handles.lives_bar {
            let material_name = format!("LivesBar_{}", bar_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.0, 1.0, 0.0, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(bar_entity, MaterialRef::new(material_name));
        }

        self.game_world.resources.ui_handles = ui_handles;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        input_system(&mut self.game_world, world);
        tile_hover_system(&mut self.game_world, world);
        placement_preview_system(&mut self.game_world, world);
        range_indicator_system(&mut self.game_world, world);

        wave_spawning_system(&mut self.game_world, world);
        enemy_movement_system(&mut self.game_world, world);
        tower_targeting_system(&mut self.game_world, world);
        tower_shooting_system(&mut self.game_world, world);
        projectile_movement_system(&mut self.game_world, world);

        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
        update_visual_effects(&mut self.game_world, world, delta_time);
        update_money_popups(
            &mut self.game_world,
            world,
            world.resources.window.timing.delta_time,
        );
        ui_update_system(&mut self.game_world, world);

        update_particle_emitters(world, world.resources.window.timing.delta_time);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);
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

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.005);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .write("output", resources.swapchain);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if key == KeyCode::KeyC || key == KeyCode::Home {
            let camera = self.game_world.resources.camera_entity;
            if let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera) {
                pan_orbit.target_focus = Vec3::new(0.0, 0.0, 0.0);
                pan_orbit.target_radius = 16.1;
                pan_orbit.target_pitch = 0.52;
                pan_orbit.target_yaw = 0.0;
            }
        }
    }
}
