use crate::commands;
use crate::data::ask::Ask;
use crate::data::cell::{Cell, Tag};
use crate::data::{self};
use crate::resources::{Clock, Machine, Plate, Script, Sign, Store, Tape, Vault};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_decals::contract::Decal;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_input::Actions;
use nightshade_physics::contract::{ColliderComponent, ColliderShape, RigidBodyComponent};
use nightshade_platform::prelude::*;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::primitives::register_primitive_mesh;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::{InstanceTransform, InstancedMesh};
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::material::{Material, TextureTransform};
use nightshade_scene::contract::primitives::Visibility;
use nightshade_sprite::prelude::{Sprite, SpriteAtlases, SpriteRegion, sprite_region};

const ANCHOR_REACH: f32 = 4.0;

struct View {
    eye: Vec3,
    right: Vec3,
    up: Vec3,
    ahead: Vec3,
    turn: Quat,
    span: f32,
    aspect: f32,
    reach: f32,
    pixels: (f32, f32),
    corner: (f32, f32),
    flat: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            eye: Vec3::zeros(),
            right: Vec3::x(),
            up: Vec3::y(),
            ahead: -Vec3::z(),
            turn: Quat::identity(),
            span: data::DEFAULT_VIEW_HEIGHT,
            aspect: 16.0 / 9.0,
            reach: 1.0,
            pixels: (1280.0, 720.0),
            corner: (0.0, 0.0),
            flat: true,
        }
    }
}

impl View {
    fn at(&self, fraction: Vec3) -> Vec3 {
        let wide = self.span * self.aspect;
        self.eye
            + self.ahead * self.reach
            + self.right * ((fraction.x - 0.5) * wide)
            + self.up * ((0.5 - fraction.y) * self.span)
    }

    fn ground(&self, fraction: Vec3) -> Vec3 {
        let wide = self.span * self.aspect;
        let across = self.right * ((fraction.x - 0.5) * wide);
        let raised = self.up * ((0.5 - fraction.y) * self.span);
        if self.flat {
            let start = self.eye + across + raised;
            let step = self.ahead.z;
            if step.abs() < 1.0e-5 {
                return start;
            }
            return start + self.ahead * (-start.z / step);
        }
        let toward = (self.ahead * self.reach + across + raised).normalize();
        if toward.y.abs() < 1.0e-5 {
            return self.eye + toward * self.reach;
        }
        self.eye + toward * (-self.eye.y / toward.y)
    }
}

pub fn frame(
    dock: Res<nightshade_ui::resources::DockState>,
    window: Res<Window>,
    mut viewport: ResMut<nightshade_render::resources::viewport::Viewport>,
) {
    let Some((wide, tall)) = window.cached_viewport_size else {
        return;
    };
    let held = &dock.primary.reserved_areas;
    let width = wide as f32 - held.left - held.right;
    let height = tall as f32 - held.top - held.bottom;
    if width < 1.0 || height < 1.0 {
        return;
    }
    viewport.active_viewport_rect = Some(nightshade_render::data::config::ViewportRect {
        x: held.left,
        y: held.top,
        width,
        height,
    });
}

pub fn sight(
    viewport: Res<nightshade_render::resources::viewport::Viewport>,
    window: Res<Window>,
    time: Res<Time>,
    state: Res<nightshade_render::data::config::RendererState>,
    mut store: ResMut<Store>,
) {
    store.rate = time.frames_per_second;
    store.gpu = state.gpu_frame_time_ms;
    store.canvas = window
        .content_rect
        .filter(|rect| rect.width > 1.0 && rect.height > 1.0)
        .map(|rect| (rect.width, rect.height))
        .or_else(|| {
            window
                .cached_viewport_size
                .filter(|(wide, tall)| *wide > 0 && *tall > 0)
                .map(|(wide, tall)| (wide as f32, tall as f32))
        });
    let held = viewport
        .active_viewport_rect
        .filter(|rect| rect.width > 1.0 && rect.height > 1.0)
        .map(|rect| (rect.x, rect.y, rect.width, rect.height));
    store.sight = held.or_else(|| {
        window
            .cached_viewport_size
            .filter(|(wide, tall)| *wide > 0 && *tall > 0)
            .map(|(wide, tall)| (0.0, 0.0, wide as f32, tall as f32))
    });
}

