#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deed {
    Look,
    Held,
    Go(usize),
    Take(usize),
    Drop(usize),
    Study(usize),
    Follow(usize),
}

#[derive(Clone, Debug)]
pub struct Link {
    pub words: String,
    pub deed: Deed,
}
