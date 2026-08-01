use std::collections::HashSet;

#[derive(Default)]
pub struct Crew {
    pub served: HashSet<[i32; 2]>,
    pub guided: HashSet<[i32; 2]>,
    pub perched: HashSet<[i32; 2]>,
    pub gated: HashSet<[i32; 2]>,
}
