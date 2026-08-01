pub struct State<S> {
    pub current: S,
    pub initialized: bool,
}

pub struct NextState<S> {
    pub pending: Option<S>,
}