pub fn tick(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<crate::data::actions::Ask>>,
    script: Res<Script>,
    vault: Res<Vault>,
    mut clock: ResMut<Clock>,
    mut tape: ResMut<Tape>,
    mut window: ResMut<Window>,
    mut registry: ResMut<MaterialRegistry>,
    mut store: ResMut<Store>,
    mut machine: ResMut<Machine>,
    mut commands: Commands,
    mut transforms: Query<(&mut LocalTransform,)>,
    mut cameras: Query<(&Camera,)>,
) {
    if script.fault.is_some() || !vault.ready(&script.models) {
        return;
    }
    let plan = &script.plan;
    let view = look(&store, &window, &mut transforms, &mut cameras);
    if let Some(wanted) = tape.seek.take() {
        if let Some(held) = crate::queries::diff::read(&tape, wanted) {
            store.restore(&held);
            respawn(plan, &mut store, &machine, &mut commands);
            paint(plan, &mut store, &machine, &mut registry, &mut commands);
            glow(plan, &store, &machine, &mut commands);
            dress(plan, &store, &mut commands);
            label(plan, &mut store, &machine);
            show(plan, &store, &view, &mut transforms);
            tape.cursor = Some(wanted);
        }
        return;
    }
    if clock.paused && !clock.step {
        return;
    }
    clock.step = false;
    if let Some(cursor) = tape.cursor.take() {
        commands::tape::rewind(&mut tape, cursor);
    }
    store.frame = store.frame.wrapping_add(1);
    store.cost = (machine.steps, store.bytes, store.commands);
    machine.steps = 0;
    store.bytes = 0;
    store.commands = 0;
    settle(plan, &mut store, &mut transforms);
    sense(plan, &mut store, &time, &input, &actions, &view);
    commands::apply(plan, &mut store);
    touch(plan, &mut store);
    commands::apply(plan, &mut store);
    store.drop_events();

    let ready: Vec<bool> = plan
        .units
        .iter()
        .map(|unit| commands::enabled(plan, &mut store, &mut machine, unit))
        .collect();

    store.ran.resize(plan.units.len(), 0);
    for (seat, index) in plan.turn.iter().enumerate() {
        if let Some(unit) = plan.units.get(*index) {
            let held = unit.once && store.started;
            let waiting = !ready.get(*index).copied().unwrap_or(false);
            if !held && !waiting && !rested(&store, unit, *index) {
                commands::run(plan, &mut store, &mut machine, unit);
                store.ran[*index] = store.frame;
            }
        }
        if plan.phases.get(seat) != plan.phases.get(seat + 1) {
            commands::apply(plan, &mut store);
            build(plan, &mut store, &machine, &mut commands);
        }
    }

    age(plan, &mut store);
    commands::apply(plan, &mut store);
    build(plan, &mut store, &machine, &mut commands);
    paint(plan, &mut store, &machine, &mut registry, &mut commands);
    glow(plan, &store, &machine, &mut commands);
    dress(plan, &store, &mut commands);
    label(plan, &mut store, &machine);
    plate(plan, &mut store, &machine);
    show(plan, &store, &view, &mut transforms);

    for key in &plan.pulses {
        store.asks.push(Ask::ClearBank { key: *key });
    }
    commands::apply(plan, &mut store);

    if let Some(slot) = plan.known.title {
        let held = store.field(slot);
        let title = crate::queries::show(&store, &machine, held);
        if !title.is_empty() && window.title != title {
            window.title = title;
        }
    }
    store.started = true;

    if let Some(slot) = plan.known.close
        && store.field(slot).truthy()
    {
        window.should_exit = true;
    }

    commands::tape::record(&mut tape, &store, data::TAPE_LENGTH);
}

fn look(
    store: &Store,
    window: &Window,
    transforms: &mut Query<(&mut LocalTransform,)>,
    cameras: &mut Query<(&Camera,)>,
) -> View {
    let rect = match store.sight {
        Some((x, y, wide, tall)) if wide > 1.0 && tall > 1.0 => (x, y, wide, tall),
        _ => match window.cached_viewport_size {
            Some((wide, tall)) if tall > 0 => (0.0, 0.0, wide as f32, tall as f32),
            _ => (0.0, 0.0, 1280.0, 720.0),
        },
    };
    let mut view = View {
        aspect: rect.2 / rect.3,
        pixels: (rect.2, rect.3),
        corner: (rect.0, rect.1),
        ..Default::default()
    };
    let Some(camera) = store.camera else {
        return view;
    };
    transforms.for_entity(camera, |(transform,)| {
        view.eye = transform.translation;
        view.turn = transform.rotation;
        view.right = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::x());
        view.up = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::y());
        view.ahead = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &(-Vec3::z()));
    });
    cameras.for_entity(camera, |(held,)| match &held.projection {
        Projection::Orthographic(flat) => {
            view.span = flat.y_mag;
            view.reach = 1.0;
            view.flat = true;
        }
        Projection::Perspective(deep) => {
            view.reach = ANCHOR_REACH;
            view.span = 2.0 * ANCHOR_REACH * (deep.y_fov_rad * 0.5).tan();
            view.flat = false;
        }
    });
    view
}

