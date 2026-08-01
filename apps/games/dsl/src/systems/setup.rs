use crate::commands;
use crate::commands::name;
use crate::data::{self, Program};
use crate::queries::{compile, parse, plan};
use crate::resources::{Arguments, Machine, Script, Store, Switcher};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::spawn_sun_without_shadows;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::components::CameraCullingMask;

pub fn initialize(
    arguments: Res<Arguments>,
    mut settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut window: ResMut<Window>,
    mut script: ResMut<Script>,
    mut store: ResMut<Store>,
    mut machine: ResMut<Machine>,
    mut switcher: ResMut<Switcher>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::None;
    settings.show_sky = false;
    settings.clear_color = data::CLEAR_COLOR;
    settings.ibl_blend_factor = 0.0;

    store.sun = Some(spawn_sun_without_shadows(&mut commands));

    let wanted = arguments
        .game
        .clone()
        .unwrap_or_else(|| "breakout".to_string());
    switcher.playing = wanted.clone();
    play(
        &wanted,
        &mut store,
        &mut machine,
        &mut script,
        &mut window,
        &mut active_camera,
        &mut commands,
    );
}

pub fn play(
    wanted: &str,
    store: &mut Store,
    machine: &mut Machine,
    script: &mut Script,
    window: &mut Window,
    active_camera: &mut ActiveCamera,
    commands: &mut Commands,
) {
    match gather(wanted) {
        Ok(source) => {
            script.source = source;
            raise(store, machine, script, window, active_camera, commands);
            if script.fault.is_none() {
                window.title = format!("{} - {wanted}", data::TITLE);
            }
        }
        Err(fault) => {
            clear(store, machine, commands);
            script.source = Program::default();
            script.plan = Default::default();
            window.title = fault.clone();
            script.fault = Some(fault);
        }
    }
}

pub fn raise(
    store: &mut Store,
    machine: &mut Machine,
    script: &mut Script,
    window: &mut Window,
    active_camera: &mut ActiveCamera,
    commands: &mut Commands,
) {
    clear(store, machine, commands);
    script.fault = None;
    script.dirty = false;
    script.models = wanted(&script.source, data::MODEL_BINDING);
    script.sounds = wanted(&script.source, data::SOUND_BINDING);

    match shape(&script.source) {
        Ok(built) => {
            commands::seed(&built, store, machine);
            script.plan = built;
        }
        Err(fault) => {
            window.title = fault.clone();
            script.fault = Some(fault);
            script.plan = Default::default();
        }
    }

    let (camera, home) = look(commands, script, store);
    store.camera = Some(camera);
    store.camera_home = home;
    active_camera.0 = Some(camera);
}

fn clear(store: &mut Store, machine: &mut Machine, commands: &mut Commands) {
    for entity in std::mem::take(&mut store.rows) {
        commands.despawn_recursive(entity);
    }
    if let Some(camera) = store.camera.take() {
        commands.despawn_recursive(camera);
    }
    store.forget();
    store.fresh = true;
    store.dug = false;
    machine.clear();
}

fn wanted(program: &Program, binding: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (_, _, root, rows) in &program.assets {
        for row in rows {
            if crate::data::facility::binding_for(&row.kind) != Some(binding) {
                continue;
            }
            let path = data::joined(root, &row.file);
            if !found.contains(&path) {
                found.push(path);
            }
        }
    }
    found
}

pub fn shape(program: &Program) -> Result<crate::data::plan::Plan, String> {
    plan::check(program)?;
    let turn = plan::order(program);
    compile::build(program, turn)
}

