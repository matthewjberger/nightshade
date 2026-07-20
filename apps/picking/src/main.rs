use nightshade::ecs::camera::commands::spawn_pan_orbit_camera_at;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::gpu_picking::GpuPickResult;
use nightshade::platform::input::resources::MouseState;
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .run()
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

struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Picking Demo".to_string();
        app.insert_resource(PickingDemo::default());
        app.add_system(Stage::Startup, initialize);
        app.add_systems(
            Stage::Update,
            (
                pan_orbit_camera_system,
                poll_panel_state,
                spin_meshes,
                sync_debug_volumes,
                update_hover,
                apply_click,
                apply_outline,
                refresh_panel_text,
                drain_keyboard,
            ),
        );
    }
}

fn initialize(picking_demo: &mut PickingDemo, world: &mut World) {
    world.res_mut::<DebugDraw>().show_grid = true;
    world.res_mut::<RenderSettings>().atmosphere = Atmosphere::Sky;
    world.res_mut::<Selection>().outline_enabled = true;
    picking_demo.show_debug_volumes = true;
    picking_demo.spin_speed = 1.0;

    spawn_sun(world);

    for (mesh_type, position, color) in MESH_LAYOUT {
        let entity = spawn_mesh(world, mesh_type, position, Vec3::new(1.0, 1.0, 1.0));
        let material_name = format!("{}_{}", mesh_type, entity.id);
        set_material(
            world,
            entity,
            material_name,
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );

        let bounding_radius = world
            .get::<nightshade::render::bounding_volume::BoundingVolume>(entity)
            .map(|bv| bv.sphere_radius)
            .unwrap_or(1.0);
        let sphere_entity = spawn_debug_sphere(world, mesh_type, position, bounding_radius);
        picking_demo.debug_volumes.push((entity, sphere_entity));

        picking_demo.meshes.push(DemoMesh {
            entity,
            label: mesh_type,
        });
    }

    let camera = spawn_pan_orbit_camera_at(
        world,
        Vec3::new(0.0, 0.0, 0.0),
        12.0,
        0.0,
        0.4,
        "Picking Demo Camera".to_string(),
    );
    world.res_mut::<ActiveCamera>().0 = Some(camera);

    let preview_entity = spawn_entities(world, LINES | VISIBILITY | GLOBAL_TRANSFORM, 1)[0];
    world.set(preview_entity, Lines::default());
    world.set(preview_entity, Visibility { visible: true });
    world.set(preview_entity, GlobalTransform::default());
    picking_demo.preview_lines_entity = Some(preview_entity);

    picking_demo.handles = build_panel(
        world,
        picking_demo.mode,
        picking_demo.show_debug_volumes,
        picking_demo.spin_speed,
    );
}

fn poll_panel_state(picking_demo: &mut PickingDemo, world: &mut World) {
    picking_demo.poll_panel(world);
}

fn spin_meshes(picking_demo: &mut PickingDemo, world: &mut World) {
    picking_demo.tick_rotation(world);
}

fn sync_debug_volumes(picking_demo: &mut PickingDemo, world: &mut World) {
    picking_demo.sync_debug_volume_visibility(world);
}

fn update_hover(mut picking_demo: ResMut<PickingDemo>, input: Res<Input>, world: &mut World) {
    let picking_demo = &mut *picking_demo;
    match picking_demo.mode {
        PickingMode::GpuHover => picking_demo.update_gpu_hover(&input, world),
        PickingMode::CpuHover => picking_demo.update_cpu_hover(&input, world),
    }
}

fn apply_click(mut picking_demo: ResMut<PickingDemo>, input: Res<Input>, world: &mut World) {
    let picking_demo = &mut *picking_demo;
    picking_demo.handle_click(&input, world);
}

fn apply_outline(picking_demo: &mut PickingDemo, world: &mut World) {
    picking_demo.sync_outline(world);
}

fn refresh_panel_text(mut picking_demo: ResMut<PickingDemo>, input: Res<Input>, world: &mut World) {
    let picking_demo = &mut *picking_demo;
    picking_demo.sync_panel(&input, world);
}

