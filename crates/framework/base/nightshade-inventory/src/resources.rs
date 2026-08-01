pub struct StackLimit(pub u32);

impl Default for StackLimit {
    fn default() -> Self {
        Self(64)
    }
}
