pub trait Job: Send + 'static {
    type Key: Copy + Eq + std::hash::Hash + Send + Sync + 'static;
    type Done: Send + 'static;
    type Tools: Default + 'static;

    fn key(&self) -> Self::Key;
    fn run(self, tools: &mut Self::Tools) -> Self::Done;
}

pub const CREW_DIVISOR: usize = 2;
