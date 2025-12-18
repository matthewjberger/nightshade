#[cfg(feature = "plugins")]
use nightshade::ecs::world::Entity;
#[cfg(feature = "plugins")]
use nightshade::plugin_runtime::{
    Caller, Linker, PluginRuntime, PluginRuntimeConfig, PluginState, read_plugin_memory,
};
use nightshade::prelude::*;
#[cfg(feature = "plugins")]
use plugins_shared::{EnemyType, GameCommand, GameEvent, ItemType};
#[cfg(feature = "plugins")]
use std::collections::HashMap;
#[cfg(feature = "plugins")]
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Template::default())?;
    Ok(())
}

#[cfg(feature = "plugins")]
struct EnemyData {
    entity: Entity,
    enemy_type: EnemyType,
    health: u32,
}

#[cfg(feature = "plugins")]
struct ItemData;

#[cfg(feature = "plugins")]
struct GameState {
    enemies: HashMap<u64, EnemyData>,
    items: HashMap<u64, ItemData>,
    player_health: u32,
    player_max_health: u32,
    player_score: u64,
    next_enemy_id: u64,
    next_item_id: u64,
}

#[cfg(feature = "plugins")]
impl Default for GameState {
    fn default() -> Self {
        Self {
            enemies: HashMap::new(),
            items: HashMap::new(),
            player_health: 100,
            player_max_health: 100,
            player_score: 0,
            next_enemy_id: 1,
            next_item_id: 1,
        }
    }
}

#[derive(Default)]
struct Template {
    #[cfg(feature = "plugins")]
    plugin_runtime: Option<PluginRuntime>,
    #[cfg(feature = "plugins")]
    game_state: GameState,
}

impl State for Template {
    fn title(&self) -> &str {
        "Template"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;

        spawn_sun(world);

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            15.0,
            0.0,
            std::f32::consts::FRAC_PI_4,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_enabled = true;
        }

        #[cfg(feature = "plugins")]
        {
            let plugins_dir = PathBuf::from("plugins");
            let config = PluginRuntimeConfig {
                plugins_base_path: plugins_dir.clone(),
                ..Default::default()
            };

            match PluginRuntime::new(config) {
                Ok(mut runtime) => {
                    runtime.with_custom_linker(|linker: &mut Linker<PluginState>, _engine| {
                        linker.func_wrap(
                            "env",
                            "host_send_game_command",
                            |mut caller: Caller<'_, PluginState>, ptr: u32, len: u32| {
                                if let Some(bytes) = read_plugin_memory(&mut caller, ptr, len) {
                                    caller.data_mut().push_custom_command(bytes);
                                }
                            },
                        )?;
                        Ok(())
                    });

                    tracing::info!("Engine plugins: plugins/engine/ (use nightshade directly)");
                    tracing::info!("App plugins: plugins/app/ (use shared game types)");

                    let engine_plugins = plugins_dir.join("engine");
                    let app_plugins = plugins_dir.join("app");

                    if let Err(error) = runtime.load_plugins_from_directory(&engine_plugins) {
                        tracing::error!("Failed to load engine plugins: {}", error);
                    }
                    if let Err(error) = runtime.load_plugins_from_directory(&app_plugins) {
                        tracing::error!("Failed to load app plugins: {}", error);
                    }

                    runtime.call_on_init(world);

                    let commands: Vec<_> = runtime
                        .drain_custom_commands()
                        .into_iter()
                        .filter_map(|(plugin_id, bytes)| {
                            GameCommand::from_bytes(&bytes).map(|cmd| (plugin_id, cmd))
                        })
                        .collect();

                    process_game_commands(world, &mut runtime, &mut self.game_state, commands);

                    self.plugin_runtime = Some(runtime);
                }
                Err(error) => {
                    tracing::error!("Failed to create plugin runtime: {}", error);
                }
            }
        }
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Template").show(ui_context, |ui| {
            ui.heading("Template");

            #[cfg(feature = "plugins")]
            {
                ui.separator();
                ui.label(format!(
                    "Health: {}/{}",
                    self.game_state.player_health, self.game_state.player_max_health
                ));
                ui.label(format!("Score: {}", self.game_state.player_score));
                ui.label(format!("Enemies: {}", self.game_state.enemies.len()));
            }
        });
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);

        #[cfg(feature = "plugins")]
        if let Some(runtime) = &mut self.plugin_runtime {
            runtime.run_frame(world);

            let commands: Vec<_> = runtime
                .drain_custom_commands()
                .into_iter()
                .filter_map(|(plugin_id, bytes)| {
                    GameCommand::from_bytes(&bytes).map(|cmd| (plugin_id, cmd))
                })
                .collect();

            process_game_commands(world, runtime, &mut self.game_state, commands);
        }
    }

    fn handle_event(&mut self, _world: &mut World, message: &Message) {
        match message {
            Message::Input { event } => {
                tracing::debug!("Input event: {:?}", event);
            }
            Message::App { type_name, .. } => {
                tracing::debug!("App event: {}", type_name);
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }

        #[cfg(feature = "plugins")]
        if let Some(runtime) = &mut self.plugin_runtime {
            runtime.queue_keyboard_event(key_code, key_state);
        }
    }

    fn on_mouse_input(&mut self, _world: &mut World, state: ElementState, button: MouseButton) {
        #[cfg(feature = "plugins")]
        if let Some(runtime) = &mut self.plugin_runtime {
            runtime.queue_mouse_event(state, button);
        }

        let _ = (state, button);
    }
}

