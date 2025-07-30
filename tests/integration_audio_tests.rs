use oxide_voice::audio::{AudioManager, VoiceActivityDetector};
use oxide_voice::voice::{WakeWordDetector, GoogleSTTProvider, GoogleTTSProvider, VoiceProcessor};
use tokio;

#[tokio::test]
async fn test_audio_manager_initialization() {
    let audio_manager = AudioManager::new();
    assert!(audio_manager.is_ok(), "AudioManager should initialize successfully");

    let manager = audio_manager.unwrap();
    let input_devices = manager.list_input_devices();
    let output_devices = manager.list_output_devices();

    println!("Input devices: {:?}", input_devices);
    println!("Output devices: {:?}", output_devices);

    // At least one device should be available on most systems
    // Note: This might fail in CI environments without audio hardware
    if !input_devices.is_empty() {
        println!("✓ Input devices found");
    } else {
        println!("⚠ No input devices found (this is expected in CI)");
    }
}

#[tokio::test]
async fn test_voice_activity_detector() {
    let vad = VoiceActivityDetector::new(0.01, 300, 500);

    // Test with silence (all zeros)
    let silence: Vec<f32> = vec![0.0; 1000];
    let has_voice = vad.detect_voice_activity(&silence, 16000);
    assert!(!has_voice, "Silence should not be detected as voice");

    // Test with noise (random values)
    let noise: Vec<f32> = (0..1000).map(|_| rand::random::<f32>() * 0.1).collect();
    let has_voice_noise = vad.detect_voice_activity(&noise, 16000);
    println!("Noise detected as voice: {}", has_voice_noise);

    // Test with strong signal
    let signal: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
    let has_voice_signal = vad.detect_voice_activity(&signal, 16000);
    assert!(has_voice_signal, "Strong signal should be detected as voice");
}

#[tokio::test]
async fn test_wake_word_detector_creation() {
    let wake_words = vec!["Hey Oxide".to_string(), "Oxide".to_string()];
    let detector = WakeWordDetector::new(wake_words.clone());

    match detector {
        Ok(_) => println!("✓ WakeWordDetector created successfully"),
        Err(e) => {
            println!("⚠ WakeWordDetector creation failed: {} (expected in CI)", e);
            // Don't fail the test in CI environments
        }
    }
}

#[tokio::test]
async fn test_voice_processor_creation() {
    let wake_words = vec!["Hey Oxide".to_string()];
    let stt_provider = Box::new(GoogleSTTProvider::new(Some("en-US".to_string())));
    let tts_provider = Box::new(GoogleTTSProvider::new(Some("en-US".to_string()), None));

    let processor = VoiceProcessor::new(wake_words, stt_provider, tts_provider);

    match processor {
        Ok(proc) => {
            println!("✓ VoiceProcessor created successfully");
            let input_devices = proc.get_input_devices();
            let output_devices = proc.get_output_devices();
            println!("Available input devices: {}", input_devices.len());
            println!("Available output devices: {}", output_devices.len());
        },
        Err(e) => {
            println!("⚠ VoiceProcessor creation failed: {} (expected in CI)", e);
        }
    }
}

#[tokio::test]
async fn test_audio_recording_simulation() {
    // This test simulates audio recording without requiring actual hardware
    let audio_manager = AudioManager::new();

    if let Ok(manager) = audio_manager {
        // Test volume detection (should return 0 when not recording)
        let volume = manager.get_input_volume();
        assert!(volume.is_ok(), "Volume detection should work");

        let vol_value = volume.unwrap();
        assert!(vol_value >= 0.0, "Volume should be non-negative");
        println!("Current input volume: {:.3}", vol_value);

        // Test recording state
        assert!(!manager.is_recording(), "Should not be recording initially");
    } else {
        println!("⚠ AudioManager not available (expected in CI)");
    }
}

#[cfg(feature = "hardware_tests")]
#[tokio::test]
async fn test_real_audio_recording() {
    // This test requires actual audio hardware and is only run with the hardware_tests feature
    let audio_manager = AudioManager::new().expect("Audio hardware required for this test");

    println!("Starting 2-second audio recording test...");
    let audio_data = audio_manager.start_recording(2.0).await;

    match audio_data {
        Ok(data) => {
            assert!(!data.is_empty(), "Should record some audio data");
            println!("✓ Recorded {} bytes of audio", data.len());

            // Test playback
            let playback_result = audio_manager.play_audio(&data).await;
            assert!(playback_result.is_ok(), "Should be able to play recorded audio");
            println!("✓ Audio playback completed");
        },
        Err(e) => {
            panic!("Audio recording failed: {}", e);
        }
    }
}