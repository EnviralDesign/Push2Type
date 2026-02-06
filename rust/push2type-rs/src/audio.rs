use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use anyhow::{Context, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::app::AppEvent;

pub struct AudioRecorder {
    sample_rate: u32,
    capturing: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<i16>>>,
    _stream: cpal::Stream,
}

impl AudioRecorder {
    pub fn new(events: crossbeam_channel::Sender<AppEvent>) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no default input device available")?;
        let supported = device.default_input_config()?;
        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels() as usize;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();

        let capturing = Arc::new(AtomicBool::new(false));
        let buffer = Arc::new(Mutex::new(Vec::<i16>::new()));
        let capturing_clone = capturing.clone();
        let buffer_clone = buffer.clone();
        let err_events = events.clone();

        let stream = match sample_format {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    if capturing_clone.load(Ordering::Relaxed) {
                        let mono = downmix_i16_to_mono(data, channels);
                        if let Ok(mut buf) = buffer_clone.lock() {
                            buf.extend_from_slice(&mono);
                        }
                    }
                },
                move |err| {
                    let _ = err_events.send(AppEvent::Error(format!("audio error: {err}")));
                },
                None,
            )?,
            cpal::SampleFormat::U16 => {
                let capturing_clone = capturing.clone();
                let buffer_clone = buffer.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[u16], _| {
                        let converted: Vec<i16> = data
                            .iter()
                            .map(|s| {
                                (*s as i32 - 32768).clamp(i16::MIN as i32, i16::MAX as i32) as i16
                            })
                            .collect();
                        if capturing_clone.load(Ordering::Relaxed) {
                            let mono = downmix_i16_to_mono(&converted, channels);
                            if let Ok(mut buf) = buffer_clone.lock() {
                                buf.extend_from_slice(&mono);
                            }
                        }
                    },
                    move |err| {
                        let _ = events.send(AppEvent::Error(format!("audio error: {err}")));
                    },
                    None,
                )?
            }
            cpal::SampleFormat::F32 => {
                let capturing_clone = capturing.clone();
                let buffer_clone = buffer.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _| {
                        let converted: Vec<i16> = data
                            .iter()
                            .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16)
                            .collect();
                        if capturing_clone.load(Ordering::Relaxed) {
                            let mono = downmix_i16_to_mono(&converted, channels);
                            if let Ok(mut buf) = buffer_clone.lock() {
                                buf.extend_from_slice(&mono);
                            }
                        }
                    },
                    move |err| {
                        let _ = events.send(AppEvent::Error(format!("audio error: {err}")));
                    },
                    None,
                )?
            }
            _ => return Err(anyhow!("unsupported sample format")),
        };
        stream.play()?;

        Ok(Self {
            sample_rate,
            capturing,
            buffer,
            _stream: stream,
        })
    }

    pub fn start_capture(&self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
        self.capturing.store(true, Ordering::Relaxed);
    }

    pub fn stop_capture(&self) -> Vec<i16> {
        self.capturing.store(false, Ordering::Relaxed);
        self.buffer.lock().map(|b| b.clone()).unwrap_or_default()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

fn downmix_i16_to_mono(data: &[i16], channels: usize) -> Vec<i16> {
    if channels <= 1 {
        return data.to_vec();
    }
    let mut out = Vec::with_capacity(data.len() / channels);
    for frame in data.chunks_exact(channels) {
        let sum: i32 = frame.iter().map(|s| *s as i32).sum();
        out.push((sum / channels as i32) as i16);
    }
    out
}
