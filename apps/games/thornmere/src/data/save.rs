pub const SAVE_FOLDER: &str = "nightshade-thornmere";
pub const SAVE_NAME: &str = "thornmere";
pub const SAVE_VERSION: u32 = 1;
pub const SAVE_SLOT: u32 = 0;
pub const SAVE_PERIOD: f32 = 20.0;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SavedQuest {
    pub quest: usize,
    pub progress: u32,
    pub complete: bool,
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SavedStack {
    pub slot: usize,
    pub item: usize,
    pub count: u32,
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Save {
    pub name: String,
    pub calling: usize,
    pub level: u32,
    pub experience: u32,
    pub kills: u32,
    pub purse: u32,
    pub hour: f32,
    pub spot: [f32; 3],
    pub bag: Vec<SavedStack>,
    pub worn: Vec<Option<usize>>,
    pub finished: Vec<bool>,
    pub active: Vec<SavedQuest>,
    pub discovered: Vec<bool>,
}

impl nightshade_save::Save for Save {
    const FOLDER: &'static str = SAVE_FOLDER;
    const NAME: &'static str = SAVE_NAME;
    const VERSION: u32 = SAVE_VERSION;
}
