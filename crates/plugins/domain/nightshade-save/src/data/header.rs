#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SaveHeader {
    pub name: String,
    pub version: u32,
    pub label: String,
}