#[cfg(feature = "plugins")]
fn dispatch_game_event_to_plugin(runtime: &mut PluginRuntime, plugin_id: u64, event: &GameEvent) {
    if let Ok(bytes) = event.to_bytes() {
        runtime.dispatch_custom_event_to_plugin(
            plugin_id,
            &bytes,
            "game_plugin_alloc",
            "game_plugin_receive_event",
        );
    }
}

#[cfg(feature = "plugins")]
fn dispatch_game_event_to_all(runtime: &mut PluginRuntime, event: &GameEvent) {
    if let Ok(bytes) = event.to_bytes() {
        runtime.dispatch_custom_event_to_all_unfiltered(
            &bytes,
            "game_plugin_alloc",
            "game_plugin_receive_event",
        );
    }
}

#[cfg(feature = "plugins")]
fn process_game_commands(
    world: &mut World,
    runtime: &mut PluginRuntime,
    game_state: &mut GameState,
    commands: Vec<(u64, GameCommand)>,
) {
    for (plugin_id, command) in commands {
        match command {
            GameCommand::SpawnEnemy {
                enemy_type,
                x,
                y,
                z,
                request_id,
            } => {
                let position = Vec3::new(x, y, z);
                let entity = spawn_enemy_entity(world, &enemy_type, position);
                let enemy_id = game_state.next_enemy_id;
                game_state.next_enemy_id += 1;
                let max_health = get_enemy_max_health(&enemy_type);

                game_state.enemies.insert(
                    enemy_id,
                    EnemyData {
                        entity,
                        enemy_type: enemy_type.clone(),
                        health: max_health,
                    },
                );

                let event = GameEvent::EnemySpawned {
                    request_id,
                    enemy_id,
                    enemy_type,
                };
                dispatch_game_event_to_plugin(runtime, plugin_id, &event);
            }
            GameCommand::DespawnEnemy { enemy_id } => {
                if let Some(enemy_data) = game_state.enemies.remove(&enemy_id) {
                    world.queue_command(WorldCommand::DespawnRecursive {
                        entity: enemy_data.entity,
                    });
                    let event = GameEvent::EnemyDied {
                        enemy_id,
                        enemy_type: enemy_data.enemy_type,
                    };
                    dispatch_game_event_to_all(runtime, &event);
                }
            }
            GameCommand::DamageEnemy { enemy_id, damage } => {
                if let Some(enemy_data) = game_state.enemies.get_mut(&enemy_id) {
                    enemy_data.health = enemy_data.health.saturating_sub(damage);
                    let remaining_health = enemy_data.health;
                    let event = GameEvent::EnemyDamaged {
                        enemy_id,
                        remaining_health,
                    };
                    dispatch_game_event_to_all(runtime, &event);
                }
            }
            GameCommand::SpawnItem {
                item_type,
                x,
                y,
                z,
                request_id,
            } => {
                let position = Vec3::new(x, y, z);
                let _entity = spawn_item_entity(world, &item_type, position);
                let item_id = game_state.next_item_id;
                game_state.next_item_id += 1;

                game_state.items.insert(item_id, ItemData);

                let event = GameEvent::ItemSpawned {
                    request_id,
                    item_id,
                    item_type,
                };
                dispatch_game_event_to_plugin(runtime, plugin_id, &event);
            }
            GameCommand::GivePlayerItem { item_type } => {
                let event = GameEvent::ItemCollected {
                    item_id: 0,
                    item_type,
                };
                dispatch_game_event_to_all(runtime, &event);
            }
            GameCommand::SetPlayerHealth { health } => {
                game_state.player_health = health.min(game_state.player_max_health);
                let event = GameEvent::PlayerHealthChanged {
                    health: game_state.player_health,
                    max_health: game_state.player_max_health,
                };
                dispatch_game_event_to_all(runtime, &event);
            }
            GameCommand::SetPlayerScore { score } => {
                game_state.player_score = score;
                let event = GameEvent::PlayerScoreChanged { score };
                dispatch_game_event_to_all(runtime, &event);
            }
            GameCommand::TriggerGameEvent { event_name } => {
                if let Some(wave_str) = event_name
                    .strip_prefix("wave_")
                    .and_then(|s| s.strip_suffix("_start"))
                {
                    if let Ok(wave_number) = wave_str.parse::<u32>() {
                        let event = GameEvent::WaveStarted { wave_number };
                        dispatch_game_event_to_all(runtime, &event);
                    }
                } else if let Some(wave_str) = event_name
                    .strip_prefix("wave_")
                    .and_then(|s| s.strip_suffix("_complete"))
                {
                    if let Ok(wave_number) = wave_str.parse::<u32>() {
                        let event = GameEvent::WaveCompleted { wave_number };
                        dispatch_game_event_to_all(runtime, &event);
                    }
                } else {
                    let event = GameEvent::GameEventTriggered { event_name };
                    dispatch_game_event_to_all(runtime, &event);
                }
            }
        }
    }
}

