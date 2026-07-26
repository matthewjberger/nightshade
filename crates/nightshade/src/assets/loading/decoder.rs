//! Parallel image decoding for the streaming loader.
//!
//! Image decoding is the dominant cost when streaming glTF textures. On native
//! the decoder fans encoded bytes out to a pool of worker threads so several
//! images decode at once across cores; the main thread only collects finished
//! RGBA and uploads it. On wasm there is a single thread and GPU calls must stay
//! on it, so decoding happens inline when results are polled and the per-frame
//! poll budget bounds the cost directly.

use super::{DecodedImage, SourceImageId, decode_to_rgba8};

/// A decoded source image returned from the decoder. `decoded` is `None` when
/// the encoded bytes failed to decode; recipes treat a missing source as a
/// neutral default. `generation` tags the load the request belonged to so the
/// drain can discard results from a scene that has since been replaced.
pub struct DecodeResult {
    pub image: SourceImageId,
    pub decoded: Option<DecodedImage>,
    pub generation: u64,
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use super::*;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::thread::JoinHandle;

    struct DecodeRequest {
        image: SourceImageId,
        encoded_bytes: Vec<u8>,
        generation: u64,
    }

    pub struct TextureDecoder {
        job_senders: Vec<Sender<DecodeRequest>>,
        results: std::sync::Mutex<Receiver<DecodeResult>>,
        workers: Vec<JoinHandle<()>>,
        next_worker: usize,
    }

    impl TextureDecoder {
        pub fn new() -> Self {
            let worker_count = std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1)
                .clamp(1, 8);
            let (result_sender, results) = channel::<DecodeResult>();
            let mut job_senders = Vec::with_capacity(worker_count);
            let mut workers = Vec::with_capacity(worker_count);
            for index in 0..worker_count {
                let (job_sender, job_receiver) = channel::<DecodeRequest>();
                job_senders.push(job_sender);
                let result_sender = result_sender.clone();
                let handle = std::thread::Builder::new()
                    .name(format!("nightshade-texture-decode-{index}"))
                    .spawn(move || {
                        for request in job_receiver {
                            let decoded = decode_to_rgba8(&request.encoded_bytes).ok();
                            drop(result_sender.send(DecodeResult {
                                image: request.image,
                                decoded,
                                generation: request.generation,
                            }));
                        }
                    })
                    .expect("spawn texture decode worker");
                workers.push(handle);
            }
            Self {
                job_senders,
                results: std::sync::Mutex::new(results),
                workers,
                next_worker: 0,
            }
        }

        pub fn submit(&mut self, image: SourceImageId, encoded_bytes: Vec<u8>, generation: u64) {
            let worker = self.next_worker % self.job_senders.len();
            self.next_worker = self.next_worker.wrapping_add(1);
            drop(self.job_senders[worker].send(DecodeRequest {
                image,
                encoded_bytes,
                generation,
            }));
        }

        pub fn poll(&mut self) -> Option<DecodeResult> {
            self.results.get_mut().unwrap().try_recv().ok()
        }
    }

    impl Drop for TextureDecoder {
        fn drop(&mut self) {
            self.job_senders.clear();
            for handle in self.workers.drain(..) {
                drop(handle.join());
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use super::*;
    use std::collections::VecDeque;

    struct DecodeRequest {
        image: SourceImageId,
        encoded_bytes: Vec<u8>,
        generation: u64,
    }

    pub struct TextureDecoder {
        pending: VecDeque<DecodeRequest>,
    }

    impl TextureDecoder {
        pub fn new() -> Self {
            Self {
                pending: VecDeque::new(),
            }
        }

        pub fn submit(&mut self, image: SourceImageId, encoded_bytes: Vec<u8>, generation: u64) {
            self.pending.push_back(DecodeRequest {
                image,
                encoded_bytes,
                generation,
            });
        }

        pub fn poll(&mut self) -> Option<DecodeResult> {
            let request = self.pending.pop_front()?;
            let decoded = decode_to_rgba8(&request.encoded_bytes).ok();
            Some(DecodeResult {
                image: request.image,
                decoded,
                generation: request.generation,
            })
        }
    }
}

pub use platform::TextureDecoder;
