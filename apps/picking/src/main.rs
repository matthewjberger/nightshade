use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::gpu_picking::GpuPickResult;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::material::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PickingDemo::default())
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum PickingMode {
    #[default]
    GpuHover,
    CpuHover,
}

struct DemoMesh {
    entity: Entity,
    label: &'static str,
}

#[derive(Default, Clone, Copy)]
struct UiHandles {
    panel: Entity,
    mode_button: Entity,
    debug_volumes_toggle: Entity,
    spin_slider: Entity,
    status_label: Entity,
    detail_label: Entity,
}

#[derive(Default)]
struct PickingDemo {
    meshes: Vec<DemoMesh>,
    debug_volumes: Vec<(Entity, Entity)>,
    highlighted_entity: Option<Entity>,
    selected_entity: Option<Entity>,
    last_pick_result: Option<GpuPickResult>,
    last_mouse_pos: (u32, u32),
    preview_lines_entity: Option<Entity>,
    mode: PickingMode,
    show_debug_volumes: bool,
    spin_speed: f32,
    handles: UiHandles,
}

const MESH_LAYOUT: [(&str, Vec3, Vec4); 5] = [
    (
        "Cube",
        Vec3::new(-8.0, 0.0, 0.0),
        Vec4::new(0.8, 0.2, 0.2, 1.0),
    ),
    (
        "Sphere",
        Vec3::new(-4.0, 0.0, 0.0),
        Vec4::new(0.2, 0.8, 0.2, 1.0),
    ),
    (
        "Cylinder",
        Vec3::new(0.0, 0.0, 0.0),
        Vec4::new(0.2, 0.2, 0.8, 1.0),
    ),
    (
        "Torus",
        Vec3::new(4.0, 0.0, 0.0),
        Vec4::new(0.2, 0.8, 0.8, 1.0),
    ),
    (
        "Cone",
        Vec3::new(8.0, 0.0, 0.0),
        Vec4::new(0.8, 0.2, 0.8, 1.0),
    ),
];

const PANEL_RECT: Rect = Rect {
    min: Vec2::new(16.0, 16.0),
    max: Vec2::new(360.0, 320.0),
};

impl State for PickingDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.window.title = "Picking Demo".to_string();
        world.resources.user_interface.enabled = true;
        world.resources.retained_ui.enabled = true;
        world.resources.debug_draw.show_grid = true;
        world.resources.render_settings.atmosphere = Atmosphere::Sky;
        world.resources.debug_draw.selection_outline_enabled = true;
        self.show_debug_volumes = true;
        self.spin_speed = 1.0;

        spawn_sun(world);

        for (mesh_type, position, color) in MESH_LAYOUT {
            let entity = spawn_mesh(world, mesh_type, position, Vec3::new(1.0, 1.0, 1.0));
            let material_name = format!("{}_{}", mesh_type, entity.id);
            material_registry_insert(
                &mut world.resources.assets.material_registry,
                material_name.clone(),
                Material {
                    base_color: color.into(),
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .assets
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                registry_add_reference(
                    &mut world.resources.assets.material_registry.registry,
                    index,
                );
            }
            world.ecs.worlds[CORE].set(entity, MaterialRef::new(material_name));

            let bounding_radius = world.ecs.worlds[CORE]
                .get::<nightshade::render::bounding_volume::BoundingVolume>(entity)
                .map(|bv| bv.sphere_radius)
                .unwrap_or(1.0);
            let sphere_entity = spawn_debug_sphere(world, mesh_type, position, bounding_radius);
            self.debug_volumes.push((entity, sphere_entity));

            self.meshes.push(DemoMesh {
                entity,
                label: mesh_type,
            });
        }

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            12.0,
            0.0,
            0.4,
            "Picking Demo Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        let preview_entity = spawn_entities(world, LINES | VISIBILITY | GLOBAL_TRANSFORM, 1)[0];
        world.ecs.worlds[CORE].set(preview_entity, Lines::default());
        world.ecs.worlds[CORE].set(preview_entity, Visibility { visible: true });
        world.ecs.worlds[CORE].set(preview_entity, GlobalTransform::default());
        self.preview_lines_entity = Some(preview_entity);

        self.handles = build_panel(world, self.mode, self.show_debug_volumes, self.spin_speed);
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);

        self.poll_panel(world);
        self.tick_rotation(world);
        self.sync_debug_volume_visibility(world);

        match self.mode {
            PickingMode::GpuHover => self.update_gpu_hover(world),
            PickingMode::CpuHover => self.update_cpu_hover(world),
        }

        self.handle_click(world);
        self.sync_outline(world);
        self.sync_panel(world);

        let events = std::mem::take(&mut world.resources.input.events);
        for event in events {
            if let AppEvent::Keyboard {
                key,
                state: KeyState::Pressed,
            } = event
            {
                match key {
                    KeyCode::Escape => world.resources.window.should_exit = true,
                    KeyCode::F1 => {
                        let panel = self.handles.panel;
                        ui_set_visible(world, panel, !ui_node_visible(world, panel));
                    }
                    _ => {}
                }
            }
        }
    }
}