fn respawn(plan: &data::plan::Plan, store: &mut Store, machine: &Machine, commands: &mut Commands) {
    for entity in &store.rows {
        commands.despawn_recursive(*entity);
    }
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) {
            continue;
        }
        let place = match plan.known.transform {
            Some(key) => {
                let held = store.read(key, row);
                Vec3::new(held.axis(0), held.axis(1), held.axis(2))
            }
            None => Vec3::zeros(),
        };
        let size = match plan.known.scale {
            Some(key) => {
                let held = store.read(key, row);
                Vec3::new(held.axis(0), held.axis(1), held.axis(2).max(0.001))
            }
            None => Vec3::new(1.0, 1.0, 1.0),
        };
        let apart = [plan.known.label, plan.known.light, plan.known.model]
            .into_iter()
            .flatten()
            .any(|key| store.holds(key, row));
        let entity = match apart {
            true => {
                let held = bare(commands, place);
                if let Some(key) = plan.known.model.filter(|key| store.holds(*key, row)) {
                    let path = crate::queries::show(store, machine, store.read(key, row));
                    store.pending.push((row, path));
                }
                held
            }
            false => {
                let mesh = match plan.known.mesh {
                    Some(key) => {
                        let held = store.read(key, row);
                        let name = crate::queries::show(store, machine, held);
                        if name.is_empty() {
                            String::from("Cube")
                        } else {
                            name
                        }
                    }
                    None => String::from("Cube"),
                };
                spawn_mesh_at(commands, &mesh, place, size)
            }
        };
        store.rows[row as usize] = entity;
    }
}

fn spin(value: Cell) -> Quat {
    if value.tag != Tag::Vector {
        return nalgebra_glm::quat_angle_axis(value.value(), &Vec3::z());
    }
    nalgebra_glm::quat_angle_axis(value.axis(1), &Vec3::y())
        * nalgebra_glm::quat_angle_axis(value.axis(0), &Vec3::x())
        * nalgebra_glm::quat_angle_axis(value.axis(2), &Vec3::z())
}

fn bare(commands: &mut Commands, place: Vec3) -> nightshade::ecs::Entity {
    commands.spawn((
        Name(String::from("Sign")),
        LocalTransform {
            translation: place,
            ..Default::default()
        },
        GlobalTransform::default(),
    ))
}

fn sense(
    plan: &data::plan::Plan,
    store: &mut Store,
    time: &Time,
    input: &Input,
    actions: &Actions<crate::data::actions::Ask>,
    view: &View,
) {
    let known = &plan.known;
    if let Some(slot) = known.seed {
        let wanted = store.field(slot).value();
        if wanted != 0.0 {
            store.seed = wanted as u32;
        }
    }
    let flow = match known.flow.map(|slot| store.field(slot)) {
        Some(value) if value.tag == Tag::Number => value.value().clamp(0.0, 4.0),
        _ => 1.0,
    };
    let longest = match known.time_longest.map(|slot| store.field(slot)) {
        Some(value) if value.tag == Tag::Number && value.value() > 0.0 => value.value(),
        _ => data::LONGEST_FRAME,
    };
    if let Some(slot) = known.time_delta {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(time.delta_time.min(longest) * flow),
        });
    }
    let (steps, bytes, commands) = store.cost;
    if let Some(slot) = known.cost_steps {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(steps as f32),
        });
    }
    if let Some(slot) = known.cost_bytes {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(bytes as f32),
        });
    }
    if let Some(slot) = known.cost_commands {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(commands as f32),
        });
    }
    if let Some(slot) = known.time_uptime {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(time.uptime_seconds as f32),
        });
    }

    let moved = actions.axis2(crate::data::actions::Ask::Move);
    let steering = moved.x + actions.axis(crate::data::actions::Ask::StepAcross);
    let forward = -(moved.y + actions.axis(crate::data::actions::Ask::StepAlong));
    let serving = actions.just_pressed(crate::data::actions::Ask::Serve);

    if let Some(slot) = known.steering {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(steering.clamp(-1.0, 1.0)),
        });
    }
    if let Some(slot) = known.forward {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(forward.clamp(-1.0, 1.0)),
        });
    }
    if let Some(slot) = known.serve {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::truth(serving),
        });
    }

    if let Some(slot) = known.rate {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(store.rate),
        });
    }
    if let Some(slot) = known.gpu {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::number(store.gpu),
        });
    }
    if let Some(slot) = known.picture {
        let (wide, tall) = store
            .sight
            .map(|held| (held.2, held.3))
            .unwrap_or((0.0, 0.0));
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::vector(&[wide, tall]),
        });
    }
    for (slot, key, held) in &known.actions {
        let Some(code) = crate::data::keys::key_of(key) else {
            continue;
        };
        let down = if *held {
            input.keyboard.is_key_pressed(code)
        } else {
            input.keyboard.just_pressed(code)
        };
        store.asks.push(Ask::SetField {
            slot: *slot,
            value: Cell::truth(down),
        });
    }

    if let Some(slot) = known.cursor {
        let (wide, tall) = store.canvas.unwrap_or((1.0, 1.0));
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::vector(&[
                (input.mouse.position.x / wide.max(1.0)).clamp(0.0, 1.0),
                (input.mouse.position.y / tall.max(1.0)).clamp(0.0, 1.0),
            ]),
        });
    }
    let pointer = pointing(input, view);
    if let Some(slot) = known.pointer {
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::vector(&[pointer.x, pointer.y]),
        });
    }
    if let Some(slot) = known.pointer_aim {
        let met = view.ground(pointer);
        store.asks.push(Ask::SetField {
            slot,
            value: Cell::vector(&[met.x, met.y, met.z]),
        });
    }
    if let Some(slot) = known.press {
        store.set_field(
            slot,
            Cell::truth(input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED)),
        );
    }
}

