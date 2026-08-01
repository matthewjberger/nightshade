use super::SourceImageId;
use nightshade_render::data::decoded_image::{BlockFamily, DecodedImage, decode_image};
use nightshade_worker::{Job, Workers, workers_hire, workers_submit, workers_take};

pub const DECODE_CREW: usize = 8;

fn decode(encoded_bytes: &[u8], family: BlockFamily) -> Option<DecodedImage> {
    decode_image(encoded_bytes, family)
        .inspect_err(|error| tracing::error!("a texture did not decode: {error}"))
        .ok()
}

pub struct DecodeJob {
    image: SourceImageId,
    encoded_bytes: Vec<u8>,
    family: BlockFamily,
    generation: u64,
}

pub struct DecodeResult {
    pub image: SourceImageId,
    pub decoded: Option<DecodedImage>,
    pub generation: u64,
}

impl Job for DecodeJob {
    type Key = (SourceImageId, u64);
    type Done = DecodeResult;
    type Tools = ();

    fn key(&self) -> Self::Key {
        (self.image, self.generation)
    }

    fn run(self, _tools: &mut Self::Tools) -> Self::Done {
        DecodeResult {
            image: self.image,
            decoded: decode(&self.encoded_bytes, self.family),
            generation: self.generation,
        }
    }
}

pub struct TextureDecoder {
    crew: Workers<DecodeJob>,
}

impl TextureDecoder {
    pub fn new() -> Self {
        Self {
            crew: workers_hire::<DecodeJob>(DECODE_CREW),
        }
    }

    pub fn submit(
        &mut self,
        image: SourceImageId,
        encoded_bytes: Vec<u8>,
        family: BlockFamily,
        generation: u64,
    ) {
        let job = DecodeJob {
            image,
            encoded_bytes,
            family,
            generation,
        };
        workers_submit(&mut self.crew, job, 0);
    }

    pub fn take(&mut self, budget: usize) -> Vec<DecodeResult> {
        workers_take(&mut self.crew, budget)
    }
}
