use crate::audio::{AudioManager, VoiceActivityDetector};
use async_trait::async_trait;
use log::{error, info, warn};
use oxide_core::google_auth::get_access_token;
use reqwest::Client;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct WakeWordDetector {
    is_listening: Arc<Mutex<bool>>,
    wake_words: Vec<String>,
    sensitivity: f32,
    audio_manager: Arc<AudioManager>,
    vad: VoiceActivityDetector,
}

impl WakeWordDetector {
    pub fn new(wake_words: Vec<String>) -> Result<Self, String> {
        let audio_manager = Arc::new(AudioManager::new()?);
        let vad = VoiceActivityDetector::new(0.01, 300, 500); // threshold, min_duration_ms, silence_duration_ms

        Ok(Self {
            is_listening: Arc::new(Mutex::new(false)),
            wake_words,
            sensitivity: 0.5,
            audio_manager,
            vad,
        })
    }

    pub async fn start_detection(&self) -> Result<mpsc::Receiver<String>, String> {
        info!(
            "Starting wake word detection for words: {:?}",
            self.wake_words
        );

        let (tx, rx) = mpsc::channel(100);
        let is_listening = Arc::clone(&self.is_listening);
        let wake_words = self.wake_words.clone();

        // Set listening state
        {
            let mut listening = is_listening.lock().unwrap();
            *listening = true;
        }

        // Spawn background thread for wake word detection
        tokio::spawn(async move {
            // Simulate wake word detection (in real implementation, use Picovoice Porcupine)
            let mut counter = 0;
            loop {
                {
                    let listening = is_listening.lock().unwrap();
                    if !*listening {
                        break;
                    }
                }

                // Simulate periodic wake word detection
                tokio::time::sleep(Duration::from_secs(5)).await;
                counter += 1;

                // Simulate wake word detection every 30 seconds for demo
                if counter % 6 == 0 {
                    if let Some(wake_word) = wake_words.first() {
                        info!("Wake word detected: {}", wake_word);
                        if tx.send(wake_word.clone()).await.is_err() {
                            warn!("Failed to send wake word detection");
                            break;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    pub async fn stop_detection(&self) -> Result<(), String> {
        info!("Stopping wake word detection...");
        let mut listening = self.is_listening.lock().unwrap();
        *listening = false;
        Ok(())
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.0, 1.0);
        info!("Wake word sensitivity set to: {}", self.sensitivity);
    }
}

#[async_trait]
pub trait STTProvider {
    async fn transcribe_audio(&self, audio_data: Vec<u8>) -> Result<String, String>;
}

pub struct GoogleSTTProvider {
    http_client: Client,
    language_code: String,
}

impl GoogleSTTProvider {
    pub fn new(language_code: Option<String>) -> Self {
        Self {
            http_client: Client::new(),
            language_code: language_code.unwrap_or_else(|| "en-US".to_string()),
        }
    }
}

#[async_trait]
impl STTProvider for GoogleSTTProvider {
    async fn transcribe_audio(&self, audio_data: Vec<u8>) -> Result<String, String> {
        info!("Transcribing audio with Google STT...");

        let access_token = get_access_token()
            .await
            .map_err(|e| format!("Failed to get access token: {}", e))?
            .ok_or("No access token available")?;

        let request_body = json!({
            "config": {
                "encoding": "WEBM_OPUS",
                "sampleRateHertz": 16000,
                "languageCode": self.language_code,
                "enableAutomaticPunctuation": true
            },
            "audio": {
                "content": base64::encode(&audio_data)
            }
        });

        let response = self
            .http_client
            .post("https://speech.googleapis.com/v1/speech:recognize")
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Google STT API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(results) = response_json["results"].as_array() {
            if let Some(first_result) = results.first() {
                if let Some(alternatives) = first_result["alternatives"].as_array() {
                    if let Some(first_alternative) = alternatives.first() {
                        if let Some(transcript) = first_alternative["transcript"].as_str() {
                            info!("Transcription successful: {}", transcript);
                            return Ok(transcript.to_string());
                        }
                    }
                }
            }
        }

        warn!("No transcription found in response");
        Ok("".to_string())
    }
}

#[async_trait]
pub trait TTSProvider {
    async fn synthesize_speech(&self, text: &str) -> Result<Vec<u8>, String>;
}

pub struct GoogleTTSProvider {
    http_client: Client,
    language_code: String,
    voice_name: String,
    speaking_rate: f32,
}

impl GoogleTTSProvider {
    pub fn new(language_code: Option<String>, voice_name: Option<String>) -> Self {
        Self {
            http_client: Client::new(),
            language_code: language_code.unwrap_or_else(|| "en-US".to_string()),
            voice_name: voice_name.unwrap_or_else(|| "en-US-Wavenet-D".to_string()),
            speaking_rate: 1.0,
        }
    }

    pub fn set_speaking_rate(&mut self, rate: f32) {
        self.speaking_rate = rate.clamp(0.25, 4.0);
        info!("TTS speaking rate set to: {}", self.speaking_rate);
    }
}

#[async_trait]
impl TTSProvider for GoogleTTSProvider {
    async fn synthesize_speech(&self, text: &str) -> Result<Vec<u8>, String> {
        info!("Synthesizing speech with Google TTS: {}", text);

        let access_token = get_access_token()
            .await
            .map_err(|e| format!("Failed to get access token: {}", e))?
            .ok_or("No access token available")?;

        let request_body = json!({
            "input": {
                "text": text
            },
            "voice": {
                "languageCode": self.language_code,
                "name": self.voice_name
            },
            "audioConfig": {
                "audioEncoding": "MP3",
                "speakingRate": self.speaking_rate
            }
        });

        let response = self
            .http_client
            .post("https://texttospeech.googleapis.com/v1/text:synthesize")
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Google TTS API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(audio_content) = response_json["audioContent"].as_str() {
            let audio_data = base64::decode(audio_content)
                .map_err(|e| format!("Failed to decode audio: {}", e))?;

            info!("Speech synthesis successful, {} bytes", audio_data.len());
            Ok(audio_data)
        } else {
            Err("No audio content in response".to_string())
        }
    }
}

// Voice processor that combines all voice functionality
pub struct VoiceProcessor {
    wake_word_detector: WakeWordDetector,
    stt_provider: Box<dyn STTProvider + Send + Sync>,
    tts_provider: Box<dyn TTSProvider + Send + Sync>,
}

impl VoiceProcessor {
    pub fn new(
        wake_words: Vec<String>,
        stt_provider: Box<dyn STTProvider + Send + Sync>,
        tts_provider: Box<dyn TTSProvider + Send + Sync>,
    ) -> Result<Self, String> {
        Ok(Self {
            wake_word_detector: WakeWordDetector::new(wake_words)?,
            stt_provider,
            tts_provider,
        })
    }

    pub async fn start_listening(&self) -> Result<mpsc::Receiver<String>, String> {
        self.wake_word_detector.start_detection().await
    }

    pub async fn stop_listening(&self) -> Result<(), String> {
        self.wake_word_detector.stop_detection().await
    }

    pub async fn transcribe_audio(&self, audio_data: Vec<u8>) -> Result<String, String> {
        self.stt_provider.transcribe_audio(audio_data).await
    }

    pub async fn synthesize_speech(&self, text: &str) -> Result<Vec<u8>, String> {
        self.tts_provider.synthesize_speech(text).await
    }

    pub async fn record_audio(&self, duration_secs: f32) -> Result<Vec<u8>, String> {
        self.wake_word_detector
            .audio_manager
            .start_recording(duration_secs)
            .await
    }

    pub async fn play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        self.wake_word_detector
            .audio_manager
            .play_audio(audio_data)
            .await
    }

    pub fn get_input_devices(&self) -> Vec<String> {
        self.wake_word_detector.audio_manager.list_input_devices()
    }

    pub fn get_output_devices(&self) -> Vec<String> {
        self.wake_word_detector.audio_manager.list_output_devices()
    }

    pub fn get_input_volume(&self) -> Result<f32, String> {
        self.wake_word_detector.audio_manager.get_input_volume()
    }
}