fn pointing(input: &Input, view: &View) -> Vec3 {
    Vec3::new(
        ((input.mouse.position.x - view.corner.0) / view.pixels.0).clamp(0.0, 1.0),
        ((input.mouse.position.y - view.corner.1) / view.pixels.1).clamp(0.0, 1.0),
        0.0,
    )
}

fn age(plan: &data::plan::Plan, store: &mut Store) {
    let (Some(key), Some(slot)) = (plan.known.lifetime, plan.known.time_delta) else {
        return;
    };
    let step = store.field(slot).value();
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) || !store.holds(key, row) {
            continue;
        }
        let left = store.read(key, row).value() - step;
        if left <= 0.0 {
            store.doom(row);
        } else {
            store.asks.push(Ask::SetColumn {
                key,
                row,
                value: Cell::number(left),
            });
        }
    }
}

fn build(plan: &data::plan::Plan, store: &mut Store, machine: &Machine, commands: &mut Commands) {
    let sites = std::mem::take(&mut store.spawn_sites);
    let values = std::mem::take(&mut store.spawn_values);
    for (site, start, width) in sites {
        let Some(keys) = plan.sites.get(site as usize) else {
            continue;
        };
        let held = &values[start as usize..(start + width) as usize];
        let mut mesh = String::from("Cube");
        let mut color = Cell::vector(&[1.0, 1.0, 1.0]);
        let mut scale = Vec3::new(1.0, 1.0, 1.0);
        let mut place = Vec3::new(0.0, 0.0, 0.0);
        let mut label: Option<String> = None;
        let mut model: Option<String> = None;
        let mut clip: Option<String> = None;
        let mut burst: Option<String> = None;
        let mut body: Option<String> = None;
        let mut picture: Option<String> = None;
        let mut stain: Option<String> = None;
        let mut copies = 0.0;
        let mut spread = Vec3::new(0.0, 0.0, 0.0);
        let mut tile = -1.0;
        let mut bounce = 0.0;
        let mut volume = 1.0;
        let mut lit = false;

        for (key, value) in keys.iter().zip(held.iter()) {
            let name = plan.columns.get(*key as usize).map(String::as_str);
            if name == Some(data::LIGHT_COLUMN) {
                lit = true;
            }
            match name {
                Some(data::MESH_BINDING) => mesh = crate::queries::show(store, machine, *value),
                Some(data::MODEL_BINDING) => {
                    model = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::SOUND_BINDING) => {
                    clip = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::BURST_BINDING) => {
                    burst = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::BODY_COLUMN) => {
                    body = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::SPRITE_BINDING) => {
                    picture = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::DECAL_BINDING) => {
                    stain = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::COPIES_COLUMN) => copies = value.value(),
                Some(data::SPREAD_COLUMN) => {
                    spread = Vec3::new(value.axis(0), value.axis(1), value.axis(2))
                }
                Some(data::TILE_COLUMN) => tile = value.value(),
                Some(data::BOUNCE_COLUMN) => bounce = value.value(),
                Some(data::VOLUME_COLUMN) => volume = value.value(),
                Some(data::LABEL_COLUMN) => {
                    label = Some(crate::queries::show(store, machine, *value))
                }
                Some(data::COLOR_BINDING) => color = *value,
                Some(data::SCALE_BINDING) => {
                    scale = Vec3::new(value.axis(0), value.axis(1), value.axis(2).max(0.001))
                }
                Some(data::TRANSFORM_COLUMN) => {
                    place = Vec3::new(value.axis(0), value.axis(1), value.axis(2))
                }
                _ => {}
            }
        }

        let field = plan.known.field.is_some_and(|key| keys.contains(&key));
        let made = field
            || label.is_some()
            || model.is_some()
            || clip.is_some()
            || burst.is_some()
            || picture.is_some()
            || stain.is_some()
            || lit;
        let entity = match (made, copies >= 2.0) {
            (true, _) => bare(commands, place),
            (false, true) => {
                let held = bare(commands, place);
                let name = mesh.clone();
                let total = copies as usize;
                commands.queue(move |storage, resources| {
                    let instances = (0..total)
                        .map(|index| InstanceTransform {
                            translation: spread * index as f32,
                            rotation: nalgebra_glm::Quat::identity(),
                            scale: Vec3::new(1.0, 1.0, 1.0),
                        })
                        .collect();
                    storage.set(held, InstancedMesh::with_instances(&name, instances));
                    storage.set(held, Visibility { visible: true });
                    storage.set(held, MaterialRef::default());
                    storage.set(held, nightshade_render::components::CastsShadow);
                    register_primitive_mesh(resources, &name);
                });
                held
            }
            (false, false) => spawn_mesh_at(commands, &mesh, place, scale),
        };
        if let Some(burst) = burst {
            let tone = Vec3::new(color.axis(0), color.axis(1), color.axis(2));
            commands.set(entity, spark(&burst, place, tone));
            commands.set(
                entity,
                nightshade_scene::prelude::Lifetime {
                    remaining: data::BURST_LIFE,
                },
            );
        }
        if let Some(clip) = clip {
            commands.set(
                entity,
                nightshade_audio::contract::AudioSource::new(clip).with_volume(volume),
            );
            commands.set(
                entity,
                nightshade_scene::prelude::Lifetime {
                    remaining: data::SOUND_LIFE,
                },
            );
        }
        if let Some(body) = &body {
            let round = mesh == "Sphere";
            commands.set(
                entity,
                match body.as_str() {
                    "static" => RigidBodyComponent::new_static(),
                    "held" => RigidBodyComponent::new_kinematic(),
                    _ => RigidBodyComponent::new_dynamic(),
                },
            );
            commands.set(
                entity,
                ColliderComponent {
                    shape: if round {
                        ColliderShape::Ball {
                            radius: scale.x.max(0.01) * 0.5,
                        }
                    } else {
                        ColliderShape::Cuboid {
                            hx: scale.x.max(0.01) * 0.5,
                            hy: scale.y.max(0.01) * 0.5,
                            hz: scale.z.max(0.01) * 0.5,
                        }
                    },
                    restitution: bounce,
                    ..Default::default()
                },
            );
        }
        if let Some(stain) = stain {
            commands.set(
                entity,
                Decal {
                    texture: Some(stain),
                    color: [
                        color.axis(0).max(0.0),
                        color.axis(1).max(0.0),
                        color.axis(2).max(0.0),
                        1.0,
                    ],
                    size: Vec2::new(1.0, 1.0),
                    depth: 1.0,
                    ..Default::default()
                },
            );
        }
        if let Some(picture) = picture {
            let tone = [
                color.axis(0).max(0.0),
                color.axis(1).max(0.0),
                color.axis(2).max(0.0),
                1.0,
            ];
            let size = Vec2::new(scale.x, scale.y);
            commands.queue(move |storage, resources| {
                let atlases = resources.res::<SpriteAtlases>();
                let region = match sprite_region(atlases, data::SHEET_ATLAS, &picture) {
                    Some(region) if tile >= 0.0 => {
                        region.cell(data::SHEET_ACROSS, data::SHEET_ACROSS, tile as u32)
                    }
                    Some(region) => region,
                    None => SpriteRegion::FULL,
                };
                storage.set(
                    entity,
                    Sprite {
                        atlas: data::SHEET_ATLAS.to_string(),
                        region,
                        size,
                        color: tone,
                        billboard: false,
                        ..Default::default()
                    },
                );
            });
        }
        let row = store.make_row(entity);
        if let Some(path) = model {
            store.pending.push((row, path));
        }
        if let Some(key) = plan.known.color {
            store.write(key, row, color);
        }
        for (key, value) in keys.iter().zip(held.iter()) {
            store.write(*key, row, *value);
        }
        if let Some(key) = plan.known.scale
            && !store.holds(key, row)
            && label.is_none()
        {
            store.write(key, row, Cell::vector(&[scale.x, scale.y, scale.z]));
        }
    }

    let gone = std::mem::take(&mut store.removals);
    for row in gone {
        if let Some(entity) = store.entity(row) {
            commands.despawn_recursive(entity);
        }
        store.drop_row(row);
    }
}

