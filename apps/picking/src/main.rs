use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PickingDemoState::default())?;
    Ok(())
}

#[derive(Default)]
struct PickingDemoState {
    meshes: Vec<(Entity, String)>,
    hovered_entity: Option<Entity>,
    original_scales: HashMap<Entity, Vec3>,
    original_colors: HashMap<Entity, Vec4>,
    spin_speed: f32,
    camera_entity: Option<Entity>,
    debug_ray_direction: Option<Vec3>,
    debug_volumes: Vec<Entity>,
    debug_obb_volumes: Vec<Entity>,
    show_debug_volumes: bool,
    show_obb_volumes: bool,
    bounding_radii: HashMap<Entity, f32>,
    max_pick_distance: Option<f32>,
    orthographic_mode: bool,
}

fn create_obb_wireframe(obb: &OrientedBoundingBox, color: Vec4) -> Vec<Line> {
    let mut lines = Vec::new();
    let corners = obb.get_corners();

    let edges = [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    for &(start_idx, end_idx) in &edges {
        lines.push(Line {
            start: corners[start_idx],
            end: corners[end_idx],
            color,
        });
    }

    lines
}

fn create_sphere_wireframe(radius: f32, color: Vec4, segments: u32) -> Vec<Line> {
    let mut lines = Vec::new();

    for i in 0..segments {
        let phi = std::f32::consts::PI * (i as f32 / segments as f32);
        let y = radius * phi.cos();
        let circle_radius = radius * phi.sin();

        for j in 0..segments {
            let theta1 = 2.0 * std::f32::consts::PI * (j as f32 / segments as f32);
            let theta2 = 2.0 * std::f32::consts::PI * ((j + 1) as f32 / segments as f32);

            let x1 = circle_radius * theta1.cos();
            let z1 = circle_radius * theta1.sin();
            let x2 = circle_radius * theta2.cos();
            let z2 = circle_radius * theta2.sin();

            lines.push(Line {
                start: nalgebra_glm::vec3(x1, y, z1),
                end: nalgebra_glm::vec3(x2, y, z2),
                color,
            });
        }
    }

    for i in 0..segments {
        let theta = 2.0 * std::f32::consts::PI * (i as f32 / segments as f32);
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        for j in 0..segments {
            let phi1 = std::f32::consts::PI * (j as f32 / segments as f32);
            let phi2 = std::f32::consts::PI * ((j + 1) as f32 / segments as f32);

            let x1 = radius * phi1.sin() * cos_theta;
            let y1 = radius * phi1.cos();
            let z1 = radius * phi1.sin() * sin_theta;

            let x2 = radius * phi2.sin() * cos_theta;
            let y2 = radius * phi2.cos();
            let z2 = radius * phi2.sin() * sin_theta;

            lines.push(Line {
                start: nalgebra_glm::vec3(x1, y1, z1),
                end: nalgebra_glm::vec3(x2, y2, z2),
                color,
            });
        }
    }

    lines
}

impl State for PickingDemoState {
    fn title(&self) -> &str {
        "3D Picking Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        spawn_sun(world);

        self.spin_speed = 1.0;
        self.show_debug_volumes = true;
        self.show_obb_volumes = true;
        self.max_pick_distance = None;
        self.orthographic_mode = false;

        let mesh_types = vec![
            (
                "Cube",
                "Cube",
                nalgebra_glm::vec3(-8.0, 0.0, 0.0),
                nalgebra_glm::vec4(0.8, 0.2, 0.2, 1.0),
            ),
            (
                "Sphere",
                "Sphere",
                nalgebra_glm::vec3(-4.0, 0.0, 0.0),
                nalgebra_glm::vec4(0.2, 0.8, 0.2, 1.0),
            ),
            (
                "Cylinder",
                "Cylinder",
                nalgebra_glm::vec3(0.0, 0.0, 0.0),
                nalgebra_glm::vec4(0.2, 0.2, 0.8, 1.0),
            ),
            (
                "Torus",
                "Torus",
                nalgebra_glm::vec3(4.0, 0.0, 0.0),
                nalgebra_glm::vec4(0.2, 0.8, 0.8, 1.0),
            ),
            (
                "Cone",
                "Cone",
                nalgebra_glm::vec3(8.0, 0.0, 0.0),
                nalgebra_glm::vec4(0.8, 0.2, 0.8, 1.0),
            ),
        ];

        for (mesh_type, name, position, color) in mesh_types {
            let entity = spawn_mesh(
                world,
                mesh_type,
                position,
                nalgebra_glm::vec3(1.0, 1.0, 1.0),
            );

            let material_name = format!("{}_{}", name, entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: color.into(),
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
            };
            world.set_material_ref(entity, MaterialRef::new(material_name));

            self.original_scales
                .insert(entity, nalgebra_glm::vec3(1.0, 1.0, 1.0));
            self.original_colors.insert(entity, color);
            self.meshes.push((entity, name.to_string()));

            let bounding_radius = if let Some(bv) = world.get_bounding_volume(entity) {
                bv.sphere_radius
            } else {
                1.0
            };
            self.bounding_radii.insert(entity, bounding_radius);

            let debug_color = nalgebra_glm::vec4(0.0, 1.0, 1.0, 1.0);
            let sphere_lines = create_sphere_wireframe(bounding_radius, debug_color, 16);

            let debug_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | LINES
                    | VISIBILITY,
                1,
            )[0];

            world.set_name(debug_entity, Name(format!("{} Debug Volume", name)));
            world.set_local_transform(
                debug_entity,
                LocalTransform {
                    translation: position,
                    scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
                    ..Default::default()
                },
            );
            world.set_global_transform(debug_entity, GlobalTransform::default());
            world.set_local_transform_dirty(debug_entity, LocalTransformDirty);
            world.set_lines(debug_entity, Lines::new(sphere_lines));
            world.set_visibility(debug_entity, Visibility { visible: true });

            self.debug_volumes.push(debug_entity);

            let obb_debug_entity = if let Some(bv) = world.get_bounding_volume(entity) {
                let entity_transform = LocalTransform {
                    translation: position,
                    scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
                    rotation: nalgebra_glm::quat_identity(),
                };
                let transform_matrix = entity_transform.as_matrix();
                let world_obb = bv.obb.transform(&transform_matrix);

                let obb_color = nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0);
                let obb_lines = create_obb_wireframe(&world_obb, obb_color);

                let obb_entity = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | LINES
                        | VISIBILITY,
                    1,
                )[0];

                world.set_name(obb_entity, Name(format!("{} OBB Debug", name)));
                world.set_local_transform(obb_entity, LocalTransform::default());
                world.set_global_transform(obb_entity, GlobalTransform::default());
                world.set_local_transform_dirty(obb_entity, LocalTransformDirty);
                world.set_lines(obb_entity, Lines::new(obb_lines));
                world.set_visibility(obb_entity, Visibility { visible: true });

                obb_entity
            } else {
                let obb_entity = world.spawn_entities(NAME, 1)[0];
                world.set_name(obb_entity, Name(format!("{} OBB Debug", name)));
                obb_entity
            };

            self.debug_obb_volumes.push(obb_debug_entity);
        }

        let camera_entity = spawn_pan_orbit_camera(
            world,
            nalgebra_glm::vec3(0.0, 0.0, 0.0),
            12.0,
            0.0,
            0.4,
            "Picking Demo Camera".to_string(),
        );

        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);

        for (i, debug_entity) in self.debug_volumes.iter().enumerate() {
            if let Some(visible) = world.get_visibility_mut(*debug_entity) {
                visible.visible = self.show_debug_volumes;
            }

            if i < self.meshes.len() {
                let (mesh_entity, _) = &self.meshes[i];
                let is_hovered = Some(*mesh_entity) == self.hovered_entity;
                let debug_color = if is_hovered {
                    nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
                } else {
                    nalgebra_glm::vec4(0.0, 1.0, 1.0, 1.0)
                };

                if let Some(lines) = world.get_lines_mut(*debug_entity) {
                    for line in &mut lines.lines {
                        line.color = debug_color;
                    }
                }
            }
        }

        for (i, obb_debug_entity) in self.debug_obb_volumes.iter().enumerate() {
            if let Some(visible) = world.get_visibility_mut(*obb_debug_entity) {
                visible.visible = self.show_obb_volumes;
            }

            if i < self.meshes.len() {
                let (mesh_entity, _) = &self.meshes[i];

                if let Some(mesh_transform) = world.get_global_transform(*mesh_entity)
                    && let Some(mesh_bv) = world.get_bounding_volume(*mesh_entity)
                {
                    let world_obb = mesh_bv.obb.transform(&mesh_transform.0);

                    let is_hovered = Some(*mesh_entity) == self.hovered_entity;
                    let obb_color = if is_hovered {
                        nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
                    } else {
                        nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0)
                    };

                    if let Some(lines) = world.get_lines_mut(*obb_debug_entity) {
                        lines.lines = create_obb_wireframe(&world_obb, obb_color);
                    }
                }
            }
        }

        for (i, (entity, _)) in self.meshes.iter().enumerate() {
            if let Some(transform) = world.get_local_transform_mut(*entity) {
                let rotation = nalgebra_glm::quat_angle_axis(
                    self.spin_speed * 0.016,
                    &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                );
                transform.rotation = rotation * transform.rotation;
                world.set_local_transform_dirty(*entity, LocalTransformDirty);
            }

            if i < self.debug_volumes.len() {
                let mesh_position = if let Some(mesh_global) = world.get_global_transform(*entity) {
                    nalgebra_glm::vec3(
                        mesh_global.0[(0, 3)],
                        mesh_global.0[(1, 3)],
                        mesh_global.0[(2, 3)],
                    )
                } else {
                    nalgebra_glm::vec3(0.0, 0.0, 0.0)
                };

                if let Some(debug_transform) = world.get_local_transform_mut(self.debug_volumes[i])
                {
                    debug_transform.translation = mesh_position;
                    world.set_local_transform_dirty(self.debug_volumes[i], LocalTransformDirty);
                }
            }
        }

        let mouse = &world.resources.input.mouse;
        let mouse_pos = mouse.position;

        let mut options = PickingOptions::default();
        if let Some(max_dist) = self.max_pick_distance {
            options.max_distance = max_dist;
        }

        let picking_results = pick_entities(world, mouse_pos, options);

        let mut closest_hit = None;
        for result in picking_results {
            for (mesh_entity, _) in &self.meshes {
                if result.entity == *mesh_entity {
                    closest_hit = Some(*mesh_entity);
                    break;
                }
            }
            if closest_hit.is_some() {
                break;
            }
        }

        if let Some(ray) = PickingRay::from_screen_position(world, mouse_pos) {
            self.debug_ray_direction = Some(ray.direction);
        }

        if let Some(prev_hovered) = self.hovered_entity
            && Some(prev_hovered) != closest_hit
        {
            if let Some(transform) = world.get_local_transform_mut(prev_hovered) {
                transform.scale = self.original_scales[&prev_hovered];
                world.set_local_transform_dirty(prev_hovered, LocalTransformDirty);
            }
            if let Some(material_ref) = world.get_material_ref(prev_hovered) {
                let name = material_ref.name.clone();
                if let Some(material) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    &name,
                ) {
                    material.base_color = self.original_colors[&prev_hovered].into();
                }
            }
        }

        self.hovered_entity = closest_hit;

        if let Some(hovered) = self.hovered_entity {
            if let Some(transform) = world.get_local_transform_mut(hovered) {
                transform.scale = self.original_scales[&hovered] * 1.2;
                world.set_local_transform_dirty(hovered, LocalTransformDirty);
            }
            if let Some(material_ref) = world.get_material_ref(hovered) {
                let name = material_ref.name.clone();
                if let Some(material) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    &name,
                ) {
                    material.base_color = nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0).into();
                }
            }
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("3D Picking Demo")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.label("Hover over the spinning meshes to select them!");
                ui.separator();

                ui.label("Controls:");
                ui.label("• Mouse: Hover to select meshes");
                ui.label("• Left mouse + drag: Orbit camera");
                ui.label("• Right mouse + drag: Pan camera");
                ui.label("• Scroll wheel: Zoom in/out");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Spin Speed:");
                    ui.add(egui::Slider::new(&mut self.spin_speed, 0.0..=5.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Show Sphere Volumes:");
                    ui.checkbox(&mut self.show_debug_volumes, "");
                });

                ui.horizontal(|ui| {
                    ui.label("Show OBB Volumes:");
                    ui.checkbox(&mut self.show_obb_volumes, "");
                });

                ui.separator();
                ui.label("Advanced Options:");

                ui.horizontal(|ui| {
                    ui.label("Max Pick Distance:");
                    let mut has_max = self.max_pick_distance.is_some();
                    ui.checkbox(&mut has_max, "Enabled");
                    if has_max {
                        let mut dist = self.max_pick_distance.unwrap_or(20.0);
                        ui.add(egui::Slider::new(&mut dist, 5.0..=50.0));
                        self.max_pick_distance = Some(dist);
                    } else {
                        self.max_pick_distance = None;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Camera Mode:");
                    if ui
                        .button(if self.orthographic_mode {
                            "Orthographic"
                        } else {
                            "Perspective"
                        })
                        .clicked()
                    {
                        self.orthographic_mode = !self.orthographic_mode;
                        if let Some(camera_entity) = self.camera_entity
                            && let Some(camera) = world.get_camera_mut(camera_entity)
                        {
                            camera.projection = if self.orthographic_mode {
                                Projection::Orthographic(OrthographicCamera {
                                    x_mag: 15.0,
                                    y_mag: 10.0,
                                    z_near: 0.1,
                                    z_far: 1000.0,
                                })
                            } else {
                                Projection::Perspective(PerspectiveCamera {
                                    aspect_ratio: None,
                                    y_fov_rad: 45.0_f32.to_radians(),
                                    z_far: None,
                                    z_near: 0.01,
                                })
                            };
                        }
                    }
                });

                ui.separator();

                if let Some(hovered) = self.hovered_entity {
                    ui.label(format!("Hovered Entity: {:?}", hovered));
                    for (entity, name) in &self.meshes {
                        if *entity == hovered {
                            ui.label(format!("Mesh Type: {}", name));
                            if let Some(transform) = world.get_global_transform(hovered) {
                                let matrix = transform.0;
                                ui.label(format!(
                                    "Position: ({:.2}, {:.2}, {:.2})",
                                    matrix[(0, 3)],
                                    matrix[(1, 3)],
                                    matrix[(2, 3)]
                                ));
                            }
                            break;
                        }
                    }
                } else {
                    ui.label("No entity hovered");
                }

                ui.separator();
                ui.label("All Meshes:");
                for (entity, name) in &self.meshes {
                    let is_hovered = Some(*entity) == self.hovered_entity;
                    if is_hovered {
                        ui.label(format!("• {} (Entity {:?}) [HOVERED]", name, entity));
                    } else {
                        ui.label(format!("• {} (Entity {:?})", name, entity));
                    }
                }

                ui.separator();
                ui.label("Debug Info:");
                if let Some(ray_dir) = self.debug_ray_direction {
                    ui.label(format!(
                        "Ray Direction: ({:.3}, {:.3}, {:.3})",
                        ray_dir.x, ray_dir.y, ray_dir.z
                    ));
                }

                let mouse = &world.resources.input.mouse;
                ui.label(format!(
                    "Mouse Position: ({:.1}, {:.1})",
                    mouse.position.x, mouse.position.y
                ));

                if let Some(window) = &world.resources.window.handle {
                    let size = window.inner_size();
                    ui.label(format!("Window Size: {}x{}", size.width, size.height));
                }
            });
    }
}
