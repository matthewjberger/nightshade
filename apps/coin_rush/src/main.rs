use nightshade_api::prelude::*;

const ARENA: f32 = 14.0;
const ROUND_SECONDS: f32 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    Playing,
    Paused,
    Over,
}

struct Game {
    screen: Screen,
    player: Entity,
    coins: Vec<(Entity, Vec3)>,
    round: u32,
    goal: u32,
    score: u32,
    time: f32,
    seed: u32,
    hud: Entity,
    title: Entity,
    play_button: Entity,
    quit_button: Entity,
    pause: Entity,
    resume_button: Entity,
    restart_button: Entity,
    pause_title_button: Entity,
    over: Entity,
    over_score: Entity,
    again_button: Entity,
    over_title_button: Entity,
}

fn random_unit(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 16) as f32 / 65_535.0 * 2.0 - 1.0
}

fn random_spot(seed: &mut u32) -> Vec3 {
    vec3(random_unit(seed) * ARENA, 0.7, random_unit(seed) * ARENA)
}

/// Switches screen and shows only that screen's overlay; the HUD rides along on
/// every screen but the title.
fn set_screen(world: &mut World, game: &mut Game, screen: Screen) {
    game.screen = screen;
    set_panel_visible(world, game.title, screen == Screen::Title);
    set_panel_visible(world, game.pause, screen == Screen::Paused);
    set_panel_visible(world, game.over, screen == Screen::Over);
    set_panel_visible(world, game.hud, screen != Screen::Title);
}

/// Resets the field for the current round: score to zero, a fresh timer, the
/// round's coin goal, and coins scattered to new spots. The animate pass moves
/// the coins to their new homes, so this only touches game state.
fn begin_round(game: &mut Game) {
    game.score = 0;
    game.goal = 4 + game.round * 2;
    game.time = ROUND_SECONDS;
    for index in 0..game.coins.len() {
        game.coins[index].1 = random_spot(&mut game.seed);
    }
}

/// Starts a fresh game at round one and drops the player back in the center.
fn start_round(world: &mut World, game: &mut Game) {
    game.round = 1;
    begin_round(game);
    set_position(world, game.player, vec3(0.0, 0.5, 0.0));
    set_screen(world, game, Screen::Playing);
}