fn shade(value: f32) -> f32 {
    (value.clamp(0.0, 1.0) * 32.0).round() / 32.0
}

fn stamp(name: &str) -> u32 {
    let mut held = 2166136261u32;
    for byte in name.as_bytes() {
        held ^= *byte as u32;
        held = held.wrapping_mul(16777619);
    }
    held
}

fn paint(
    plan: &data::plan::Plan,
    store: &mut Store,
    machine: &Machine,
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
) {
    let Some(key) = plan.known.color else {
        return;
    };
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.holds(key, row) {
            continue;
        }
        if plan
            .known
            .label
            .is_some_and(|label| store.holds(label, row))
        {
            continue;
        }
        if plan.known.sprite.is_some_and(|held| store.holds(held, row)) {
            continue;
        }
        if plan.known.decal.is_some_and(|held| store.holds(held, row)) {
            continue;
        }
        let value = store.read(key, row);
        let tone = [
            shade(value.axis(0)),
            shade(value.axis(1)),
            shade(value.axis(2)),
            1.0,
        ];
        let skin = plan
            .known
            .texture
            .filter(|held| store.holds(*held, row))
            .map(|held| {
                let value = store.read(held, row);
                crate::queries::show(store, machine, value)
            })
            .filter(|name| !name.is_empty());
        let repeat = plan
            .known
            .repeat
            .filter(|held| store.holds(*held, row))
            .map(|held| store.read(held, row).value())
            .filter(|held| *held > 0.0)
            .unwrap_or(1.0);
        let name = match &skin {
            Some(skin) => format!(
                "dsl_{:.3}_{:.3}_{:.3}_{skin}_{repeat:.2}",
                tone[0], tone[1], tone[2]
            ),
            None => format!("dsl_{:.3}_{:.3}_{:.3}", tone[0], tone[1], tone[2]),
        };
        let mark = stamp(&name);
        if store.painted.get(row as usize).copied() == Some(mark) {
            continue;
        }
        if store.materials.insert(name.clone()) {
            insert_material(
                registry,
                name.clone(),
                Material {
                    base_color: tone,
                    base_texture: skin,
                    base_texture_transform: TextureTransform {
                        scale: [repeat, repeat],
                        ..Default::default()
                    },
                    roughness: 0.55,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
        }
        if let Some(entity) = store.entity(row)
            && let Some(id) =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    registry, &name,
                )
        {
            commands.set(entity, MaterialRef::new(id));
        }
        if let Some(slot) = store.painted.get_mut(row as usize) {
            *slot = mark;
        }
    }
}

