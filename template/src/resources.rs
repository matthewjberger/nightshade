/// App-wide state. Systems read and mutate these fields directly; grow the
/// struct, or split it into more resources, as your game does.
#[derive(Default)]
pub struct TemplateResources {
    pub example: ExampleState,
}

/// Example resource. Resources are global per-app state that systems read and
/// mutate. Replace, rename, or add more as your game grows.
#[derive(Default)]
pub struct ExampleState {
    pub frame_count: u64,
    pub elapsed_seconds: f32,
}
