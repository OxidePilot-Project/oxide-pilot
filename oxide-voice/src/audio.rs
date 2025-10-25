use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat};
use hound::{WavSpec, WavWriter};
use log::{error, info, warn};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

// Commands for the audio worker thread
#[derive(Debug)]
enum AudioCommand {
    StartRecording {
        duration_secs: f32,
        response: oneshot::Sender<Result<Vec<u8>, String>>,
    },
    PlayAudio {
        data: Vec<u8>,
        response: oneshot::Sender<Result<(), String>>,
    },
    GetInputDevices {
        response: oneshot::Sender<Vec<String>>,
    },
    GetOutputDevices {
        response: oneshot::Sender<Vec<String>>,
    },
    GetInputVolume {
        response: oneshot::Sender<Result<f32, String>>,
    },
}

pub struct AudioManager {
    command_sender: mpsc::UnboundedSender<AudioCommand>,
}

// Internal audio worker that handles the actual audio operations
struct AudioWorker {
    host: Host,
    input_device: Option<Device>,
    _output_device: Option<Device>,
    command_receiver: mpsc::UnboundedReceiver<AudioCommand>,
}

impl AudioManager {
    pub fn new() -> Result<Self, String> {
        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        let host = cpal::default_host();
        let input_device = host.default_input_device();
        let output_device = host.default_output_device();

        if input_device.is_none() {
            warn!("No default input device found");
        }

        if output_device.is_none() {
            warn!("No default output device found");
        }

        let worker = AudioWorker {
            host,
            input_device,
            _output_device: output_device,
            command_receiver,
        };

        // Spawn the worker thread
        std::thread::spawn(move || {
            worker.run();
        });

        Ok(Self { command_sender })
    }

    pub async fn list_input_devices(&self) -> Vec<String> {
        let (response_tx, response_rx) = oneshot::channel();

        if self.command_sender.send(AudioCommand::GetInputDevices { response: response_tx }).is_err() {
            return Vec::new();
        }

        response_rx.await.unwrap_or_default()
    }

    pub async fn list_output_devices(&self) -> Vec<String> {
        let (response_tx, response_rx) = oneshot::channel();

        if self.command_sender.send(AudioCommand::GetOutputDevices { response: response_tx }).is_err() {
            return Vec::new();
        }

        response_rx.await.unwrap_or_default()
    }

    pub async fn start_recording(&self, duration_secs: f32) -> Result<Vec<u8>, String> {
        let (response_tx, response_rx) = oneshot::channel();

        if self.command_sender.send(AudioCommand::StartRecording {
            duration_secs,
            response: response_tx
        }).is_err() {
            return Err("Audio worker is not available".to_string());
        }

        response_rx.await.map_err(|_| "Audio worker response failed".to_string())?

    }

    #[allow(dead_code)]
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
            .map_err(|e| format!("Failed to create WAV writer: {e}"))?;

        for &sample in samples {
            // Convert f32 to i16
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {e}"))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {e}"))?;

        Ok(wav_data)
    }

    pub async fn play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        let (response_tx, response_rx) = oneshot::channel();

        if self.command_sender.send(AudioCommand::PlayAudio {
            data: audio_data.to_vec(),
            response: response_tx
        }).is_err() {
            return Err("Audio worker is not available".to_string());
        }

        response_rx.await.map_err(|_| "Audio worker response failed".to_string())?
    }

    pub async fn get_input_volume(&self) -> Result<f32, String> {
        let (response_tx, response_rx) = oneshot::channel();

        if self.command_sender.send(AudioCommand::GetInputVolume {
            response: response_tx
        }).is_err() {
            return Err("Audio worker is not available".to_string());
        }

        response_rx.await.map_err(|_| "Audio worker response failed".to_string())?
    }
}

impl AudioWorker {
    fn run(mut self) {
        while let Some(command) = self.command_receiver.blocking_recv() {
            match command {
                AudioCommand::StartRecording { duration_secs, response } => {
                    let result = self.handle_start_recording(duration_secs);
                    let _ = response.send(result);
                }
                AudioCommand::PlayAudio { data, response } => {
                    let result = self.handle_play_audio(&data);
                    let _ = response.send(result);
                }
                AudioCommand::GetInputDevices { response } => {
                    let devices = self.handle_get_input_devices();
                    let _ = response.send(devices);
                }
                AudioCommand::GetOutputDevices { response } => {
                    let devices = self.handle_get_output_devices();
                    let _ = response.send(devices);
                }
                AudioCommand::GetInputVolume { response } => {
                    let result = self.handle_get_input_volume();
                    let _ = response.send(result);
                }
            }
        }
    }

    fn handle_get_input_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
            .unwrap_or_default()
    }

    fn handle_get_output_devices(&self) -> Vec<String> {
        self.host
            .output_devices()
            .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
            .unwrap_or_default()
    }

    fn handle_get_input_volume(&self) -> Result<f32, String> {
        // For now, return a placeholder value
        // In a real implementation, this would check the current input level
        Ok(0.5)
    }

    fn handle_play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to create output stream: {e}"))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create sink: {e}"))?;

        // Clone the audio data to avoid lifetime issues
        let audio_data_owned = audio_data.to_vec();
        let cursor = Cursor::new(audio_data_owned);
        let decoder = Decoder::new(cursor)
            .map_err(|e| format!("Failed to decode audio: {e}"))?;

        sink.append(decoder);
        sink.sleep_until_end();

        Ok(())
    }

    fn handle_start_recording(&self, duration_secs: f32) -> Result<Vec<u8>, String> {
        let device = self
            .input_device
            .as_ref()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get input config: {e}"))?;

        info!("Recording with config: {config:?}");

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let sample_format = config.sample_format();

        let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));
        let is_recording = Arc::new(Mutex::new(true));

        let buffer_clone = Arc::clone(&audio_buffer);
        let recording_clone = Arc::clone(&is_recording);

        let stream = match sample_format {
            SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let recording = recording_clone.lock().unwrap();
                        if *recording {
                            let mut buffer = buffer_clone.lock().unwrap();
                            for &sample in data {
                                buffer.push_back(sample);
                            }
                        }
                    },
                    |err| error!("Audio input error: {err}"),
                    None,
                )
                .map_err(|e| format!("Failed to build input stream: {e}"))?,
            SampleFormat::I16 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let recording = recording_clone.lock().unwrap();
                        if *recording {
                            let mut buffer = buffer_clone.lock().unwrap();
                            for &sample in data {
                                let normalized = sample as f32 / i16::MAX as f32;
                                buffer.push_back(normalized);
                            }
                        }
                    },
                    |err| error!("Audio input error: {err}"),
                    None,
                )
                .map_err(|e| format!("Failed to build input stream: {e}"))?,
            _ => return Err("Unsupported sample format".to_string()),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start recording: {e}"))?;

        // Record for the specified duration
        std::thread::sleep(std::time::Duration::from_secs_f32(duration_secs));

        // Stop recording
        {
            let mut recording = is_recording.lock().unwrap();
            *recording = false;
        }

        drop(stream);

        // Convert buffer to WAV format
        let samples: Vec<f32> = {
            let buffer = audio_buffer.lock().unwrap();
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
            .map_err(|e| format!("Failed to create WAV writer: {e}"))?;

        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {e}"))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {e}"))?;

        Ok(wav_data)
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

    pub fn detect_voice_activity(&self, samples: &[f32], _sample_rate: u32) -> bool {
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