fn glow(plan: &data::plan::Plan, store: &Store, machine: &Machine, commands: &mut Commands) {
    let Some(key) = plan.known.light else {
        return;
    };
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) || !store.holds(key, row) {
            continue;
        }
        let Some(entity) = store.entity(row) else {
            continue;
        };
        let held = store.read(key, row);
        let kind = match held.tag {
            Tag::Text => match crate::queries::show(store, machine, held).as_str() {
                "spot" => LightType::Spot,
                "sun" => LightType::Directional,
                _ => LightType::Point,
            },
            _ => LightType::Point,
        };
        let tone = match plan.known.color.filter(|slot| store.holds(*slot, row)) {
            Some(slot) => {
                let value = store.read(slot, row);
                Vec3::new(value.axis(0), value.axis(1), value.axis(2))
            }
            None => Vec3::new(1.0, 1.0, 1.0),
        };
        let strength = match plan.known.strength.filter(|slot| store.holds(*slot, row)) {
            Some(slot) => store.read(slot, row).value(),
            None => data::DEFAULT_LIGHT_STRENGTH,
        };
        let reach = match plan.known.reach.filter(|slot| store.holds(*slot, row)) {
            Some(slot) => store.read(slot, row).value(),
            None => data::DEFAULT_LIGHT_REACH,
        };
        commands.set(
            entity,
            Light {
                light_type: kind,
                color: tone,
                intensity: strength,
                range: reach,
                ..Default::default()
            },
        );
    }
}

fn dress(plan: &data::plan::Plan, store: &Store, commands: &mut Commands) {
    let width = store.rows.len() as u32;
    for row in 0..width {
        let Some(entity) = store.entity(row) else {
            continue;
        };
        if !store.living(row) {
            continue;
        }
        if let Some(key) = plan.known.parent
            && store.holds(key, row)
            && let Some(above) = store.read(key, row).row()
            && let Some(held) = store.entity(above)
        {
            commands.set(entity, ChildOf(held));
        }
        if let Some(key) = plan.known.visible
            && store.holds(key, row)
        {
            commands.set(
                entity,
                Visibility {
                    visible: store.read(key, row).truthy(),
                },
            );
        }
    }
}

fn settle(
    plan: &data::plan::Plan,
    store: &mut Store,
    transforms: &mut Query<(&mut LocalTransform,)>,
) {
    let Some(key) = plan.known.transform else {
        return;
    };
    let width = store.rows.len() as u32;
    let mut found: Vec<(u32, Vec3)> = Vec::new();
    for row in 0..width {
        if !store.living(row) {
            continue;
        }
        let walks = plan.known.path.is_some_and(|held| store.holds(held, row));
        let falls = plan.known.body.is_some_and(|held| store.holds(held, row))
            && plan.known.body.is_some_and(|held| {
                crate::queries::show(store, &Machine::default(), store.read(held, row)) != "static"
            });
        if !walks && !falls {
            continue;
        }
        let Some(entity) = store.entity(row) else {
            continue;
        };
        transforms.for_entity(entity, |(transform,)| {
            found.push((row, transform.translation));
        });
    }
    for (row, place) in found {
        store.asks.push(Ask::SetColumn {
            key,
            row,
            value: Cell::vector(&[place.x, place.y, place.z]),
        });
    }
}

fn spark(name: &str, place: Vec3, tone: Vec3) -> nightshade_particles::contract::ParticleEmitter {
    use nightshade_particles::contract::ParticleEmitter;
    match name {
        "smoke" => ParticleEmitter::smoke(place),
        "fire" => ParticleEmitter::fire(place),
        "flash" => ParticleEmitter::flash_burst(place),
        "glitter" => ParticleEmitter::firework_glitter(place, 48),
        "explosion" => ParticleEmitter::firework_explosion(place, tone, 96),
        _ => ParticleEmitter::sparks(place),
    }
}