fn gather(wanted: &str) -> Result<Program, String> {
    let mut files: Vec<(String, String, Program)> = Vec::new();
    let mut pending = vec![wanted.to_string()];
    let mut seen: Vec<String> = Vec::new();

    while let Some(file) = pending.pop() {
        if seen.contains(&file) {
            continue;
        }
        seen.push(file.clone());
        let one = parse::read(find(&file)?)?;
        let module = named(&one.module, &file);
        for used in &one.uses {
            pending.push(used.clone());
        }
        files.push((file, module, one));
    }

    let mut program = Program {
        module: files
            .iter()
            .find(|(file, _, _)| file == wanted)
            .map(|(_, module, _)| module.clone())
            .unwrap_or_default(),
        ..Default::default()
    };

    let many = files.len() > 1;
    let moduled: Vec<(String, String)> = files
        .iter()
        .map(|(file, module, _)| (file.clone(), module.clone()))
        .collect();
    let declared: Vec<(String, Vec<String>)> = files
        .iter()
        .map(|(_, module, one)| (module.clone(), plan::owned(one)))
        .collect();

    for (_, module, one) in &mut files {
        let mut visible = Vec::new();
        for used in &one.uses {
            let held = moduled
                .iter()
                .find(|(file, _)| file == used)
                .map(|(_, held)| held.clone())
                .unwrap_or_else(|| used.trim_end_matches(".ns").to_string());
            program.visible.push((module.clone(), held.clone()));
            let names = declared
                .iter()
                .find(|(one, _)| one == &held)
                .map(|(_, names)| names.clone())
                .unwrap_or_default();
            visible.push((held, names));
        }
        for system in &mut one.systems {
            system.module = module.clone();
        }
        if many {
            name::qualify(one, module, &visible);
        }
    }

    let mut running: Vec<String> = vec![wanted.to_string()];
    let mut edge: Vec<String> = vec![wanted.to_string()];
    while let Some(file) = edge.pop() {
        let Some((_, _, one)) = files.iter().find(|(held, _, _)| held == &file) else {
            continue;
        };
        for next in &one.runs {
            if !running.contains(next) {
                running.push(next.clone());
                edge.push(next.clone());
            }
        }
    }

    let mut wanted: Vec<(String, String, crate::data::Expression)> = Vec::new();
    for (_, _, one) in &files {
        for (file, target, value) in &one.sets {
            let module = moduled
                .iter()
                .find(|(held, _)| held == file)
                .map(|(_, held)| held.clone())
                .unwrap_or_default();
            wanted.push((module, target.clone(), value.clone()));
        }
    }

    for (file, module, mut one) in files {
        if !running.contains(&file) {
            one.systems.clear();
            one.orders.clear();
        }
        for name in &one.shows {
            program.shown.push((module.clone(), name.clone()));
        }
        for name in one.components.iter().chain(one.events.iter()) {
            program.owner.push((name.clone(), module.clone()));
        }
        for (name, _) in one.resources.iter().chain(one.configs.iter()) {
            program.owner.push((name.clone(), module.clone()));
        }
        for (name, _, _) in &one.queries {
            program.owner.push((name.clone(), module.clone()));
        }
        for (name, _, _, _) in &one.assets {
            program.owner.push((name.clone(), module.clone()));
        }
        join(&mut program, one);
    }
    for (module, target, value) in wanted {
        let Some((block, field)) = target.split_once('.') else {
            continue;
        };
        let held = if module.is_empty() {
            block.to_string()
        } else {
            format!("{module}{}{block}", name::MARK)
        };
        for (name, fields) in program
            .resources
            .iter_mut()
            .chain(program.configs.iter_mut())
        {
            if name != &held {
                continue;
            }
            for one in fields.iter_mut() {
                if one.name == field {
                    one.default = Some(value.clone());
                }
            }
        }
    }
    Ok(program)
}

fn join(program: &mut Program, one: Program) {
    let Program {
        module: _,
        owner,
        visible,
        shows: _,
        shown: _,
        components,
        shapes,
        resources,
        configs,
        events,
        commands,
        payloads,
        actions,
        queries,
        systems,
        orders,
        variants,
        uses: _,
        runs: _,
        sets: _,
        pulses,
        assets,
        turn: _,
    } = one;
    program.owner.extend(owner);
    program.visible.extend(visible);
    program.components.extend(components);
    program.shapes.extend(shapes);
    program.resources.extend(resources);
    program.configs.extend(configs);
    program.events.extend(events);
    program.commands.extend(commands);
    program.payloads.extend(payloads);
    program.actions.extend(actions);
    program.queries.extend(queries);
    program.systems.extend(systems);
    program.orders.extend(orders);
    program.variants.extend(variants);
    program.pulses.extend(pulses);
    program.assets.extend(assets);
}

fn named(module: &str, file: &str) -> String {
    if module.is_empty() {
        file.trim_end_matches(".ns").to_string()
    } else {
        module.to_string()
    }
}

fn find(wanted: &str) -> Result<&'static str, String> {
    data::FILES
        .iter()
        .chain(data::lesson::SOURCES.iter())
        .find(|(name, _)| *name == wanted)
        .map(|(_, source)| *source)
        .ok_or_else(|| format!("there is no game called {wanted}"))
}

fn look(commands: &mut Commands, script: &Script, store: &Store) -> (Entity, Vec3) {
    let known = &script.plan.known;
    let eye = known.camera_eye.map(|slot| store.field(slot));
    match eye {
        Some(eye) if eye.tag != crate::data::cell::Tag::Nothing => {
            let place = Vec3::new(eye.axis(0), eye.axis(1), eye.axis(2));
            let reach = nalgebra_glm::length(&place).max(1.0);
            let camera = nightshade_camera_controller::commands::spawn_pan_orbit_camera_at(
                commands,
                Vec3::zeros(),
                reach,
                place.x.atan2(place.z),
                (place.y / reach).asin(),
                "Orbit Camera".to_string(),
            );
            commands.set(
                camera,
                CameraCullingMask::new(nightshade_render::components::CullingMask::DEFAULT_LAYER),
            );
            (camera, place)
        }
        _ => {
            let height = match known.view_height.map(|slot| store.field(slot)) {
                Some(value) if value.tag != crate::data::cell::Tag::Nothing => value.value(),
                _ => data::DEFAULT_VIEW_HEIGHT,
            };
            let place = Vec3::new(0.0, 0.0, data::CAMERA_DISTANCE);
            let camera = commands.spawn((
                Name("Field Camera".to_string()),
                LocalTransform {
                    translation: place,
                    ..Default::default()
                },
                GlobalTransform::default(),
                Camera {
                    projection: Projection::Orthographic(OrthographicCamera {
                        x_mag: None,
                        y_mag: height,
                        z_near: 0.1,
                        z_far: 100.0,
                    }),
                    smoothing: None,
                },
            ));
            (camera, place)
        }
    }
}
