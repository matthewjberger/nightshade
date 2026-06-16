use nightshade_api::prelude::*;

const ARENA: f32 = 55.0;
const ROUND_SECONDS: f32 = 50.0;
const PROP_COUNT: usize = 64;
const COIN_COUNT: usize = 14;
const PICKUP_RADIUS: f32 = 2.5;
const COIN_HEIGHT: f32 = 1.4;
const PLAYER_SPEED: f32 = 9.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    Playing,
    Paused,
    Over,
}

struct Theme {
    ground: &'static str,
    hill: [f32; 4],
    foliage: [f32; 4],
}

const THEMES: [Theme; 3] = [
    Theme {
        ground: "ground_green",
        hill: rgb(0.45, 0.35, 0.22),
        foliage: GREEN,
    },
    Theme {
        ground: "ground_orange",
        hill: rgb(0.72, 0.56, 0.32),
        foliage: rgb(0.5, 0.6, 0.25),
    },
    Theme {
        ground: "ground_dark",
        hill: GRAY,
        foliage: rgb(0.62, 0.72, 0.82),
    },
];

struct Game {
    screen: Screen,
    player: Option<Entity>,
    floor: Entity,
    props: Vec<Entity>,
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

fn next_random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 16) as f32 / 65_535.0
}

fn scatter(seed: &mut u32, min_radius: f32, max_radius: f32) -> (f32, f32) {
    let angle = next_random(seed) * std::f32::consts::TAU;
    let radius = min_radius + next_random(seed) * (max_radius - min_radius);
    (angle.cos() * radius, angle.sin() * radius)
}

fn spawn_prop(world: &mut World, seed: &mut u32, theme: &Theme) -> Entity {
    let (x, z) = scatter(seed, 8.0, ARENA);
    let (shape, scale, color) = match (next_random(seed) * 4.0) as u32 {
        0 => {
            let width = 2.0 + next_random(seed) * 5.0;
            let height = 1.0 + next_random(seed) * 3.0;
            (Shape::Cube, vec3(width, height, width), theme.hill)
        }
        1 => {
            let height = 3.0 + next_random(seed) * 6.0;
            (Shape::Cylinder, vec3(1.2, height, 1.2), STEEL)
        }
        2 => {
            let width = 1.5 + next_random(seed) * 2.0;
            let height = 3.0 + next_random(seed) * 4.0;
            (Shape::Cone, vec3(width, height, width), theme.foliage)
        }
        _ => {
            let radius = 1.0 + next_random(seed) * 2.5;
            (Shape::Sphere, vec3(radius, radius, radius), GRAY)
        }
    };
    spawn_object(
        world,
        Object {
            shape,
            position: vec3(x, scale.y * 0.5, z),
            scale,
            color,
            body: Body::Static,
        },
    )
}

fn spawn_coin(world: &mut World) -> Entity {
    let coin = spawn_object(
        world,
        Object {
            shape: Shape::Cylinder,
            position: vec3(0.0, COIN_HEIGHT, 0.0),
            scale: vec3(0.7, 0.12, 0.7),
            color: GOLD,
            body: Body::None,
        },
    );
    rotate(world, coin, Vec3::x(), std::f32::consts::FRAC_PI_2);
    coin
}

/// Tears down the previous world and grows a fresh one for the current round: a
/// new ground texture, a new scatter of props, and coins relocated to new spots.
fn generate_world(world: &mut World, game: &mut Game) {
    let theme = &THEMES[((game.round - 1) % 3) as usize];
    set_texture(world, game.floor, theme.ground);
    set_texture_tiling(world, game.floor, 14.0);
    for prop in game.props.drain(..).collect::<Vec<_>>() {
        despawn(world, prop);
    }
    let mut seed = game.seed;
    for _ in 0..PROP_COUNT {
        let prop = spawn_prop(world, &mut seed, theme);
        game.props.push(prop);
    }
    for index in 0..game.coins.len() {
        let (x, z) = scatter(&mut seed, 5.0, ARENA * 0.85);
        game.coins[index].1 = vec3(x, COIN_HEIGHT, z);
    }
    game.seed = seed;
}

/// Shows the overlay for the current screen and the HUD only while in a round.
fn refresh_panels(world: &mut World, game: &Game) {
    set_panel_visible(world, game.title, game.screen == Screen::Title);
    set_panel_visible(world, game.pause, game.screen == Screen::Paused);
    set_panel_visible(world, game.over, game.screen == Screen::Over);
    set_panel_visible(
        world,
        game.hud,
        matches!(game.screen, Screen::Playing | Screen::Paused),
    );
}

/// Drops into a fresh first person run at round one, spawning the player and
/// locking the cursor for mouse look.
fn start_game(world: &mut World, game: &mut Game) {
    game.round = 1;
    game.score = 0;
    game.goal = 5 + game.round * 3;
    game.time = ROUND_SECONDS;
    generate_world(world, game);
    let player = first_person(world, vec3(0.0, 2.0, 0.0));
    set_player_speed(world, player, PLAYER_SPEED);
    game.player = Some(player);
    game.screen = Screen::Playing;
    refresh_panels(world, game);
}

/// Leaves the run for a free standing camera with the cursor released, the shared
/// exit from a round to the title or the game over screen.
fn leave_run(world: &mut World, game: &mut Game, screen: Screen) {
    game.player = None;
    fixed_camera(world, vec3(0.0, 20.0, 34.0), vec3(0.0, 2.0, 0.0));
    lock_cursor(world, false);
    game.screen = screen;
    refresh_panels(world, game);
}

