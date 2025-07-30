use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, SampleRate, Stream, StreamConfig};
use hound::{WavSpec, WavWriter};
use log::{error, info, warn};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AudioManager {
    host: Host,
    input_device: Option<Device>,
    output_device: Option<Device>,
    is_recording: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioManager {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let input_device = host.default_input_device();
        let output_device = host.default_output_device();

        if input_device.is_none() {
            warn!("No default input device found");
        }

        if output_device.is_none() {
            warn!("No default output device found");
        }

        Ok(Self {
            host,
            input_device,
            output_device,
            is_recording: Arc::new(Mutex::new(false)),
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub fn list_input_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
            .unwrap_or_default()
    }

    pub fn list_output_devices(&self) -> Vec<String> {
        self.host
            .output_devices()
            .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
            .unwrap_or_default()
    }

    pub async fn start_recording(&self, duration_secs: f32) -> Result<Vec<u8>, String> {
        let device = self
            .input_device
            .as_ref()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;

        info!("Recording with config: {:?}", config);

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let sample_format = config.sample_format();

        // Clear the buffer
        {
            let mut buffer = self.audio_buffer.lock().unwrap();
            buffer.clear();
        }

        // Set recording flag
        {
            let mut recording = self.is_recording.lock().unwrap();
            *recording = true;
        }

        let buffer_clone = Arc::clone(&self.audio_buffer);
        let recording_clone = Arc::clone(&self.is_recording);

        let stream = match sample_format {
            SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buffer = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buffer.push_back(sample);
                        }
                    },
                    |err| error!("Audio input error: {}", err),
                    None,
                )
                .map_err(|e| format!("Failed to build input stream: {}", e))?,
            SampleFormat::I16 => {
                device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let mut buffer = buffer_clone.lock().unwrap();
                            for &sample in data {
                                // Convert i16 to f32
                                let normalized = sample as f32 / i16::MAX as f32;
                                buffer.push_back(normalized);
                            }
                        },
                        |err| error!("Audio input error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build input stream: {}", e))?
            }
            _ => return Err("Unsupported sample format".to_string()),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start recording: {}", e))?;

        // Record for the specified duration
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(duration_secs)).await;

        // Stop recording
        {
            let mut recording = self.is_recording.lock().unwrap();
            *recording = false;
        }

        drop(stream);

        // Convert buffer to WAV format
        let samples: Vec<f32> = {
            let buffer = self.audio_buffer.lock().unwrap();
            buffer.iter().cloned().collect()
        };

        if samples.is_empty() {
            return Err("No audio data recorded".to_string());
        }

        let wav_data = self.samples_to_wav(&samples, sample_rate, channels)?;

        info!(
            "Recorded {} samples, {} bytes WAV data",
            samples.len(),
            wav_data.len()
        );
        Ok(wav_data)
    }

    fn samples_to_wav(
        &self,
        samples: &[f32],
        sample_rate: u32,
        channels: u16,
    ) -> Result<Vec<u8>, String> {
        let mut wav_data = Vec::new();
        let cursor = Cursor::new(&mut wav_data);

        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::new(cursor, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for &sample in samples {
            // Convert f32 to i16
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

        Ok(wav_data)
    }

    pub async fn play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to create output stream: {}", e))?;

        let sink =
            Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;

        let cursor = Cursor::new(audio_data);
        let decoder = Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))?;

        sink.append(decoder);
        sink.sleep_until_end();

        Ok(())
    }

    pub fn get_input_volume(&self) -> Result<f32, String> {
        // Get current volume level from the buffer
        let buffer = self.audio_buffer.lock().unwrap();
        if buffer.is_empty() {
            return Ok(0.0);
        }

        let sum: f32 = buffer.iter().map(|&x| x.abs()).sum();
        let avg = sum / buffer.len() as f32;
        Ok(avg)
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }
}

pub struct VoiceActivityDetector {
    threshold: f32,
    min_duration_ms: u32,
    silence_duration_ms: u32,
}

impl VoiceActivityDetector {
    pub fn new(threshold: f32, min_duration_ms: u32, silence_duration_ms: u32) -> Self {
        Self {
            threshold,
            min_duration_ms,
            silence_duration_ms,
        }
    }

    pub fn detect_voice_activity(&self, samples: &[f32], sample_rate: u32) -> bool {
        // Calculate RMS (Root Mean Square) energy
        let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();

        // Simple threshold-based detection
        rms > self.threshold
    }

    pub fn find_speech_segments(&self, samples: &[f32], sample_rate: u32) -> Vec<(usize, usize)> {
        let mut segments = Vec::new();
        let window_size = (sample_rate as f32 * 0.025) as usize; // 25ms windows
        let hop_size = window_size / 2;

        let mut in_speech = false;
        let mut speech_start = 0;
        let mut silence_count = 0;

        for i in (0..samples.len().saturating_sub(window_size)).step_by(hop_size) {
            let window = &samples[i..i + window_size];
            let has_voice = self.detect_voice_activity(window, sample_rate);

            if has_voice {
                if !in_speech {
                    speech_start = i;
                    in_speech = true;
                }
                silence_count = 0;
            } else if in_speech {
                silence_count += hop_size;
                let silence_ms = (silence_count as f32 / sample_rate as f32) * 1000.0;

                if silence_ms > self.silence_duration_ms as f32 {
                    let speech_duration_ms =
                        ((i - speech_start) as f32 / sample_rate as f32) * 1000.0;
                    if speech_duration_ms > self.min_duration_ms as f32 {
                        segments.push((speech_start, i));
                    }
                    in_speech = false;
                }
            }
        }

        // Handle case where speech continues to the end
        if in_speech {
            let speech_duration_ms =
                ((samples.len() - speech_start) as f32 / sample_rate as f32) * 1000.0;
            if speech_duration_ms > self.min_duration_ms as f32 {
                segments.push((speech_start, samples.len()));
            }
        }

        segments
    }
}