fn drain_keyboard(
    mut picking_demo: ResMut<PickingDemo>,
    reader: EventReader<AppEvent>,
    world: &mut World,
) {
    let picking_demo = &mut *picking_demo;
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
        {
            match key {
                KeyCode::Escape => world.res_mut::<Window>().should_exit = true,
                KeyCode::F1 => {
                    let panel = picking_demo.handles.panel;
                    let visible = ui_node_visible(world, panel);
                    ui_set_visible(world, panel, !visible);
                }
                _ => {}
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
        let delta = world.res::<Time>().delta_time.max(0.0);
        if self.spin_speed.abs() < f32::EPSILON || delta <= 0.0 {
            return;
        }
        let rotation =
            nalgebra_glm::quat_angle_axis(self.spin_speed * delta, &Vec3::new(0.0, 1.0, 0.0));
        for mesh in &self.meshes {
            if let Some(transform) =
                world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(mesh.entity)
            {
                transform.rotation = rotation * transform.rotation;
            }
        }
        for (mesh_entity, sphere_entity) in &self.debug_volumes {
            if let Some(global) =
                world.get::<nightshade::ecs::transform::components::GlobalTransform>(*mesh_entity)
            {
                let translation = Vec3::new(global.0[(0, 3)], global.0[(1, 3)], global.0[(2, 3)]);
                if let Some(transform) = world
                    .get_mut::<nightshade::ecs::transform::components::LocalTransform>(
                        *sphere_entity,
                    )
                {
                    transform.translation = translation;
                }
            }
        }
    }

    fn sync_debug_volume_visibility(&self, world: &mut World) {
        for (_, sphere_entity) in &self.debug_volumes {
            if let Some(visibility) =
                world.get_mut::<nightshade::ecs::primitives::Visibility>(*sphere_entity)
            {
                visibility.visible = self.show_debug_volumes;
            }
        }
    }

    fn update_gpu_hover(&mut self, input: &Input, world: &mut World) {
        if let Some(result) = world.res_mut::<GpuPicking>().take_result() {
            self.last_pick_result = Some(result);
        }

        let mouse = input.mouse;
        let mouse_pos = mouse.position;
        let current_mouse_pos = (mouse_pos.x as u32, mouse_pos.y as u32);
        let ui_capturing = world.res::<UserInterface>().hud_wants_pointer
            || world
                .res::<RetainedUiState>()
                .interaction
                .hovered_entity
                .is_some()
            || world
                .res::<RetainedUiState>()
                .interaction
                .active_entity
                .is_some();
        let mouse_held = mouse.state.contains(MouseState::LEFT_CLICKED);
        if mouse_held {
            self.highlighted_entity = None;
        } else if !ui_capturing && current_mouse_pos != self.last_mouse_pos {
            world
                .res_mut::<GpuPicking>()
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
            world.set(preview_entity, lines);
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
                    let mesh_position = world
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

    fn update_cpu_hover(&mut self, input: &Input, world: &mut World) {
        if let Some(preview_entity) = self.preview_lines_entity {
            world.set(preview_entity, Lines::new(vec![]));
        }

        let mouse = input.mouse;
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

    fn handle_click(&mut self, input: &Input, world: &mut World) {
        let mouse = input.mouse;
        if !mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
            return;
        }
        let ui_capturing = world.res::<UserInterface>().hud_wants_pointer
            || world
                .res::<RetainedUiState>()
                .interaction
                .hovered_entity
                .is_some()
            || world
                .res::<RetainedUiState>()
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
        world.res_mut::<Selection>().active_entity =
            self.highlighted_entity.or(self.selected_entity);
    }

    fn sync_panel(&self, input: &Input, world: &mut World) {
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

        let mouse = input.mouse.position;
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
    world.set(entity, Name(format!("{name} Debug Volume")));
    world.set(
        entity,
        LocalTransform {
            translation: position,
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
    );
    world.set(entity, GlobalTransform::default());
    world.set(entity, Lines::new(lines));
    world.set(entity, Visibility { visible: true });
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
