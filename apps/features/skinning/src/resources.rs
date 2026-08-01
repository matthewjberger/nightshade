#[derive(Default)]
pub struct Model {
    pub asked: Option<nightshade_load::LoadHandle>,
    pub walk: Option<usize>,
    pub run: Option<usize>,
}

#[derive(Default)]
pub struct Gait {
    pub on_run: bool,
}

#[derive(Default)]
pub struct Watch {
    pub frame: u32,
    pub previous_palette_sum: f64,
}