impl PickingDemo {
    fn poll_panel(&mut self, world: &mut World) {
        if let Some(slider) = widget::<UiSliderData>(world, self.handles.spin_slider) {
            self.spin_speed = slider.value;
        }
        if let Some(toggle) = widget::<UiCheckboxData>(world, self.handles.debug_volumes_toggle) {
            self.show_debug_volumes = toggle.value;
        }

        let mut mode_clicked = false;
        for event in ui_events(world) {
            if let UiEvent::ButtonClicked(entity) = event
                && *entity == self.handles.mode_button
            {
                mode_clicked = true;
            }
        }
        if mode_clicked {
            self.highlighted_entity = None;
            self.mode = match self.mode {
                PickingMode::GpuHover => PickingMode::CpuHover,
                PickingMode::CpuHover => PickingMode::GpuHover,
            };
        }
    }

    fn tick_rotation(&mut self, world: &mut World) {
        let delta = world.resources.window.timing.delta_time.max(0.0);
        if self.spin_speed.abs() < f32::EPSILON || delta <= 0.0 {
            return;
        }
        let rotation =
            nalgebra_glm::quat_angle_axis(self.spin_speed * delta, &Vec3::new(0.0, 1.0, 0.0));
        for mesh in &self.meshes {
            if let Some(transform) =
                world.ecs.worlds[CORE]
                    .get_mut::<nightshade::ecs::transform::components::LocalTransform>(mesh.entity)
            {
                transform.rotation = rotation * transform.rotation;
            }
        }
        for (mesh_entity, sphere_entity) in &self.debug_volumes {
            if let Some(global) =
                world.ecs.worlds[CORE]
                    .get::<nightshade::ecs::transform::components::GlobalTransform>(*mesh_entity)
            {
                let translation = Vec3::new(global.0[(0, 3)], global.0[(1, 3)], global.0[(2, 3)]);
                if let Some(transform) = world.ecs.worlds[CORE]
                    .get_mut::<nightshade::ecs::transform::components::LocalTransform>(
                    *sphere_entity,
                ) {
                    transform.translation = translation;
                }
            }
        }
    }

    fn sync_debug_volume_visibility(&self, world: &mut World) {
        for (_, sphere_entity) in &self.debug_volumes {
            if let Some(visibility) = world.ecs.worlds[CORE]
                .get_mut::<nightshade::ecs::primitives::Visibility>(*sphere_entity)
            {
                visibility.visible = self.show_debug_volumes;
            }
        }
    }

