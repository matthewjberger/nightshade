use nightshade::ecs::animation::components::{AnimationClip, AnimationProperty};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::terrain::spawn_terrain_with_material;
use nightshade::ecs::text::components::TextProperties;
use nightshade::prelude::*;
use std::collections::HashSet;

const FOX_MODEL: &[u8] = include_bytes!("../../../assets/models/fox.glb");
const FOX_SCALE: f32 = 0.01;

const FIRST_NAMES: &[&str] = &[
    "Rusty", "Maple", "Bramble", "Hazel", "Fern", "Copper", "Willow", "Ash", "Clover", "Sage",
    "Juniper", "Ember", "Moss", "Birch", "Reed", "Flint",
];

const LAST_NAMES: &[&str] = &[
    "Swift", "Whisker", "Paw", "Tail", "Ear", "Nose", "Fur", "Bark", "Leaf", "Stone", "Brook",
    "Glen", "Hill", "Vale", "Frost", "Dawn",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(NavMeshDemo::default())?;
    Ok(())
}

struct FoxData {
    entity: Entity,
    agent_entity: Entity,
    name_entity: Entity,
    full_name: String,
    name_color: Vec4,
    was_moving: bool,
    current_rotation: f32,
    current_animation: Option<usize>,
}

#[derive(Default)]
struct AnimationIndices {
    survey: Option<usize>,
    walk: Option<usize>,
    run: Option<usize>,
}

#[derive(Clone)]
struct TerrainConfig {
    width: f32,
    depth: f32,
    resolution_x: u32,
    resolution_z: u32,
    height_scale: f32,
    seed: u32,
    frequency: f64,
    octaves: usize,
    lacunarity: f64,
    persistence: f64,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            width: 80.0,
            depth: 80.0,
            resolution_x: 64,
            resolution_z: 64,
            height_scale: 3.0,
            seed: 42,
            frequency: 0.02,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.45,
        }
    }
}

impl TerrainConfig {
    fn to_nightshade_config(&self) -> nightshade::ecs::terrain::TerrainConfig {
        nightshade::ecs::terrain::TerrainConfig {
            width: self.width,
            depth: self.depth,
            resolution_x: self.resolution_x,
            resolution_z: self.resolution_z,
            height_scale: self.height_scale,
            noise: nightshade::ecs::terrain::NoiseConfig {
                seed: self.seed,
                frequency: self.frequency,
                octaves: self.octaves,
                lacunarity: self.lacunarity,
                persistence: self.persistence,
                noise_type: nightshade::ecs::terrain::NoiseType::Perlin,
            },
            uv_scale: [10.0, 10.0],
        }
    }
}

struct TreeObstacle {
    position: Vec3,
    radius: f32,
    height: f32,
}

struct NavMeshDemo {
    camera_entity: Option<Entity>,
    foxes: Vec<FoxData>,
    tree_count: usize,
    tree_obstacles: Vec<TreeObstacle>,
    show_navmesh: bool,
    wander_mode: bool,
    follow_mode: bool,
    followed_fox_index: Option<usize>,
    used_names: HashSet<String>,
    agent_speed: f32,
    fox_loaded: bool,
    animation_indices: AnimationIndices,
    filtered_animations: Vec<AnimationClip>,
    skins: Vec<nightshade::ecs::prefab::GltfSkin>,
    prefab: Option<nightshade::ecs::prefab::Prefab>,
    terrain_config: TerrainConfig,
}

impl Default for NavMeshDemo {
    fn default() -> Self {
        Self {
            camera_entity: None,
            foxes: Vec::new(),
            tree_count: 0,
            tree_obstacles: Vec::new(),
            show_navmesh: false,
            wander_mode: false,
            follow_mode: false,
            followed_fox_index: None,
            used_names: HashSet::new(),
            agent_speed: 5.0,
            fox_loaded: false,
            animation_indices: AnimationIndices::default(),
            filtered_animations: Vec::new(),
            skins: Vec::new(),
            prefab: None,
            terrain_config: TerrainConfig::default(),
        }
    }
}

