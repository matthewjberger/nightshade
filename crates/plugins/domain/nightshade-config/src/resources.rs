use std::marker::PhantomData;

pub struct ConfigFile<S> {
    pub folder: &'static str,
    pub name: &'static str,
    pub saved_revision: u64,
    pub seen_revision: u64,
    pub settle_seconds: f32,
    schema: PhantomData<fn() -> S>,
}

impl<S> ConfigFile<S> {
    pub fn new(folder: &'static str, name: &'static str) -> Self {
        Self {
            folder,
            name,
            saved_revision: 0,
            seen_revision: 0,
            settle_seconds: 0.0,
            schema: PhantomData,
        }
    }
}
