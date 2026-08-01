use std::collections::HashSet;

#[derive(Default)]
pub struct Den {
    pub kept: HashSet<usize>,
    pub named: HashSet<usize>,
    pub pointed: HashSet<usize>,
    pub hoarded: HashSet<usize>,
    pub waiting: HashSet<usize>,
    pub freed: HashSet<usize>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Counting(pub f32);
