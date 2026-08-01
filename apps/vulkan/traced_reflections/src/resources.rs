#[derive(Default)]
pub struct Pending {
    pub model: Option<nightshade_load::LoadHandle>,
    pub masked: Option<nightshade_load::LoadHandle>,
}