fn setup(world: &mut World) -> Game {
    set_window_title(world, "Coin Rush");
    let floor = spawn_floor(world, ARENA + 2.0);
    load_texture(
        world,
        "proto_dark_06",
        include_bytes!("../../../assets/textures/prototype/dark/texture_06.png"),
    );
    set_texture(world, floor, "proto_dark_06");
    set_texture_tiling(world, floor, 8.0);
    fixed_camera(world, vec3(0.0, 24.0, 16.0), vec3(0.0, 0.0, 0.0));

    let player = spawn_object(
        world,
        Object {
            shape: Shape::Sphere,
            position: vec3(0.0, 0.5, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
            color: TEAL,
            body: Body::Static,
        },
    );
    let mut seed = 0x1234_5678;
    let coins = (0..8)
        .map(|_| {
            let spot = random_spot(&mut seed);
            let coin = spawn_object(
                world,
                Object {
                    shape: Shape::Cylinder,
                    position: spot,
                    scale: vec3(0.7, 0.12, 0.7),
                    color: GOLD,
                    body: Body::Static,
                },
            );
            // A squashed cylinder tipped onto its side: a coin standing on edge.
            rotate(world, coin, Vec3::x(), std::f32::consts::FRAC_PI_2);
            (coin, spot)
        })
        .collect();
    let hud = spawn_text(world, "Score 0   Time 30", ScreenAnchor::TopCenter);

    let title = spawn_panel(world, ScreenAnchor::Center, 460.0, 220.0);
    let title_heading = panel_heading(world, title, "Coin Rush");
    set_text_size(world, title_heading, 36.0);
    panel_label(world, title, "Reach the coin goal before time runs out.");
    panel_label(world, title, "WASD or arrows to move. P to pause.");
    let play_button = panel_button(world, title, "Play");
    let quit_button = panel_button(world, title, "Quit");

    let pause = spawn_panel(world, ScreenAnchor::Center, 300.0, 210.0);
    let pause_heading = panel_heading(world, pause, "Paused");
    set_text_size(world, pause_heading, 32.0);
    let resume_button = panel_button(world, pause, "Resume");
    let restart_button = panel_button(world, pause, "Restart");
    let pause_title_button = panel_button(world, pause, "Quit to title");

    let over = spawn_panel(world, ScreenAnchor::Center, 300.0, 210.0);
    let over_heading = panel_heading(world, over, "Time!");
    set_text_size(world, over_heading, 32.0);
    let over_score = panel_label(world, over, "Score 0");
    let again_button = panel_button(world, over, "Play again");
    let over_title_button = panel_button(world, over, "Quit to title");

    let mut game = Game {
        screen: Screen::Title,
        player,
        coins,
        round: 1,
        goal: 6,
        score: 0,
        time: ROUND_SECONDS,
        seed,
        hud,
        title,
        play_button,
        quit_button,
        pause,
        resume_button,
        restart_button,
        pause_title_button,
        over,
        over_score,
        again_button,
        over_title_button,
    };
    set_screen(world, &mut game, Screen::Title);
    game
}

/// The menu and pause flow: button clicks on whichever overlay is up, plus P to
/// toggle pause while playing.
fn menus(world: &mut World, game: &mut Game) {
    match game.screen {
        Screen::Title => {
            if button_clicked(world, game.play_button) {
                start_round(world, game);
            } else if button_clicked(world, game.quit_button) {
                request_exit(world);
            }
        }
        Screen::Playing => {
            if key_pressed(world, KeyCode::KeyP) {
                set_screen(world, game, Screen::Paused);
            }
        }
        Screen::Paused => {
            if button_clicked(world, game.resume_button) || key_pressed(world, KeyCode::KeyP) {
                set_screen(world, game, Screen::Playing);
            } else if button_clicked(world, game.restart_button) {
                start_round(world, game);
            } else if button_clicked(world, game.pause_title_button) {
                set_screen(world, game, Screen::Title);
            }
        }
        Screen::Over => {
            if button_clicked(world, game.again_button) {
                start_round(world, game);
            } else if button_clicked(world, game.over_title_button) {
                set_screen(world, game, Screen::Title);
            }
        }
    }
}

/// Spins and bobs every coin on every screen, then, while playing, moves the
/// player, scores coins toward the round goal, advances a round on reaching it,
/// and ends the game when the timer runs out first.
fn play(world: &mut World, game: &mut Game) {
    let dt = delta_time(world);
    let clock = elapsed_seconds(world);
    for index in 0..game.coins.len() {
        let (coin, home) = game.coins[index];
        let bob = (clock * 1.5 + index as f32 * 0.7).sin() * 0.25;
        set_position(world, coin, vec3(home.x, home.y + bob, home.z));
        rotate(world, coin, Vec3::y(), dt * 3.0);
    }
    if game.screen != Screen::Playing {
        return;
    }

    let speed = 10.0;
    let mut p = position(world, game.player);
    if key_down(world, KeyCode::KeyA) || key_down(world, KeyCode::ArrowLeft) {
        p.x -= speed * dt;
    }
    if key_down(world, KeyCode::KeyD) || key_down(world, KeyCode::ArrowRight) {
        p.x += speed * dt;
    }
    if key_down(world, KeyCode::KeyW) || key_down(world, KeyCode::ArrowUp) {
        p.z -= speed * dt;
    }
    if key_down(world, KeyCode::KeyS) || key_down(world, KeyCode::ArrowDown) {
        p.z += speed * dt;
    }
    p.x = p.x.clamp(-ARENA, ARENA);
    p.z = p.z.clamp(-ARENA, ARENA);
    set_position(world, game.player, p);

    for index in 0..game.coins.len() {
        let home = game.coins[index].1;
        if (home.x - p.x).abs() < 1.0 && (home.z - p.z).abs() < 1.0 {
            game.score += 1;
            game.coins[index].1 = random_spot(&mut game.seed);
        }
    }

    if game.score >= game.goal {
        game.round += 1;
        begin_round(game);
    }

    game.time = (game.time - dt).max(0.0);
    set_text(
        world,
        game.hud,
        &format!(
            "Round {}   {}/{}   Time {:.0}",
            game.round,
            game.score,
            game.goal,
            game.time.ceil()
        ),
    );
    if game.time <= 0.0 {
        let summary = format!("Reached round {}", game.round);
        set_text(world, game.over_score, &summary);
        set_screen(world, game, Screen::Over);
    }
}

fn main() {
    run!(setup, menus, play).unwrap();
}
