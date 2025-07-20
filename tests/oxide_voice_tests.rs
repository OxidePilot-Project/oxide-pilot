use oxide_voice::voice::{WakeWordDetector, GoogleSTTProvider, GoogleTTSProvider};
use async_trait::async_trait;

#[tokio::test]
async fn test_wake_word_detector() {
    let detector = WakeWordDetector::new();
    assert!(detector.start_detection().await.is_ok());
    assert!(detector.stop_detection().await.is_ok());
}

#[tokio::test]
async fn test_google_stt_provider() {
    let stt_provider = GoogleSTTProvider::new();
    let audio_data = vec![1, 2, 3, 4]; // Dummy audio data
    let result = stt_provider.transcribe_audio(audio_data).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Transcribed text from Google STT"));
}

#[tokio::test]
async fn test_google_tts_provider() {
    let tts_provider = GoogleTTSProvider::new();
    let text = "Hello, world!";
    let result = tts_provider.synthesize_speech(text).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}