fn rested(store: &Store, unit: &data::plan::Unit, index: usize) -> bool {
    if !unit.pure {
        return false;
    }
    let last = store.ran.get(index).copied().unwrap_or_default();
    if last == 0 {
        return false;
    }
    if unit.hears.iter().any(|key| {
        store
            .events
            .get(*key as usize)
            .is_some_and(|held| !held.is_empty())
    }) {
        return false;
    }
    let stirred = unit
        .watches
        .iter()
        .any(|key| store.marks.get(*key as usize).copied().unwrap_or_default() >= last)
        || unit.listens.iter().any(|slot| {
            store
                .field_marks
                .get(*slot as usize)
                .copied()
                .unwrap_or_default()
                >= last
        });
    !stirred
}

fn frame_of(plan: &data::plan::Plan, store: &Store, row: u32) -> ([f32; 2], [f32; 2]) {
    let anchor = match plan.known.anchor.filter(|slot| store.holds(*slot, row)) {
        Some(slot) => {
            let value = store.read(slot, row);
            [value.axis(0), value.axis(1)]
        }
        None => [0.5, 0.5],
    };
    let size = match plan.known.panel.filter(|slot| store.holds(*slot, row)) {
        Some(slot) => {
            let value = store.read(slot, row);
            [value.axis(0).abs(), value.axis(1).abs()]
        }
        None => [0.0, 0.0],
    };
    (anchor, size)
}

fn touch(plan: &data::plan::Plan, store: &mut Store) {
    let (Some(button), Some(clicked), Some(cursor), Some(press)) = (
        plan.known.button,
        plan.known.clicked,
        plan.known.cursor,
        plan.known.press,
    ) else {
        return;
    };
    if !store.field(press).truthy() {
        return;
    }
    let held = store.field(cursor);
    let at = [held.axis(0), held.axis(1)];
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) || !store.holds(button, row) {
            continue;
        }
        let (anchor, size) = frame_of(plan, store, row);
        if (at[0] - anchor[0]).abs() > size[0] * 0.5 || (at[1] - anchor[1]).abs() > size[1] * 0.5 {
            continue;
        }
        store.asks.push(Ask::SetColumn {
            key: clicked,
            row,
            value: Cell::number(1.0),
        });
    }
}

fn plate(plan: &data::plan::Plan, store: &mut Store, machine: &Machine) {
    let mut wanted: Vec<Plate> = Vec::new();
    if let Some(key) = plan.known.panel {
        let width = store.rows.len() as u32;
        for row in 0..width {
            if !store.living(row) || !store.holds(key, row) {
                continue;
            }
            let shown = match plan.known.visible.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => store.read(slot, row).truthy(),
                None => true,
            };
            if !shown {
                continue;
            }
            let (anchor, size) = frame_of(plan, store, row);
            if size[0] <= 0.0 || size[1] <= 0.0 {
                continue;
            }
            let tone = match plan.known.color.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => {
                    let value = store.read(slot, row);
                    [value.axis(0), value.axis(1), value.axis(2), 1.0]
                }
                None => [0.1, 0.12, 0.16, 1.0],
            };
            let line = match plan.known.label.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => {
                    let held = store.read(slot, row);
                    crate::queries::show(store, machine, held)
                }
                None => String::new(),
            };
            let ink = match plan.known.ink.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => {
                    let value = store.read(slot, row);
                    [value.axis(0), value.axis(1), value.axis(2), 1.0]
                }
                None => [0.93, 0.95, 1.0, 1.0],
            };
            let letters = match plan.known.size.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => store.read(slot, row).value().max(1.0),
                None => data::DEFAULT_TEXT_SIZE,
            };
            wanted.push(Plate {
                size,
                tone,
                anchor,
                line,
                ink,
                letters,
            });
        }
    }
    store.panels = wanted;
}

fn label(plan: &data::plan::Plan, store: &mut Store, machine: &Machine) {
    let mut wanted: Vec<Sign> = Vec::new();
    if let Some(key) = plan.known.label {
        let width = store.rows.len() as u32;
        for row in 0..width {
            if !store.living(row) || !store.holds(key, row) {
                continue;
            }
            if plan.known.panel.is_some_and(|slot| store.holds(slot, row)) {
                continue;
            }
            let shown = match plan.known.visible.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => store.read(slot, row).truthy(),
                None => true,
            };
            if !shown {
                continue;
            }
            let held = store.read(key, row);
            let line = crate::queries::show(store, machine, held);
            if line.is_empty() {
                continue;
            }
            let size = match plan.known.size.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => store.read(slot, row).value(),
                None => data::DEFAULT_TEXT_SIZE,
            };
            let tone = match plan.known.color.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => {
                    let value = store.read(slot, row);
                    [value.axis(0), value.axis(1), value.axis(2), 1.0]
                }
                None => [1.0, 1.0, 1.0, 1.0],
            };
            let justify = match plan.known.justify.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => {
                    let value = store.read(slot, row);
                    match crate::queries::show(store, machine, value).as_str() {
                        "left" => 0,
                        "right" => 2,
                        _ => 1,
                    }
                }
                None => 1,
            };
            let anchor = match plan.known.anchor.filter(|slot| store.holds(*slot, row)) {
                Some(slot) => {
                    let value = store.read(slot, row);
                    [value.axis(0), value.axis(1)]
                }
                None => [0.5, 0.5],
            };
            wanted.push(Sign {
                line,
                tone,
                size: size.max(1.0),
                justify,
                anchor,
            });
        }
    }
    store.signs = wanted;
}