    fn update_gpu_hover(&mut self, world: &mut World) {
        if let Some(result) = world.resources.gpu_picking.take_result() {
            self.last_pick_result = Some(result);
        }

        let mouse = world.resources.input.mouse;
        let mouse_pos = mouse.position;
        let current_mouse_pos = (mouse_pos.x as u32, mouse_pos.y as u32);
        let ui_capturing = world.resources.user_interface.hud_wants_pointer
            || world
                .resources
                .retained_ui
                .interaction
                .hovered_entity
                .is_some()
            || world
                .resources
                .retained_ui
                .interaction
                .active_entity
                .is_some();
        let mouse_held = mouse.state.contains(MouseState::LEFT_CLICKED);
        if mouse_held {
            self.highlighted_entity = None;
        } else if !ui_capturing && current_mouse_pos != self.last_mouse_pos {
            world
                .resources
                .gpu_picking
                .request_pick(current_mouse_pos.0, current_mouse_pos.1);
            self.last_mouse_pos = current_mouse_pos;
        }

        if let Some(preview_entity) = self.preview_lines_entity {
            let lines = match &self.last_pick_result {
                Some(result) if result.depth > 0.0 && !mouse_held => {
                    preview_marker(result.world_position, result.world_normal, 0.6)
                }
                _ => Lines::new(vec![]),
            };
            world.ecs.worlds[CORE].set(preview_entity, lines);
        }

        if mouse_held {
            return;
        }

        self.highlighted_entity = self.last_pick_result.as_ref().and_then(|result| {
            if result.depth <= 0.0 {
                return None;
            }
            self.meshes
                .iter()
                .map(|mesh| {
                    let mesh_position = world.ecs.worlds[CORE]
                        .get::<nightshade::ecs::transform::components::GlobalTransform>(mesh.entity)
                        .map(|t| Vec3::new(t.0[(0, 3)], t.0[(1, 3)], t.0[(2, 3)]))
                        .unwrap_or_else(Vec3::zeros);
                    let distance = nalgebra_glm::distance(&result.world_position, &mesh_position);
                    (mesh.entity, distance)
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .filter(|(_, distance)| *distance < 1.5)
                .map(|(entity, _)| entity)
        });
    }

    fn update_cpu_hover(&mut self, world: &mut World) {
        if let Some(preview_entity) = self.preview_lines_entity {
            world.ecs.worlds[CORE].set(preview_entity, Lines::new(vec![]));
        }

        let mouse = world.resources.input.mouse;
        if mouse.state.contains(MouseState::LEFT_CLICKED) {
            self.highlighted_entity = None;
            return;
        }
        let hits = pick_entities(world, mouse.position, PickingOptions::default());
        let valid: std::collections::HashSet<Entity> =
            self.meshes.iter().map(|m| m.entity).collect();
        self.highlighted_entity = hits
            .into_iter()
            .find(|hit| valid.contains(&hit.entity))
            .map(|hit| hit.entity);
    }

    fn handle_click(&mut self, world: &mut World) {
        let mouse = world.resources.input.mouse;
        if !mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
            return;
        }
        let ui_capturing = world.resources.user_interface.hud_wants_pointer
            || world
                .resources
                .retained_ui
                .interaction
                .hovered_entity
                .is_some()
            || world
                .resources
                .retained_ui
                .interaction
                .active_entity
                .is_some();
        if ui_capturing {
            return;
        }
        let target = self.highlighted_entity;
        self.selected_entity = match (self.selected_entity, target) {
            (Some(prev), Some(now)) if prev == now => None,
            (_, Some(now)) => Some(now),
            (_, None) => None,
        };
    }

    fn sync_outline(&self, world: &mut World) {
        world
            .resources
            .editor_selection
            .bounding_volume_selected_entity = self.highlighted_entity.or(self.selected_entity);
    }

    fn sync_panel(&self, world: &mut World) {
        ui_button_set_text(world, self.handles.mode_button, mode_label(self.mode));
        let label_for = |entity: Entity| -> String {
            self.meshes
                .iter()
                .find(|m| m.entity == entity)
                .map(|m| m.label.to_string())
                .unwrap_or_else(|| "?".to_string())
        };
        let status = match (self.selected_entity, self.highlighted_entity) {
            (Some(selected), Some(hovered)) if selected == hovered => {
                format!("Selected: {} (hover)", label_for(selected))
            }
            (Some(selected), Some(hovered)) => format!(
                "Selected: {}    Hover: {}",
                label_for(selected),
                label_for(hovered)
            ),
            (Some(selected), None) => format!("Selected: {}", label_for(selected)),
            (None, Some(hovered)) => format!("Hover: {}", label_for(hovered)),
            (None, None) => "Selected: none".to_string(),
        };
        ui_set_text(world, self.handles.status_label, &status);

        let mouse = world.resources.input.mouse.position;
        let detail = match &self.last_pick_result {
            Some(result) if result.depth > 0.0 => format!(
                "Mouse {:.0}, {:.0}    Hit ({:.2}, {:.2}, {:.2})",
                mouse.x,
                mouse.y,
                result.world_position.x,
                result.world_position.y,
                result.world_position.z
            ),
            _ => format!("Mouse {:.0}, {:.0}    no gpu hit", mouse.x, mouse.y),
        };
        ui_set_text(world, self.handles.detail_label, &detail);
    }
}

fn mode_label(mode: PickingMode) -> &'static str {
    match mode {
        PickingMode::GpuHover => "Mode: GPU hover",
        PickingMode::CpuHover => "Mode: CPU hover",
    }
}