impl State for NavMeshDemo {
    fn title(&self) -> &str {
        "NavMesh Pathfinding Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;

        spawn_sun(world);

        let focus = nalgebra_glm::vec3(0.0, 2.0, 0.0);
        let camera =
            spawn_pan_orbit_camera(world, focus, 50.0, 0.0, 0.6, "Main Camera".to_string());
        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);

        self.spawn_terrain(world);
        self.spawn_trees(world);
        self.build_navmesh(world);
        self.load_fox_model(world);
        self.spawn_initial_foxes(world);

        set_navmesh_debug_draw(world, self.show_navmesh);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        if !self.follow_mode {
            pan_orbit_camera_system(world);
        }
        run_navmesh_systems(world);

        self.handle_mouse_input(world);
        self.update_wander_mode(world);
        self.sync_foxes_to_agents(world);
        self.update_fox_animations(world);
        self.update_follow_camera(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("NavMesh Demo Controls").show(ui_context, |ui| {
            ui.label("Left-click anywhere to move all foxes");
            ui.label("Right-click to spawn a new fox");
            ui.separator();

            ui.label("Visualization:");
            if ui
                .checkbox(&mut self.show_navmesh, "Show NavMesh")
                .changed()
            {
                set_navmesh_debug_draw(world, self.show_navmesh);
            }

            ui.separator();

            ui.label("Behavior:");
            ui.checkbox(&mut self.wander_mode, "Wander Mode");
            if ui.checkbox(&mut self.follow_mode, "Follow Mode").changed() {
                if self.follow_mode && !self.foxes.is_empty() {
                    use rand::Rng;
                    self.followed_fox_index = Some(rand::rng().random_range(0..self.foxes.len()));
                } else {
                    self.followed_fox_index = None;
                }
            }
            if let Some(index) = self.followed_fox_index
                && let Some(fox) = self.foxes.get(index)
            {
                ui.horizontal(|ui| {
                    ui.label("Following:");
                    ui.colored_label(
                        egui::Color32::from_rgba_unmultiplied(
                            (fox.name_color.x * 255.0) as u8,
                            (fox.name_color.y * 255.0) as u8,
                            (fox.name_color.z * 255.0) as u8,
                            255,
                        ),
                        &fox.full_name,
                    );
                });
            }

            ui.separator();

            ui.label("Pathfinding:");
            let current_algorithm = world.resources.navmesh.algorithm;
            egui::ComboBox::from_label("Algorithm")
                .selected_text(current_algorithm.name())
                .show_ui(ui, |ui| {
                    for algorithm in PathfindingAlgorithm::all() {
                        if ui
                            .selectable_label(current_algorithm == *algorithm, algorithm.name())
                            .clicked()
                        {
                            world.resources.navmesh.algorithm = *algorithm;
                        }
                    }
                });

            ui.separator();

            ui.label("Fox Settings:");
            if ui
                .add(egui::Slider::new(&mut self.agent_speed, 1.0..=15.0).text("Speed"))
                .changed()
            {
                for fox in &self.foxes {
                    set_agent_speed(world, fox.agent_entity, self.agent_speed);
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Spawn Fox").clicked() {
                    let x = (rand::random::<f32>() - 0.5) * 30.0;
                    let z = (rand::random::<f32>() - 0.5) * 30.0;
                    let y = self.sample_height(x, z);
                    self.spawn_fox(world, nalgebra_glm::vec3(x, y, z));
                }

                if ui.button("Clear Foxes").clicked() {
                    for fox in self.foxes.drain(..) {
                        world.despawn_entities(&[fox.agent_entity, fox.name_entity]);
                        despawn_recursive_immediate(world, fox.entity);
                    }
                    self.used_names.clear();
                    self.followed_fox_index = None;
                    self.follow_mode = false;
                }
            });

            ui.separator();

            ui.label("Stats:");
            ui.label(format!("Foxes: {}", self.foxes.len()));
            ui.label(format!("Trees: {}", self.tree_count));
            ui.label(format!(
                "NavMesh Triangles: {}",
                world.resources.navmesh.triangles.len()
            ));
            ui.label(format!(
                "FPS: {:.1}",
                world.resources.window.timing.frames_per_second
            ));
        });
    }
}