fn setup(world: &mut World) -> Game {
    set_title(world, "Coin Dash");
    set_background(world, Background::Sky);
    set_fog(
        world,
        Some(Fog {
            color: [0.7, 0.8, 0.92],
            start: 28.0,
            end: ARENA * 1.9,
        }),
    );
    set_sun(world, [1.0, 0.96, 0.9], 3.2);

    let floor = spawn_floor(world, ARENA + 10.0);
    load_texture(
        world,
        "ground_green",
        include_bytes!("../../../assets/textures/prototype/green/texture_04.png"),
    );
    load_texture(
        world,
        "ground_orange",
        include_bytes!("../../../assets/textures/prototype/orange/texture_04.png"),
    );
    load_texture(
        world,
        "ground_dark",
        include_bytes!("../../../assets/textures/prototype/dark/texture_06.png"),
    );
    let coins = (0..COIN_COUNT)
        .map(|_| (spawn_coin(world), vec3(0.0, COIN_HEIGHT, 0.0)))
        .collect();
    let hud = spawn_text(world, "", ScreenAnchor::TopCenter);

    let title = spawn_panel(world, ScreenAnchor::Center, 540.0, 240.0);
    let title_heading = panel_heading(world, title, "Coin Dash");
    set_text_size(world, title_heading, 36.0);
    panel_label(
        world,
        title,
        "Run through procedural worlds and grab every coin.",
    );
    panel_label(
        world,
        title,
        "WASD and mouse to move, Shift to sprint, Space to jump. P to pause.",
    );
    let play_button = panel_button(world, title, "Play");
    let quit_button = panel_button(world, title, "Quit");

    let pause = spawn_panel(world, ScreenAnchor::Center, 300.0, 210.0);
    let pause_heading = panel_heading(world, pause, "Paused");
    set_text_size(world, pause_heading, 32.0);
    let resume_button = panel_button(world, pause, "Resume");
    let restart_button = panel_button(world, pause, "Restart");
    let pause_title_button = panel_button(world, pause, "Quit to title");

    let over = spawn_panel(world, ScreenAnchor::Center, 340.0, 210.0);
    let over_heading = panel_heading(world, over, "Time!");
    set_text_size(world, over_heading, 32.0);
    let over_score = panel_label(world, over, "");
    let again_button = panel_button(world, over, "Play again");
    let over_title_button = panel_button(world, over, "Quit to title");

    let mut game = Game {
        screen: Screen::Title,
        player: None,
        floor,
        props: Vec::new(),
        coins,
        round: 1,
        goal: 8,
        score: 0,
        time: ROUND_SECONDS,
        seed: 0x9E37_79B9,
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
    generate_world(world, &mut game);
    fixed_camera(world, vec3(0.0, 20.0, 34.0), vec3(0.0, 2.0, 0.0));
    lock_cursor(world, false);
    refresh_panels(world, &game);
    game
}

/// Menu flow and pause toggling for whichever overlay is up.
fn menus(world: &mut World, game: &mut Game) {
    match game.screen {
        Screen::Title => {
            if button_clicked(world, game.play_button) {
                start_game(world, game);
            } else if button_clicked(world, game.quit_button) {
                request_exit(world);
            }
        }
        Screen::Playing => {
            if key_pressed(world, KeyCode::KeyP) {
                lock_cursor(world, false);
                game.screen = Screen::Paused;
                refresh_panels(world, game);
            }
        }
        Screen::Paused => {
            if button_clicked(world, game.resume_button) || key_pressed(world, KeyCode::KeyP) {
                lock_cursor(world, true);
                game.screen = Screen::Playing;
                refresh_panels(world, game);
            } else if button_clicked(world, game.restart_button) {
                start_game(world, game);
            } else if button_clicked(world, game.pause_title_button) {
                leave_run(world, game, Screen::Title);
            }
        }
        Screen::Over => {
            if button_clicked(world, game.again_button) {
                start_game(world, game);
            } else if button_clicked(world, game.over_title_button) {
                leave_run(world, game, Screen::Title);
            }
        }
    }
}

/// Spins and bobs the coins everywhere, then, in a round, scores pickups, grows
/// the world when the goal is met, and ends the run when the timer empties.
fn play(world: &mut World, game: &mut Game) {
    let dt = delta_time(world);
    let clock = elapsed_seconds(world);
    for index in 0..game.coins.len() {
        let (coin, home) = game.coins[index];
        let bob = (clock * 1.5 + index as f32 * 0.7).sin() * 0.2;
        set_position(world, coin, vec3(home.x, home.y + bob, home.z));
        rotate(world, coin, Vec3::y(), dt * 3.0);
    }
    if game.screen != Screen::Playing {
        return;
    }
    let Some(player) = game.player else {
        return;
    };

    let here = position(world, player);
    for index in 0..game.coins.len() {
        let home = game.coins[index].1;
        if (home.x - here.x).hypot(home.z - here.z) < PICKUP_RADIUS {
            game.score += 1;
            let (x, z) = scatter(&mut game.seed, 5.0, ARENA * 0.85);
            game.coins[index].1 = vec3(x, COIN_HEIGHT, z);
        }
    }

    if game.score >= game.goal {
        game.round += 1;
        game.score = 0;
        game.goal = 5 + game.round * 3;
        game.time = ROUND_SECONDS;
        generate_world(world, game);
    }

    game.time = (game.time - dt).max(0.0);
    set_text(
        world,
        game.hud,
        &format!(
            "World {}   {}/{}   {:.0}s",
            game.round,
            game.score,
            game.goal,
            game.time.ceil()
        ),
    );
    if game.time <= 0.0 {
        let summary = format!("You reached world {}", game.round);
        set_text(world, game.over_score, &summary);
        leave_run(world, game, Screen::Over);
    }
}

fn main() {
    run!(setup, menus, play).unwrap();
}