fn build_panel(
    world: &mut World,
    initial_mode: PickingMode,
    initial_show_volumes: bool,
    initial_spin: f32,
) -> UiHandles {
    let mut tree = UiTreeBuilder::new(world);
    let panel = tree.add_floating_panel("picking_demo", "Picking Demo", PANEL_RECT);
    let content = widget::<UiPanelData>(tree.world_mut(), panel)
        .map(|d| d.content_entity)
        .expect("panel content");
    let mut mode_button = Entity::default();
    let mut status_label = Entity::default();
    let mut detail_label = Entity::default();
    let mut spin_slider = Entity::default();
    let mut debug_volumes_toggle = Entity::default();
    tree.in_parent(content, |tree| {
        tree.add_label("Hover for outline, click to lock.");
        tree.add_label("Drag to orbit, scroll to zoom.");
        tree.add_label("F1 toggles this panel.");
        mode_button = tree.add_button(mode_label(initial_mode));
        status_label = tree.add_label("Selected: none");
        detail_label = tree.add_label("");
        tree.add_label("Spin speed");
        spin_slider = tree.add_slider(initial_spin, 0.0, 5.0);
        debug_volumes_toggle = tree.add_checkbox("Show debug volumes", initial_show_volumes);
    });
    tree.finish();

    UiHandles {
        panel,
        mode_button,
        debug_volumes_toggle,
        spin_slider,
        status_label,
        detail_label,
    }
}

fn spawn_debug_sphere(world: &mut World, name: &str, position: Vec3, radius: f32) -> Entity {
    let lines = sphere_wireframe(radius, vec4(0.0, 1.0, 1.0, 1.0), 16);
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LINES | VISIBILITY,
        1,
    )[0];
    world.ecs.worlds[CORE].set(entity, Name(format!("{name} Debug Volume")));
    world.ecs.worlds[CORE].set(
        entity,
        LocalTransform {
            translation: position,
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
    );
    world.ecs.worlds[CORE].set(entity, GlobalTransform::default());
    world.ecs.worlds[CORE].set(entity, Lines::new(lines));
    world.ecs.worlds[CORE].set(entity, Visibility { visible: true });
    entity
}

fn sphere_wireframe(radius: f32, color: Vec4, segments: u32) -> Vec<Line> {
    let mut lines = Vec::new();
    for i in 0..segments {
        let phi = std::f32::consts::PI * (i as f32 / segments as f32);
        let y = radius * phi.cos();
        let circle_radius = radius * phi.sin();
        for j in 0..segments {
            let theta1 = 2.0 * std::f32::consts::PI * (j as f32 / segments as f32);
            let theta2 = 2.0 * std::f32::consts::PI * ((j + 1) as f32 / segments as f32);
            lines.push(Line {
                start: Vec3::new(
                    circle_radius * theta1.cos(),
                    y,
                    circle_radius * theta1.sin(),
                ),
                end: Vec3::new(
                    circle_radius * theta2.cos(),
                    y,
                    circle_radius * theta2.sin(),
                ),
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
            lines.push(Line {
                start: Vec3::new(
                    radius * phi1.sin() * cos_theta,
                    radius * phi1.cos(),
                    radius * phi1.sin() * sin_theta,
                ),
                end: Vec3::new(
                    radius * phi2.sin() * cos_theta,
                    radius * phi2.cos(),
                    radius * phi2.sin() * sin_theta,
                ),
                color,
            });
        }
    }
    lines
}

fn preview_marker(position: Vec3, normal: Vec3, size: f32) -> Lines {
    let normal = if normal.norm() > 1e-4 {
        nalgebra_glm::normalize(&normal)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let helper = if normal.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let tangent = nalgebra_glm::normalize(&nalgebra_glm::cross(&normal, &helper));
    let bitangent = nalgebra_glm::cross(&normal, &tangent);
    let half = size * 0.5;
    let color = vec4(0.0, 1.0, 0.0, 1.0);
    let normal_color = vec4(1.0, 1.0, 0.0, 1.0);

    let corners = [
        position + tangent * half + bitangent * half,
        position - tangent * half + bitangent * half,
        position - tangent * half - bitangent * half,
        position + tangent * half - bitangent * half,
    ];
    let mut lines_vec = Vec::with_capacity(5);
    for i in 0..4 {
        lines_vec.push(Line {
            start: corners[i],
            end: corners[(i + 1) % 4],
            color,
        });
    }
    lines_vec.push(Line {
        start: position,
        end: position + normal * size,
        color: normal_color,
    });
    Lines::new(lines_vec)
}
