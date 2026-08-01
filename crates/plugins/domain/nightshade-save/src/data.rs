mod autosave;
mod codec;
mod header;
mod save;

pub use autosave::SaveDue;
pub use codec::SaveCodec;
pub use header::SaveHeader;
pub use save::Save;

pub const HEADER_TAG: &str = "nightshade-save";

pub const FIELD_SEPARATOR: char = '\t';

pub const HEADER_FIELDS: usize = 4;

pub const FILE_SUFFIX: &str = ".save";
