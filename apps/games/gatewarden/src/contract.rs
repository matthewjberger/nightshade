#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Map,
    Scene,
    Battle,
    Shop,
    Over,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    #[default]
    Lost,
    Won,
}
