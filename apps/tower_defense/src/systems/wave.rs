use crate::ecs::{EnemySpawnInfo, EnemyType, GameState, GameWorld};
use crate::systems::spawn_enemy;
use nightshade::prelude::*;

pub fn plan_wave(game_world: &mut GameWorld) {
    game_world.resources.wave += 1;
    let wave = game_world.resources.wave;
    let mut spawns = Vec::new();

    let is_boss_wave = wave.is_multiple_of(5);
    let enemy_count = 5 + wave * 2;

    let enemy_types = if is_boss_wave {
        match wave {
            5 => vec![
                (EnemyType::Normal, 0.3),
                (EnemyType::Fast, 0.3),
                (EnemyType::Tank, 0.2),
                (EnemyType::Boss, 0.2),
            ],
            10 => vec![
                (EnemyType::Normal, 0.25),
                (EnemyType::Fast, 0.25),
                (EnemyType::Tank, 0.15),
                (EnemyType::Flying, 0.15),
                (EnemyType::Boss, 0.2),
            ],
            15 => vec![
                (EnemyType::Normal, 0.2),
                (EnemyType::Fast, 0.2),
                (EnemyType::Tank, 0.15),
                (EnemyType::Flying, 0.15),
                (EnemyType::Shielded, 0.1),
                (EnemyType::Boss, 0.2),
            ],
            _ => vec![
                (EnemyType::Normal, 0.15),
                (EnemyType::Fast, 0.15),
                (EnemyType::Tank, 0.15),
                (EnemyType::Flying, 0.15),
                (EnemyType::Shielded, 0.1),
                (EnemyType::Healer, 0.1),
                (EnemyType::Boss, 0.2),
            ],
        }
    } else {
        match wave {
            1..=2 => vec![(EnemyType::Normal, 1.0)],
            3..=4 => vec![(EnemyType::Normal, 0.7), (EnemyType::Fast, 0.3)],
            6..=9 => vec![
                (EnemyType::Normal, 0.4),
                (EnemyType::Fast, 0.3),
                (EnemyType::Tank, 0.2),
                (EnemyType::Flying, 0.1),
            ],
            11..=14 => vec![
                (EnemyType::Normal, 0.3),
                (EnemyType::Fast, 0.25),
                (EnemyType::Tank, 0.2),
                (EnemyType::Flying, 0.15),
                (EnemyType::Shielded, 0.1),
            ],
            _ => vec![
                (EnemyType::Normal, 0.25),
                (EnemyType::Fast, 0.2),
                (EnemyType::Tank, 0.2),
                (EnemyType::Flying, 0.15),
                (EnemyType::Shielded, 0.1),
                (EnemyType::Healer, 0.1),
            ],
        }
    };

    let spawn_interval = match wave {
        1..=3 => 1.0,
        4..=6 => 0.8,
        _ => 0.6,
    };

    let mut rng = rand::rng();
    let mut spawn_time = 0.0;

    for _ in 0..enemy_count {
        let roll: f32 = rng.random();
        let mut cumulative = 0.0;
        let mut selected_type = EnemyType::Normal;

        for (enemy_type, probability) in &enemy_types {
            cumulative += probability;
            if roll < cumulative {
                selected_type = *enemy_type;
                break;
            }
        }

        spawns.push(EnemySpawnInfo {
            enemy_type: selected_type,
            spawn_time,
        });
        spawn_time += spawn_interval;
    }

    game_world.resources.enemies_to_spawn = spawns;
    game_world.resources.spawn_timer = 0.0;
    game_world.resources.game_state = GameState::WaveInProgress;
    game_world.resources.wave_announce_timer = if is_boss_wave { 3.0 } else { 2.0 };
}

pub fn wave_spawning_system(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game_world.resources.game_speed;

    if game_world.resources.game_state == GameState::WaitingForWave {
        if game_world.resources.wave_delay > 0.0 {
            game_world.resources.wave_delay -= delta_time;
        } else {
            plan_wave(game_world);
        }
        return;
    }

    if game_world.resources.game_state != GameState::WaveInProgress {
        return;
    }

    game_world.resources.spawn_timer += delta_time;

    let current_time = game_world.resources.spawn_timer;
    let mut spawns_to_process = Vec::new();

    for (index, spawn_info) in game_world.resources.enemies_to_spawn.iter().enumerate() {
        if spawn_info.spawn_time <= current_time {
            spawns_to_process.push((index, spawn_info.enemy_type));
        }
    }

    for (_index, enemy_type) in spawns_to_process.iter() {
        spawn_enemy(game_world, world, *enemy_type);
    }

    for &(index, _) in spawns_to_process.iter().rev() {
        game_world.resources.enemies_to_spawn.remove(index);
    }

    if game_world.resources.enemies_to_spawn.is_empty()
        && game_world.resources.enemies_list.is_empty()
    {
        game_world.resources.game_state = GameState::WaitingForWave;
        game_world.resources.wave_delay = 3.0;
    }
}
