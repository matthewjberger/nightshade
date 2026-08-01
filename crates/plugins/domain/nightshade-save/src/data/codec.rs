use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SaveCodec {
    #[default]
    Json,
    Bincode,
}

impl SaveCodec {
    pub fn encode<T: Serialize>(self, value: &T) -> Option<Vec<u8>> {
        match self {
            SaveCodec::Json => serde_json::to_vec_pretty(value).ok(),
            SaveCodec::Bincode => {
                bincode::serde::encode_to_vec(value, bincode::config::standard()).ok()
            }
        }
    }

    pub fn decode<T: DeserializeOwned>(self, payload: &[u8]) -> Option<T> {
        match self {
            SaveCodec::Json => serde_json::from_slice(payload).ok(),
            SaveCodec::Bincode => {
                bincode::serde::decode_from_slice(payload, bincode::config::standard())
                    .ok()
                    .map(|(value, _)| value)
            }
        }
    }
}
