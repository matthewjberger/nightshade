//! GPU frame timing, from the graphics side rather than the processor's.
//!
//! The processor only ever measures how long it took to *describe* a frame:
//! `submit` hands the work off and returns, and the graphics side is typically
//! running a frame or more behind. A wall-clock timer around the graph will
//! happily report a fraction of a millisecond while the device is taking twenty.
//! The only honest answer comes from the device stamping a counter itself.

/// A timestamp pair spanning everything a frame submits, read back a frame or
/// two later.
///
/// Late on purpose: resolving a query in the frame that wrote it means waiting
/// for the queue to drain, which turns the measurement into the thing being
/// measured. What the reading loses in latency it keeps in honesty, and a cost
/// that moves over tens of frames reads the same either way.
///
/// Absent whenever the adapter cannot stamp timestamps from an encoder, in which
/// case every method is a no-op and the reading stays at zero rather than
/// reporting a fabricated one.
#[derive(Default)]
pub struct GpuTiming {
    queries: Option<Queries>,
    /// Milliseconds the device spent on the last frame it resolved.
    pub milliseconds: f32,
}

struct Queries {
    set: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readback: wgpu::Buffer,
    period: f32,
    stage: Stage,
    /// Raised by the map callback once the readback is legible, and shared
    /// because the callback outlives the call that installed it.
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// Where the one outstanding readback has got to. Exactly one is ever in
/// flight, which is what keeps this from queueing maps faster than they retire.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Idle,
    Resolved,
    Mapping,
}

impl Drop for Queries {
    /// Cancels an outstanding map, since dropping a mapped buffer is an error.
    fn drop(&mut self) {
        if self.stage == Stage::Mapping {
            self.readback.unmap();
        }
    }
}

impl GpuTiming {
    /// Builds the query set, or nothing when the adapter cannot stamp a
    /// timestamp from a command encoder.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let features = device.features();
        if !features.contains(wgpu::Features::TIMESTAMP_QUERY)
            || !features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
        {
            return Self::default();
        }
        Self {
            queries: Some(Queries {
                set: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("frame_timing"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 2,
                }),
                resolved: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("frame_timing_resolved"),
                    size: 16,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                readback: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("frame_timing_readback"),
                    size: 16,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                period: queue.get_timestamp_period(),
                stage: Stage::Idle,
                ready: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }),
            milliseconds: 0.0,
        }
    }

    /// A command buffer that opens the span, to submit ahead of the frame's own.
    ///
    /// Its own buffer rather than a write into the graph's, because submission
    /// order is preserved and this way the graph never has to know it is being
    /// timed.
    pub fn open(&mut self, device: &wgpu::Device) -> Option<wgpu::CommandBuffer> {
        let queries = self.queries.as_mut()?;
        if queries.stage != Stage::Idle {
            return None;
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_timing_open"),
        });
        encoder.write_timestamp(&queries.set, 0);
        Some(encoder.finish())
    }

    /// A command buffer that closes the span and resolves it, to submit after
    /// the frame's own.
    pub fn close(&mut self, device: &wgpu::Device) -> Option<wgpu::CommandBuffer> {
        let queries = self.queries.as_mut()?;
        if queries.stage != Stage::Idle {
            return None;
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_timing_close"),
        });
        encoder.write_timestamp(&queries.set, 1);
        encoder.resolve_query_set(&queries.set, 0..2, &queries.resolved, 0);
        encoder.copy_buffer_to_buffer(&queries.resolved, 0, &queries.readback, 0, 16);
        queries.stage = Stage::Resolved;
        Some(encoder.finish())
    }

    /// Advances the one outstanding readback: asks for the map once the resolve
    /// has been submitted, and takes the span once the map has landed.
    ///
    /// Called after submitting, never before: a map asked for on a buffer whose
    /// copy is still sitting unsubmitted is ordered ahead of that copy and reads
    /// the frame before last.
    pub fn poll(&mut self) {
        let Some(queries) = &mut self.queries else {
            return;
        };
        match queries.stage {
            Stage::Idle => {}
            Stage::Resolved => {
                let ready = queries.ready.clone();
                queries
                    .readback
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        ready.store(result.is_ok(), std::sync::atomic::Ordering::Release);
                    });
                queries.stage = Stage::Mapping;
            }
            Stage::Mapping => {
                if !queries
                    .ready
                    .swap(false, std::sync::atomic::Ordering::AcqRel)
                {
                    return;
                }
                {
                    let view = queries.readback.slice(..).get_mapped_range();
                    let stamps: &[u64] = bytemuck::cast_slice(&view);
                    let span = stamps[1].saturating_sub(stamps[0]);
                    self.milliseconds = span as f32 * queries.period * 1e-6;
                }
                queries.readback.unmap();
                queries.stage = Stage::Idle;
            }
        }
    }
}
