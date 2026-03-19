use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

pub struct Recorder {
    stream: Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    audio_level: Arc<Mutex<AudioLevel>>,
}

pub struct AudioLevel {
    current_rms: f32,
    /// Tracks the quiet baseline (adapts fast at start, then slowly)
    quiet_level: f32,
    /// Tracks the loud peak (adapts dynamically)
    loud_level: f32,
    history: Vec<f32>,
    sample_count: u32,
    total_callbacks: u32,
}

impl AudioLevel {
    fn new() -> Self {
        Self {
            current_rms: 0.0,
            quiet_level: f32::MAX, // will be set on first samples
            loud_level: 0.0,
            history: vec![0.0; 30],
            sample_count: 0,
            total_callbacks: 0,
        }
    }

    fn update(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();

        self.total_callbacks += 1;

        // Smooth RMS
        if rms > self.current_rms {
            self.current_rms = self.current_rms * 0.3 + rms * 0.7;
        } else {
            self.current_rms = self.current_rms * 0.92 + rms * 0.08;
        }

        // During first ~50 callbacks (~1 sec), quickly learn the quiet baseline
        if self.total_callbacks < 50 {
            if rms < self.quiet_level {
                self.quiet_level = rms;
            }
            self.loud_level = self.quiet_level * 5.0;
        } else {
            // Quiet level: slowly track the minimum (noise floor)
            if rms < self.quiet_level * 1.5 {
                self.quiet_level = self.quiet_level * 0.998 + rms * 0.002;
            }
            // Loud level: track peaks, decay slowly
            if rms > self.loud_level {
                self.loud_level = self.loud_level * 0.5 + rms * 0.5;
            } else {
                self.loud_level = self.loud_level * 0.999 + self.quiet_level * 0.001;
                // Don't let loud_level drop below 3x quiet
                self.loud_level = self.loud_level.max(self.quiet_level * 3.0);
            }
        }

        self.sample_count += 1;
        if self.sample_count >= 8 {
            self.sample_count = 0;

            let range = self.loud_level - self.quiet_level;
            let normalized = if range > 0.0001 {
                ((self.current_rms - self.quiet_level) / range).clamp(0.0, 1.0)
            } else {
                0.0
            };

            self.history.push(normalized);
            if self.history.len() > 30 {
                self.history.remove(0);
            }
        }
    }
}

impl Recorder {
    pub fn start() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("No input config: {e}"))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let audio_level: Arc<Mutex<AudioLevel>> = Arc::new(Mutex::new(AudioLevel::new()));
        let buf_clone = Arc::clone(&buffer);
        let level_clone = Arc::clone(&audio_level);

        let err_fn = |e: cpal::StreamError| {
            eprintln!("Audio stream error: {e}");
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mono = to_mono(data, channels);
                        if let Ok(mut level) = level_clone.lock() {
                            level.update(&mono);
                        }
                        if let Ok(mut buf) = buf_clone.lock() {
                            buf.extend_from_slice(&mono);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("Build stream: {e}"))?,
            cpal::SampleFormat::I16 => {
                let buf_clone2 = Arc::clone(&buffer);
                let level_clone2 = Arc::clone(&audio_level);
                device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let floats: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            let mono = to_mono(&floats, channels);
                            if let Ok(mut level) = level_clone2.lock() {
                                level.update(&mono);
                            }
                            if let Ok(mut buf) = buf_clone2.lock() {
                                buf.extend_from_slice(&mono);
                            }
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Build stream: {e}"))?
            }
            fmt => return Err(format!("Unsupported sample format: {fmt:?}")),
        };

        stream.play().map_err(|e| format!("Play stream: {e}"))?;

        Ok(Recorder {
            stream,
            buffer,
            sample_rate,
            audio_level,
        })
    }

    /// Get waveform history (Vec of 0.0 - 1.0 values, ~30 entries)
    pub fn get_levels(&self) -> (f32, Vec<f32>) {
        if let Ok(level) = self.audio_level.lock() {
            let current = level.history.last().copied().unwrap_or(0.0);
            (current, level.history.clone())
        } else {
            (0.0, vec![])
        }
    }

    /// Stops recording and returns (samples_16khz_mono, duration_secs).
    pub fn stop(self) -> (Vec<f32>, f32) {
        drop(self.stream);

        let raw = self.buffer.lock().unwrap().clone();
        let duration = raw.len() as f32 / self.sample_rate as f32;
        let resampled = resample_to_16khz(&raw, self.sample_rate);

        (resampled, duration)
    }
}

fn to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }
    samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Simple linear interpolation resample to 16kHz.
pub fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Vec<f32> {
    if source_rate == 16000 {
        return samples.to_vec();
    }
    let ratio = source_rate as f64 / 16000.0;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let src_idx = i as f64 * ratio;
            let idx = src_idx as usize;
            let frac = (src_idx - idx as f64) as f32;
            let a = samples[idx.min(samples.len() - 1)];
            let b = samples[(idx + 1).min(samples.len() - 1)];
            a + (b - a) * frac
        })
        .collect()
}
