use std::marker::PhantomData;

pub struct BindingsFile<A> {
    pub folder: &'static str,
    pub name: &'static str,
    pub saved_revision: u64,
    pub seen_revision: u64,
    pub settle_seconds: f32,
    action: PhantomData<fn() -> A>,
}

impl<A> BindingsFile<A> {
    pub fn new(folder: &'static str, name: &'static str) -> Self {
        Self {
            folder,
            name,
            saved_revision: 0,
            seen_revision: 0,
            settle_seconds: 0.0,
            action: PhantomData,
        }
    }
}
