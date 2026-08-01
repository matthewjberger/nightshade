use crate::data::stream::StreamHandle;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct Capture {
    stream: cpal::Stream,
    rate: u32,
    channels: u32,
}

impl Capture {
    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn start(&self) -> bool {
        self.stream.play().is_ok()
    }

    pub fn stop(&self) -> bool {
        self.stream.pause().is_ok()
    }
}

pub fn capture_devices() -> Vec<String> {
    let host = cpal::default_host();
    let Ok(devices) = host.input_devices() else {
        return Vec::new();
    };
    devices.map(|device| device.to_string()).collect()
}

pub fn open_capture(name: &str, into: StreamHandle, gate: f32) -> Option<Capture> {
    let host = cpal::default_host();
    let device = if name.is_empty() {
        host.default_input_device()?
    } else {
        host.input_devices()
            .ok()?
            .find(|device| device.to_string() == name)?
    };

    let config = device.default_input_config().ok()?;
    let rate = config.sample_rate();
    let channels = config.channels() as u32;
    let format = config.sample_format();
    let shape: cpal::StreamConfig = config.into();

    let stream = match format {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                shape,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if loudest(data) >= gate {
                        into.push_interleaved(data, channels);
                    }
                },
                |error| tracing::error!("Audio capture failed: {error}"),
                None,
            )
            .ok()?,
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                shape,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let widened: Vec<f32> = data
                        .iter()
                        .map(|sample| *sample as f32 / i16::MAX as f32)
                        .collect();
                    if loudest(&widened) >= gate {
                        into.push_interleaved(&widened, channels);
                    }
                },
                |error| tracing::error!("Audio capture failed: {error}"),
                None,
            )
            .ok()?,
        cpal::SampleFormat::U16 => device
            .build_input_stream(
                shape,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let widened: Vec<f32> = data
                        .iter()
                        .map(|sample| *sample as f32 / u16::MAX as f32 * 2.0 - 1.0)
                        .collect();
                    if loudest(&widened) >= gate {
                        into.push_interleaved(&widened, channels);
                    }
                },
                |error| tracing::error!("Audio capture failed: {error}"),
                None,
            )
            .ok()?,
        _ => return None,
    };

    Some(Capture {
        stream,
        rate,
        channels,
    })
}

fn loudest(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0f32, |held, sample| held.max(sample.abs()))
}