#[cfg(feature = "plugins")]
fn get_enemy_max_health(enemy_type: &EnemyType) -> u32 {
    match enemy_type {
        EnemyType::Slime => 20,
        EnemyType::Skeleton => 50,
        EnemyType::Dragon => 200,
    }
}

#[cfg(feature = "plugins")]
fn spawn_enemy_entity(world: &mut World, enemy_type: &EnemyType, position: Vec3) -> Entity {
    match enemy_type {
        EnemyType::Slime => {
            let entity = spawn_sphere_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.5, 0.5, 0.5);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [0.2, 0.8, 0.2, 1.0];
                }
            }
            entity
        }
        EnemyType::Skeleton => {
            let entity = spawn_cylinder_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.3, 1.0, 0.3);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [0.9, 0.9, 0.85, 1.0];
                }
            }
            entity
        }
        EnemyType::Dragon => {
            let entity = spawn_cube_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(2.0, 2.0, 3.0);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [0.8, 0.2, 0.1, 1.0];
                }
            }
            entity
        }
    }
}

#[cfg(feature = "plugins")]
fn spawn_item_entity(world: &mut World, item_type: &ItemType, position: Vec3) -> Entity {
    match item_type {
        ItemType::HealthPotion => {
            let entity = spawn_sphere_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.3, 0.3, 0.3);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [1.0, 0.2, 0.2, 1.0];
                }
            }
            entity
        }
        ItemType::ManaPotion => {
            let entity = spawn_sphere_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.3, 0.3, 0.3);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [0.2, 0.2, 1.0, 1.0];
                }
            }
            entity
        }
        ItemType::Sword => {
            let entity = spawn_cube_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.1, 0.8, 0.1);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [0.7, 0.7, 0.8, 1.0];
                }
            }
            entity
        }
        ItemType::Shield => {
            let entity = spawn_cube_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.6, 0.6, 0.1);
            }
            if let Some(material_ref) = world.get_material_ref(entity).cloned() {
                if let Some(material) = world
                    .resources
                    .material_registry
                    .get_mut(&material_ref.name)
                {
                    material.base_color = [0.6, 0.4, 0.2, 1.0];
                }
            }
            entity
        }
    }
}
