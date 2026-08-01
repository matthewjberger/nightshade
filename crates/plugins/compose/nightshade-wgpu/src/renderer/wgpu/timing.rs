use crate::renderer::wgpu::build;
#[derive(Default)]
pub struct GpuTiming {
    queries: Option<Queries>,
    pub milliseconds: f32,
}

struct Queries {
    set: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readback: wgpu::Buffer,
    period: f32,
    stage: Stage,
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Idle,
    Resolved,
    Mapping,
}

impl Drop for Queries {
    fn drop(&mut self) {
        if self.stage == Stage::Mapping {
            self.readback.unmap();
        }
    }
}

impl GpuTiming {
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
                resolved: build::buffer(
                    device,
                    "frame_timing_resolved",
                    16,
                    wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                ),
                readback: build::buffer(
                    device,
                    "frame_timing_readback",
                    16,
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                ),
                period: queue.get_timestamp_period(),
                stage: Stage::Idle,
                ready: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }),
            milliseconds: 0.0,
        }
    }

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
