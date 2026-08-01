use crate::theme;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum CropKind {
    #[default]
    Carrot,
    Wheat,
}

#[derive(Clone, Copy, Debug)]
pub struct CropInfo {
    pub kind: CropKind,
    pub feeds: f32,
    pub name: &'static str,
    pub seed_name: &'static str,
    pub stages: [u32; 4],
    pub grow_seconds: f32,
}

pub const CROPS: [CropInfo; 2] = [
    CropInfo {
        kind: CropKind::Carrot,
        feeds: theme::CARROT_FOOD,
        name: "Carrot",
        seed_name: "Carrot Seeds",
        stages: [5, 6, 7, 8],
        grow_seconds: 50.0,
    },
    CropInfo {
        kind: CropKind::Wheat,
        feeds: theme::WHEAT_FOOD,
        name: "Wheat",
        seed_name: "Wheat Seeds",
        stages: [64, 65, 66, 67],
        grow_seconds: 40.0,
    },
];

impl CropKind {
    pub fn feeds(self) -> f32 {
        CROPS[self as usize].feeds
    }

    pub fn info(self) -> CropInfo {
        CROPS[self as usize]
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: u8) -> Option<CropKind> {
        CROPS.get(index as usize).map(|row| row.kind)
    }
}

impl From<CropKind> for u8 {
    fn from(value: CropKind) -> Self {
        value.index() as u8
    }
}

impl TryFrom<u8> for CropKind {
    type Error = &'static str;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        CropKind::from_index(code).ok_or("unknown crop")
    }
}

const fn crops_line_up() -> bool {
    let mut index = 0;
    while index < CROPS.len() {
        if CROPS[index].kind as usize != index {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(
    crops_line_up(),
    "a CROPS row does not sit at the index of its CropKind"
);