impl NavMeshDemo {
    fn sample_height(&self, x: f32, z: f32) -> f32 {
        nightshade::ecs::terrain::sample_terrain_height(
            x,
            z,
            &self.terrain_config.to_nightshade_config(),
        )
    }

    fn spawn_terrain(&self, world: &mut World) {
        let grass_material = Material {
            base_color: [0.25, 0.45, 0.2, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        };

        spawn_terrain_with_material(
            world,
            self.terrain_config.to_nightshade_config(),
            Vec3::zeros(),
            grass_material,
        );
    }

    fn spawn_trees(&mut self, world: &mut World) {
        let num_trees = 40;

        for index in 0..num_trees {
            let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
            let theta = index as f32 * golden_angle;
            let r = (index as f32 / num_trees as f32).sqrt() * 30.0 + 8.0;
            let x = theta.cos() * r;
            let z = theta.sin() * r;

            let dist_from_center = (x * x + z * z).sqrt();
            if dist_from_center < 6.0 {
                continue;
            }

            let trunk_height = 0.6 + (index as f32 * 0.02) % 0.3;
            let trunk_radius = 0.1 + (index as f32 * 0.002) % 0.06;
            let tree_scale = 0.7 + (index as f32 * 0.015) % 0.5;

            self.tree_count += 1;

            let terrain_y = self.sample_height(x, z);

            self.tree_obstacles.push(TreeObstacle {
                position: Vec3::new(x, terrain_y, z),
                radius: trunk_radius + 0.3,
                height: trunk_height + 2.0,
            });

            let trunk_material_name = format!("TreeTrunk_{}", index);
            material_registry_insert(
                &mut world.resources.material_registry,
                trunk_material_name.clone(),
                Material {
                    base_color: [0.35, 0.22, 0.12, 1.0],
                    roughness: 0.95,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&trunk_material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }

            let trunk = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.set_local_transform(
                trunk,
                LocalTransform {
                    translation: Vec3::new(x, terrain_y + trunk_height / 2.0, z),
                    rotation: Quat::identity(),
                    scale: Vec3::new(trunk_radius * 2.0, trunk_height, trunk_radius * 2.0),
                },
            );
            world.set_render_mesh(trunk, RenderMesh::new("Cube"));
            world.set_material_ref(trunk, MaterialRef::new(trunk_material_name));
            world.set_casts_shadow(trunk, CastsShadow);

            let green_variation = (index as f32 * 0.02) % 0.12;
            let tier_radii = [2.0 * tree_scale, 1.5 * tree_scale, 0.9 * tree_scale];
            let tier_heights = [1.4 * tree_scale, 1.2 * tree_scale, 1.0 * tree_scale];
            let tier_y_offsets = [0.0, 0.8 * tree_scale, 1.5 * tree_scale];

            for tier in 0..3 {
                let radius = tier_radii[tier];
                let height = tier_heights[tier];
                let y_pos = terrain_y + trunk_height + tier_y_offsets[tier] + height / 2.0;

                let cone_material_name = format!("TreeCone_{}_{}", index, tier);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    cone_material_name.clone(),
                    Material {
                        base_color: [0.1, 0.38 + green_variation, 0.08, 1.0],
                        roughness: 0.95,
                        metallic: 0.0,
                        ..Default::default()
                    },
                );
                if let Some(&mat_index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&cone_material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(mat_index);
                }

                let cone = world.spawn_entities(
                    LOCAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | GLOBAL_TRANSFORM
                        | RENDER_MESH
                        | MATERIAL_REF
                        | CASTS_SHADOW,
                    1,
                )[0];
                world.set_local_transform(
                    cone,
                    LocalTransform {
                        translation: Vec3::new(x, y_pos, z),
                        rotation: Quat::identity(),
                        scale: Vec3::new(radius, height, radius),
                    },
                );
                world.set_render_mesh(cone, RenderMesh::new("Cone"));
                world.set_material_ref(cone, MaterialRef::new(cone_material_name));
                world.set_casts_shadow(cone, CastsShadow);
            }
        }
    }

    fn build_navmesh(&self, world: &mut World) {
        let terrain_config = self.terrain_config.to_nightshade_config();
        let terrain_result = generate_terrain_mesh(&terrain_config);

        let mut vertices: Vec<[f32; 3]> = terrain_result
            .mesh
            .vertices
            .iter()
            .map(|v| v.position)
            .collect();

        let mut indices: Vec<[u32; 3]> = terrain_result
            .mesh
            .indices
            .chunks(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect();

        for obstacle in &self.tree_obstacles {
            Self::add_box_obstacle(
                &mut vertices,
                &mut indices,
                obstacle.position,
                obstacle.radius,
                obstacle.height,
            );
        }

        let config = RecastNavMeshConfig {
            agent_radius: 0.5,
            agent_height: 1.8,
            cell_size_fraction: 6.0,
            cell_height_fraction: 12.0,
            walkable_climb: 0.5,
            walkable_slope_angle: std::f32::consts::FRAC_PI_4,
            min_region_size: 4,
            merge_region_size: 10,
            max_simplification_error: 0.5,
            edge_max_len_factor: 6,
            max_vertices_per_polygon: 6,
            detail_sample_dist: 1.0,
            detail_sample_max_error: 0.25,
        };

        match generate_navmesh_recast(&vertices, &indices, &config) {
            Some(navmesh) => {
                tracing::info!(
                    "Built navmesh: {} triangles, {} connections",
                    navmesh.triangles.len(),
                    navmesh.adjacency.values().map(|v| v.len()).sum::<usize>()
                );
                world.resources.navmesh = navmesh;
            }
            None => {
                tracing::warn!("Failed to generate navmesh from terrain");
            }
        }
    }

    fn add_box_obstacle(
        vertices: &mut Vec<[f32; 3]>,
        indices: &mut Vec<[u32; 3]>,
        position: Vec3,
        radius: f32,
        height: f32,
    ) {
        let base_index = vertices.len() as u32;

        let min_x = position.x - radius;
        let max_x = position.x + radius;
        let min_y = position.y;
        let max_y = position.y + height;
        let min_z = position.z - radius;
        let max_z = position.z + radius;

        vertices.push([min_x, min_y, min_z]);
        vertices.push([max_x, min_y, min_z]);
        vertices.push([max_x, min_y, max_z]);
        vertices.push([min_x, min_y, max_z]);
        vertices.push([min_x, max_y, min_z]);
        vertices.push([max_x, max_y, min_z]);
        vertices.push([max_x, max_y, max_z]);
        vertices.push([min_x, max_y, max_z]);

        indices.push([base_index, base_index + 2, base_index + 1]);
        indices.push([base_index, base_index + 3, base_index + 2]);

        indices.push([base_index + 4, base_index + 5, base_index + 6]);
        indices.push([base_index + 4, base_index + 6, base_index + 7]);

        indices.push([base_index, base_index + 1, base_index + 5]);
        indices.push([base_index, base_index + 5, base_index + 4]);

        indices.push([base_index + 2, base_index + 3, base_index + 7]);
        indices.push([base_index + 2, base_index + 7, base_index + 6]);

        indices.push([base_index, base_index + 4, base_index + 7]);
        indices.push([base_index, base_index + 7, base_index + 3]);

        indices.push([base_index + 1, base_index + 2, base_index + 6]);
        indices.push([base_index + 1, base_index + 6, base_index + 5]);
    }

    fn load_fox_model(&mut self, world: &mut World) {
        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(FOX_MODEL);

        match load_result {
            Ok(result) => {
                for (index, anim) in result.animations.iter().enumerate() {
                    let name_lower = anim.name.to_lowercase();
                    if name_lower.contains("survey") {
                        self.animation_indices.survey = Some(index);
                    } else if name_lower.contains("walk") {
                        self.animation_indices.walk = Some(index);
                    } else if name_lower.contains("run") {
                        self.animation_indices.run = Some(index);
                    }
                }

                for (name, (rgba_data, width, height)) in result.textures {
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                for (name, mesh) in result.meshes {
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                let root_bone_indices: std::collections::HashSet<usize> = [0, 1, 2, 3].into();
                self.filtered_animations = result
                    .animations
                    .iter()
                    .map(|clip| AnimationClip {
                        name: clip.name.clone(),
                        duration: clip.duration,
                        channels: clip
                            .channels
                            .iter()
                            .filter(|channel| {
                                if channel.target_property == AnimationProperty::Translation {
                                    return false;
                                }
                                if root_bone_indices.contains(&channel.target_node)
                                    && channel.target_property == AnimationProperty::Rotation
                                {
                                    return false;
                                }
                                true
                            })
                            .cloned()
                            .collect(),
                    })
                    .collect();

                self.skins = result.skins;
                self.prefab = result.prefabs.into_iter().next();
                self.fox_loaded = true;
            }
            Err(e) => {
                tracing::error!("Failed to load fox model: {}", e);
            }
        }
    }

    fn spawn_initial_foxes(&mut self, world: &mut World) {
        let positions = vec![(-5.0_f32, -5.0_f32), (5.0, -5.0), (-5.0, 5.0)];

        for (x, z) in positions {
            let y = self.sample_height(x, z);
            self.spawn_fox(world, nalgebra_glm::vec3(x, y, z));
        }
    }

    fn spawn_fox(&mut self, world: &mut World, position: nalgebra_glm::Vec3) {
        if !self.fox_loaded {
            return;
        }

        let Some(prefab) = &self.prefab else {
            return;
        };

        let agent_entity = world.spawn_entities(
            LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | NAVMESH_AGENT,
            1,
        )[0];

        if let Some(transform) = world.get_local_transform_mut(agent_entity) {
            transform.translation = position;
        }

        if let Some(agent) = world.get_navmesh_agent_mut(agent_entity) {
            agent.movement_speed = self.agent_speed;
        }

        let fox_entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
            world,
            prefab,
            &self.filtered_animations,
            &self.skins,
            Vec3::zeros(),
        );

        if let Some(transform) = world.get_local_transform_mut(fox_entity) {
            transform.translation = position;
            transform.scale = Vec3::new(FOX_SCALE, FOX_SCALE, FOX_SCALE);
        }
        world.mark_local_transform_dirty(fox_entity);

        let initial_animation = self.animation_indices.survey;
        if let Some(player) = world.get_animation_player_mut(fox_entity) {
            if let Some(survey_index) = initial_animation {
                player.play(survey_index);
                player.speed = 0.5;
            } else if !player.clips.is_empty() {
                player.play(0);
                player.speed = 0.5;
            }
        }

        let (full_name, name_color) = self.generate_fox_name();
        let name_position = position + nalgebra_glm::vec3(0.0, 1.5, 0.0);
        let name_entity = spawn_3d_billboard_text_with_properties(
            world,
            &full_name,
            name_position,
            TextProperties {
                font_size: 24.0,
                color: name_color,
                outline_width: 0.3,
                outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        self.foxes.push(FoxData {
            entity: fox_entity,
            agent_entity,
            name_entity,
            full_name,
            name_color,
            was_moving: false,
            current_rotation: 0.0,
            current_animation: initial_animation,
        });
    }

    fn generate_fox_name(&mut self) -> (String, Vec4) {
        use rand::Rng;

        let max_combinations = FIRST_NAMES.len() * LAST_NAMES.len();
        if self.used_names.len() >= max_combinations {
            self.used_names.clear();
        }

        let mut rng = rand::rng();
        loop {
            let first_index = rng.random_range(0..FIRST_NAMES.len());
            let last_index = rng.random_range(0..LAST_NAMES.len());
            let full_name = format!("{} {}", FIRST_NAMES[first_index], LAST_NAMES[last_index]);

            if !self.used_names.contains(&full_name) {
                self.used_names.insert(full_name.clone());
                let color = Self::name_to_color(&full_name);
                return (full_name, color);
            }
        }
    }

    fn name_to_color(name: &str) -> Vec4 {
        let mut hash: u32 = 0;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }

        let hue = (hash % 360) as f32;
        let saturation = 0.7;
        let value = 0.9;

        let c = value * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = value - c;

        let (r, g, b) = match (hue / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        nalgebra_glm::vec4(r + m, g + m, b + m, 1.0)
    }

    fn sync_foxes_to_agents(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        let terrain_config = self.terrain_config.to_nightshade_config();

        for fox in &mut self.foxes {
            let agent_pos = world
                .get_local_transform(fox.agent_entity)
                .map(|t| t.translation)
                .unwrap_or(Vec3::zeros());

            let terrain_y = nightshade::ecs::terrain::sample_terrain_height(
                agent_pos.x,
                agent_pos.z,
                &terrain_config,
            );
            let fox_pos = Vec3::new(agent_pos.x, terrain_y, agent_pos.z);

            let agent = world.get_navmesh_agent(fox.agent_entity);
            let is_moving = agent
                .map(|a| a.state == NavMeshAgentState::Moving)
                .unwrap_or(false);

            if is_moving
                && let Some(agent) = agent
                && let Some(waypoint) = agent.current_waypoint()
            {
                let direction = waypoint - agent_pos;
                if direction.x.abs() > 0.01 || direction.z.abs() > 0.01 {
                    let target_rotation = direction.x.atan2(direction.z);
                    let rotation_speed = 10.0;

                    let mut angle_diff = target_rotation - fox.current_rotation;
                    while angle_diff > std::f32::consts::PI {
                        angle_diff -= 2.0 * std::f32::consts::PI;
                    }
                    while angle_diff < -std::f32::consts::PI {
                        angle_diff += 2.0 * std::f32::consts::PI;
                    }
                    fox.current_rotation += angle_diff * rotation_speed * delta_time;
                }
            }

            fox.was_moving = is_moving;

            if let Some(transform) = world.get_local_transform_mut(fox.entity) {
                transform.translation = fox_pos;
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(fox.current_rotation, &Vec3::y());
            }
            world.mark_local_transform_dirty(fox.entity);

            if let Some(transform) = world.get_local_transform_mut(fox.name_entity) {
                transform.translation = fox_pos + nalgebra_glm::vec3(0.0, 1.5, 0.0);
            }
            world.mark_local_transform_dirty(fox.name_entity);
        }
    }

    fn update_fox_animations(&mut self, world: &mut World) {
        const RUN_DISTANCE_THRESHOLD: f32 = 8.0;
        const WALK_DISTANCE_THRESHOLD: f32 = 2.0;

        for fox in &mut self.foxes {
            let agent = world.get_navmesh_agent(fox.agent_entity);
            let (is_moving, distance) = agent
                .map(|a| {
                    (
                        a.state == NavMeshAgentState::Moving,
                        a.distance_to_destination,
                    )
                })
                .unwrap_or((false, 0.0));

            let (target_animation, target_speed) = if !is_moving {
                (self.animation_indices.survey, 0.5)
            } else if distance > RUN_DISTANCE_THRESHOLD {
                (self.animation_indices.run, 1.0)
            } else if distance > WALK_DISTANCE_THRESHOLD {
                (self.animation_indices.walk, 1.5)
            } else {
                (self.animation_indices.walk, 1.0)
            };

            if target_animation != fox.current_animation {
                if let Some(anim_index) = target_animation
                    && let Some(player) = world.get_animation_player_mut(fox.entity)
                {
                    player.blend_to(anim_index, 0.3);
                    player.speed = target_speed;
                    fox.current_animation = Some(anim_index);
                }
            } else if let Some(player) = world.get_animation_player_mut(fox.entity) {
                player.speed = target_speed;
            }
        }
    }

    fn update_wander_mode(&self, world: &mut World) {
        if !self.wander_mode {
            return;
        }

        let terrain_config = self.terrain_config.to_nightshade_config();

        for fox in &self.foxes {
            let agent = world.get_navmesh_agent(fox.agent_entity);
            let needs_destination = agent
                .map(|a| {
                    a.state == NavMeshAgentState::Idle
                        || a.state == NavMeshAgentState::Arrived
                        || a.state == NavMeshAgentState::NoPath
                })
                .unwrap_or(false);

            if needs_destination {
                let x = (rand::random::<f32>() - 0.5) * 60.0;
                let z = (rand::random::<f32>() - 0.5) * 60.0;
                let y = nightshade::ecs::terrain::sample_terrain_height(x, z, &terrain_config);
                let target = nalgebra_glm::vec3(x, y, z);
                set_agent_destination(world, fox.agent_entity, target);
            }
        }
    }

    fn update_follow_camera(&mut self, world: &mut World) {
        if !self.follow_mode {
            return;
        }

        let Some(followed_index) = self.followed_fox_index else {
            return;
        };

        let Some(fox) = self.foxes.get(followed_index) else {
            self.followed_fox_index = None;
            self.follow_mode = false;
            return;
        };

        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let fox_pos = world
            .get_local_transform(fox.entity)
            .map(|t| t.translation)
            .unwrap_or(Vec3::zeros());

        let camera_offset = nalgebra_glm::vec3(0.0, 8.0, 12.0);
        let rotated_offset =
            nalgebra_glm::rotate_vec3(&camera_offset, fox.current_rotation, &Vec3::y());
        let camera_pos = fox_pos + rotated_offset;
        let look_target = fox_pos + nalgebra_glm::vec3(0.0, 1.0, 0.0);

        if let Some(transform) = world.get_local_transform_mut(camera_entity) {
            transform.translation = camera_pos;
            let direction = nalgebra_glm::normalize(&(look_target - camera_pos));
            let up = Vec3::y();
            let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up));
            let corrected_up = nalgebra_glm::cross(&right, &direction);
            let rotation_matrix = nalgebra_glm::mat3(
                right.x,
                corrected_up.x,
                -direction.x,
                right.y,
                corrected_up.y,
                -direction.y,
                right.z,
                corrected_up.z,
                -direction.z,
            );
            transform.rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
        }
        world.mark_local_transform_dirty(camera_entity);
    }

    fn handle_mouse_input(&mut self, world: &mut World) {
        let left_clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_PRESSED);
        let right_clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::RIGHT_JUST_PRESSED);

        if !left_clicked && !right_clicked {
            return;
        }

        let mouse_position = world.resources.input.mouse.position;

        let ground_position = get_ground_position_from_screen(world, mouse_position, 0.0);

        if let Some(ground_pos) = ground_position {
            if ground_pos.x.abs() > 35.0 || ground_pos.z.abs() > 35.0 {
                return;
            }

            let terrain_y = self.sample_height(ground_pos.x, ground_pos.z);
            let target_pos = nalgebra_glm::vec3(ground_pos.x, terrain_y, ground_pos.z);

            if left_clicked {
                for fox in &self.foxes {
                    set_agent_destination(world, fox.agent_entity, target_pos);
                }
            } else if right_clicked {
                self.spawn_fox(world, target_pos);
            }
        }
    }
}