fn show(
    plan: &data::plan::Plan,
    store: &Store,
    view: &View,
    transforms: &mut Query<(&mut LocalTransform,)>,
) {
    let width = store.rows.len() as u32;
    for row in 0..width {
        let Some(entity) = store.entity(row) else {
            continue;
        };
        let anchored = plan
            .known
            .anchor
            .filter(|key| store.holds(*key, row))
            .map(|key| {
                let value = store.read(key, row);
                view.at(Vec3::new(value.axis(0), value.axis(1), 0.0))
            });
        if let Some(place) = anchored {
            transforms.for_entity(entity, |(transform,)| {
                transform.translation = place;
                transform.rotation = view.turn;
            });
        } else if let Some(key) = plan.known.transform
            && store.holds(key, row)
            && !plan.known.body.is_some_and(|held| store.holds(held, row))
            && !plan.known.path.is_some_and(|held| store.holds(held, row))
        {
            let value = store.read(key, row);
            let place = Vec3::new(value.axis(0), value.axis(1), value.axis(2));
            transforms.for_entity(entity, |(transform,)| {
                transform.translation = place;
            });
        }
        if let Some(key) = plan.known.scale
            && store.holds(key, row)
        {
            let value = store.read(key, row);
            let size = Vec3::new(value.axis(0), value.axis(1), value.axis(2).max(0.001));
            transforms.for_entity(entity, |(transform,)| {
                transform.scale = size;
            });
        }
        if let Some(key) = plan.known.rotation
            && store.holds(key, row)
        {
            let turn = spin(store.read(key, row));
            transforms.for_entity(entity, |(transform,)| {
                transform.rotation = turn;
            });
        }
    }
}

pub fn aim(
    script: Res<Script>,
    mut store: ResMut<Store>,
    mut orbits: Query<(&mut PanOrbitCamera,)>,
) {
    let Some(camera) = store.camera else {
        return;
    };
    let known = &script.plan.known;
    let shake = Vec3::new(
        known
            .shake_x
            .map(|slot| store.field(slot).value())
            .unwrap_or(0.0),
        known
            .shake_y
            .map(|slot| store.field(slot).value())
            .unwrap_or(0.0),
        0.0,
    );
    let held = known.eye.map(|slot| store.field(slot));
    let says = held.is_some_and(|value| value.tag == Tag::Vector);
    if !says && shake.x.abs() < 1.0e-5 && shake.y.abs() < 1.0e-5 {
        return;
    }
    let eye = match held {
        Some(value) if value.tag == Tag::Vector => Vec3::new(
            value.axis(0),
            value.axis(1),
            if value.width > 2 {
                value.axis(2)
            } else {
                store.camera_home.z
            },
        ),
        _ => store.camera_home,
    };
    let middle = match known.aim.map(|slot| store.field(slot)) {
        Some(value) if value.tag == Tag::Vector => {
            Vec3::new(value.axis(0), value.axis(1), value.axis(2))
        }
        _ => Vec3::zeros(),
    };
    let offset = eye - middle;
    let reach = nalgebra_glm::length(&offset).max(0.01);
    let turn = offset.x.atan2(offset.z);
    let rise = (offset.y / reach).clamp(-1.0, 1.0).asin();
    let focus = middle + shake;
    let last = store.aimed;
    let mut steered = store.steered;
    orbits.for_entity(camera, |(orbit,)| {
        if let Some((held_turn, held_rise, held_reach)) = last
            && ((orbit.target_yaw - held_turn).abs() > 1.0e-4
                || (orbit.target_pitch - held_rise).abs() > 1.0e-4
                || (orbit.target_radius - held_reach).abs() > 1.0e-4)
        {
            steered = true;
        }
        orbit.focus = focus;
        orbit.target_focus = focus;
        if steered {
            return;
        }
        orbit.yaw = turn;
        orbit.target_yaw = turn;
        orbit.pitch = rise;
        orbit.target_pitch = rise;
        orbit.radius = reach;
        orbit.target_radius = reach;
    });
    store.steered = steered;
    store.aimed = (!steered).then_some((turn, rise, reach));
}
