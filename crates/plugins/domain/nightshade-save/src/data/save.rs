use crate::data::codec::SaveCodec;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Save: Serialize + DeserializeOwned {
    const FOLDER: &'static str;
    const NAME: &'static str;
    const VERSION: u32;
    const SLOTS: u32 = 1;
    const CODEC: SaveCodec = SaveCodec::Json;

    fn migrate(from: u32, payload: &[u8]) -> Option<Self> {
        if from > Self::VERSION {
            return None;
        }
        Self::CODEC.decode(payload)
    }
}
