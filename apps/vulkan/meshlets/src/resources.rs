#[derive(Default)]
pub struct Pending {
    pub model: Option<nightshade_load::LoadHandle>,
}

#[derive(Default, Clone, Copy)]
pub struct Field {
    pub copies: usize,
    pub triangles: usize,
}
