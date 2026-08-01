#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateTransition<S> {
    pub before: Option<S>,
    pub after: S,
}
