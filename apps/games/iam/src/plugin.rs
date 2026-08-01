use crate::data::config::Config;
use crate::resources::AppResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_wgpu::RenderAppExt;
use nightshade_wgpu::backend::{
    PassPosition, render_graph_add_color_texture, render_graph_add_pass_at,
    render_graph_bind_pass_slot, render_graph_get_pass_mut,
};
use nightshade_wgpu::pre_render::Renderer;
use nightshade_wgpu::wgpu;

const INTERFACE_PASSES: [&str; 3] = ["viewport_compose_pass", "paint_pass", "paint_image_pass"];

fn install_screen(mut renderer: Renderer, config: Res<Config>) {
    if render_graph_get_pass_mut(&mut renderer.graph, "iam_screen").is_some() {
        return;
    }
    if render_graph_get_pass_mut(&mut renderer.graph, "paint_pass").is_none() {
        return;
    }

    let format = renderer.surface_format;
    let width = renderer.surface_config.width.max(1);
    let height = renderer.surface_config.height.max(1);
    let interface = render_graph_add_color_texture(&mut renderer.graph, "iam_interface")
        .format(format)
        .size(width, height)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        .clear_color(wgpu::Color::BLACK)
        .transient();

    for pass in INTERFACE_PASSES {
        let slot = if pass == "viewport_compose_pass" {
            "output"
        } else {
            "color"
        };
        if render_graph_get_pass_mut(&mut renderer.graph, pass).is_some()
            && let Err(error) =
                render_graph_bind_pass_slot(&mut renderer.graph, pass, slot, interface)
        {
            panic!("iam could not retarget {pass}: {error}");
        }
    }

    let config = *config;

    let mut logo = crate::data::intro::new(&renderer.device, format);
    crate::data::intro::configure(&mut logo, &config);
    render_graph_add_pass_at(
        &mut renderer.graph,
        Box::new(logo),
        &[("canvas", interface)],
        PassPosition::Last,
    )
    .expect("the logo pass");

    let mut pipes = crate::data::pipes::new(&renderer.device, format);
    crate::data::pipes::configure(&mut pipes, &config);
    render_graph_add_pass_at(
        &mut renderer.graph,
        Box::new(pipes),
        &[("canvas", interface)],
        PassPosition::Last,
    )
    .expect("the pipes pass");

    let mut phosphor = crate::data::phosphor::new(&renderer.device, format);
    crate::data::phosphor::configure(&mut phosphor, &config);
    render_graph_add_pass_at(
        &mut renderer.graph,
        Box::new(phosphor),
        &[("canvas", interface)],
        PassPosition::Last,
    )
    .expect("the phosphor pass");

    let swapchain = renderer.targets.swapchain;
    let mut screen = crate::data::screen::new(&renderer.device, format);
    crate::data::screen::configure(&mut screen, &config);
    render_graph_add_pass_at(
        &mut renderer.graph,
        Box::new(screen),
        &[("input", interface), ("output", swapchain)],
        PassPosition::Last,
    )
    .expect("the screen pass");
}

fn drive_intro(mut renderer: Renderer, resources: Res<AppResources>) {
    let (darkness, cover) = crate::systems::intro::fade(&resources);
    let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "iam_intro") else {
        return;
    };
    let Some(logo) =
        (pass as &mut dyn std::any::Any).downcast_mut::<crate::data::intro::IntroPass>()
    else {
        return;
    };
    crate::data::intro::set_fade(logo, darkness, cover);
}

fn drive_pipes(mut renderer: Renderer, resources: Res<AppResources>) {
    let cover = crate::systems::screensaver::fade(&resources);
    let grown = resources.boot.pipes_grown;
    let run = resources.boot.pipes.clone();
    let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "iam_pipes") else {
        return;
    };
    let Some(pipes) =
        (pass as &mut dyn std::any::Any).downcast_mut::<crate::data::pipes::PipesPass>()
    else {
        return;
    };
    crate::data::pipes::set_run(pipes, cover, grown, &run);
}

fn drive_lens(mut renderer: Renderer, resources: Res<AppResources>, config: Res<Config>) {
    let held = resources.glass.lens;
    let at = resources.glass.lens_at;
    let taken = resources.glass.shutoff;
    let night = resources.session.night;
    let nights = resources.script.nights.len().max(2);
    let config = *config;
    let zoom = config.lens_zoom;
    let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "iam_screen") else {
        return;
    };
    let Some(screen) =
        (pass as &mut dyn std::any::Any).downcast_mut::<crate::data::screen::ScreenPass>()
    else {
        return;
    };
    crate::data::screen::set_lens(screen, held, at, zoom);
    crate::data::screen::set_shutoff(screen, taken);
    crate::data::screen::set_wear(screen, &config, night as f32 / (nights - 1) as f32);
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let window = app.world.res_mut::<Window>();
        window.title = "iam".to_string();
        window.initial_size = Some((1180, 760));
        window.min_window_size = Some((880, 600));
        window.show_cursor = false;
        app.insert_resource(Config::default());
        app.insert_resource(AppResources::default());
        app.insert_resource(systems::desk::Gathered::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::ask::Ask>::new());
        app.add_plugin(nightshade_audio::AudioPlugin);
        app.add_system(Stage::Startup, systems::setup::dress_the_runtime);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::desk::gather_windows);
        app.add_system(Stage::Update, systems::desk::gather_held);
        app.add_system(Stage::Update, systems::desk::gather_desk);
        app.add_system(Stage::Update, systems::desk::gather_files);
        app.add_system(Stage::Update, systems::desk::gather_rows);
        app.add_system(Stage::Update, systems::conversation::gather_threads);
        app.add_system(Stage::Update, systems::intro::advance);
        app.add_system(Stage::Update, systems::screensaver::advance);
        app.add_system(Stage::Update, systems::ambience::start);
        app.add_system(Stage::Update, systems::ambience::click);
        app.add_system(Stage::Update, systems::ambience::cues);
        app.add_system(Stage::Update, systems::desk::take_the_screen);
        app.add_system(Stage::Update, systems::desk::move_pointer);
        app.add_system(Stage::Update, systems::desk::hold_lens);
        app.add_system(Stage::Update, systems::desk::run_alarm);
        app.add_system(Stage::Update, systems::desk::watch_tonight);
        app.add_system(Stage::Update, systems::desk::tick_clock);
        app.add_system(Stage::Update, systems::desk::wind_notices);
        app.add_system(Stage::Update, systems::desk::run_notices);
        app.add_system(Stage::Update, systems::browser::watch_width);
        app.add_system(Stage::Update, systems::desk::type_into_store);
        app.add_system(Stage::Update, systems::autoplay::drive);
        app.add_system(Stage::Update, systems::desk::run_toasts);
        app.add_system(Stage::Update, systems::desk::type_password);
        app.add_system(Stage::Update, systems::desk::blink_caret);
        app.add_system(Stage::Update, systems::browser::advance);
        app.add_system(Stage::Update, systems::desk::drag_windows);
        app.add_system(Stage::Update, systems::input::poll);
        app.add_system(Stage::Update, systems::conversation::advance);
        app.add_system(Stage::Update, systems::feeds::advance);
        app.add_system(Stage::Update, systems::interface::sync);
        app.add_system(Stage::Update, systems::desk::refresh);
        app.add_system(Stage::Update, systems::conversation::write_back);
        app.on_pre_render(install_screen);
        app.on_pre_render(drive_intro);
        app.on_pre_render(drive_pipes);
        app.on_pre_render(drive_lens);
    }
}
