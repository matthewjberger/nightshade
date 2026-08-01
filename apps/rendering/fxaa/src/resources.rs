use nightshade_cli::clap;

#[derive(clap::Args, Default)]
pub struct FxaaArguments {
    #[arg(long, help = "Open with the smoothing switch off")]
    pub off: bool,
    #[arg(
        long,
        help = "Have the interface press the switch part way through the run"
    )]
    pub toggle: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Panel {
    pub smoothing: nightshade::ecs::Entity,
    pub pixels: nightshade::ecs::Entity,
}

#[derive(Default)]
pub struct Opening {
    pub smoothing_on: bool,
    pub driver: Option<nightshade_ui_test::UiTestDriver>,
}
