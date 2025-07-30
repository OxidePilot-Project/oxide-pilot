use chrono::Utc;
use log::{error, info, warn};
use oxide_copilot::ai::AIOrchestrator;
use oxide_copilot::copilot::CopilotAgent;
use oxide_copilot::functions::FunctionRegistry;
use oxide_core::config::{
    AIProvidersConfig, CopilotConfig, GoogleConfig, GuardianConfig, OxidePilotConfig,
};
use oxide_core::performance::{PerformanceMonitor, PerformanceTimer, ResourceOptimizer};
use oxide_core::types::{Context, Interaction};
use oxide_guardian::guardian::{Guardian, SystemStatus, ThreatEvent};
use oxide_memory::memory::{ContextQuery, MemoryEntryType, MemoryManager, MemoryStats};
use oxide_voice::voice::{GoogleSTTProvider, GoogleTTSProvider, VoiceProcessor};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct OxideSystem {
    config: Arc<Mutex<OxidePilotConfig>>,
    guardian: Arc<Guardian>,
    copilot: Arc<CopilotAgent>,
    memory_manager: Arc<MemoryManager>,
    voice_processor: Arc<VoiceProcessor>,
    performance_monitor: Arc<PerformanceMonitor>,
    resource_optimizer: Arc<Mutex<ResourceOptimizer>>,
    is_running: Arc<Mutex<bool>>,
}

impl OxideSystem {
    pub async fn new(config: OxidePilotConfig) -> Result<Self, String> {
        info!("Initializing Oxide Pilot System...");

        // Validate configuration
        config
            .validate()
            .map_err(|e| format!("Configuration validation failed: {}", e))?;

        // Initialize memory manager
        let memory_manager = Arc::new(MemoryManager::new(Some("oxide_data".to_string())));
        memory_manager.initialize().await?;

        // Initialize Guardian Agent
        let guardian = Arc::new(Guardian::new(config.guardian.clone()));

        // Initialize AI Orchestrator
        let ai_orchestrator = Arc::new(AIOrchestrator::new(config.ai_providers.clone()));

        // Initialize Function Registry
        let function_registry = Arc::new(FunctionRegistry::new());

        // Initialize Copilot Agent
        let copilot = Arc::new(CopilotAgent::new(
            config.copilot.clone(),
            ai_orchestrator,
            function_registry,
        ));

        // Initialize Voice Processor
        let wake_words = vec![config.copilot.wake_word.clone()];
        let stt_provider = Box::new(GoogleSTTProvider::new(Some("en-US".to_string())));
        let tts_provider = Box::new(GoogleTTSProvider::new(Some("en-US".to_string()), None));
        let voice_processor =
            Arc::new(VoiceProcessor::new(wake_words, stt_provider, tts_provider)?);

        info!(
            "Audio devices - Input: {:?}, Output: {:?}",
            voice_processor.get_input_devices(),
            voice_processor.get_output_devices()
        );

        // Initialize Performance Monitor
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let resource_optimizer = Arc::new(Mutex::new(ResourceOptimizer::new(Arc::clone(
            &performance_monitor,
        ))));

        let system = Self {
            config: Arc::new(Mutex::new(config)),
            guardian,
            copilot,
            memory_manager,
            voice_processor,
            performance_monitor,
            resource_optimizer,
            is_running: Arc::new(Mutex::new(false)),
        };

        info!("Oxide Pilot System initialized successfully");
        Ok(system)
    }

    pub async fn start(&self) -> Result<(), String> {
        info!("Starting Oxide Pilot System...");

        {
            let mut running = self.is_running.lock().unwrap();
            if *running {
                return Err("System is already running".to_string());
            }
            *running = true;
        }

        // Start Guardian monitoring
        self.guardian.start_monitoring();
        info!("Guardian Agent started");

        // Start voice processing
        let voice_receiver = self.voice_processor.start_listening().await?;
        info!("Voice processing started");

        // Start main system loop
        self.start_main_loop(voice_receiver).await;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        info!("Stopping Oxide Pilot System...");

        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }

        // Stop voice processing
        self.voice_processor.stop_listening().await?;

        info!("Oxide Pilot System stopped");
        Ok(())
    }

    async fn start_main_loop(&self, mut voice_receiver: mpsc::Receiver<String>) {
        let is_running = Arc::clone(&self.is_running);
        let copilot = Arc::clone(&self.copilot);
        let memory_manager = Arc::clone(&self.memory_manager);
        let voice_processor = Arc::clone(&self.voice_processor);

        tokio::spawn(async move {
            info!("Main system loop started");

            while {
                let running = is_running.lock().unwrap();
                *running
            } {
                tokio::select! {
                    // Handle wake word detection
                    wake_word = voice_receiver.recv() => {
                        if let Some(word) = wake_word {
                            info!("Wake word detected: {}", word);

                            // Record real audio for transcription
                            info!("Recording audio for transcription...");
                            match voice_processor.record_audio(3.0).await {
                                Ok(audio_data) => {
                                    info!("Recorded {} bytes of audio", audio_data.len());
                                    match voice_processor.transcribe_audio(audio_data).await {
                                Ok(transcription) => {
                                    if !transcription.is_empty() {
                                        info!("User said: {}", transcription);

                                        // Process user input with Copilot
                                        let context = Context {
                                            system_state: Default::default(),
                                            user_history: Vec::new(),
                                            relevant_events: Vec::new(),
                                            knowledge_entries: Vec::new(),
                                        };

                                        match copilot.handle_user_input(transcription.clone(), context.clone()).await {
                                            Ok(response) => {
                                                info!("Copilot response: {}", response);

                                                // Store interaction in memory
                                                let interaction = Interaction {
                                                    id: uuid::Uuid::new_v4(),
                                                    timestamp: Utc::now(),
                                                    user_input: transcription,
                                                    agent_response: response.clone(),
                                                    context,
                                                };

                                                if let Err(e) = memory_manager.store_interaction(interaction).await {
                                                    error!("Failed to store interaction: {}", e);
                                                }

                                                // Synthesize and play speech response
                                                match voice_processor.synthesize_speech(&response).await {
                                                    Ok(audio_data) => {
                                                        info!("Speech synthesized, {} bytes", audio_data.len());
                                                        if let Err(e) = voice_processor.play_audio(&audio_data).await {
                                                            error!("Failed to play audio: {}", e);
                                                        } else {
                                                            info!("Audio played successfully");
                                                        }
                                                    },
                                                    Err(e) => error!("Failed to synthesize speech: {}", e),
                                                }
                                            },
                                            Err(e) => error!("Copilot error: {}", e),
                                        }
                                    }
                                    },
                                    Err(e) => error!("Transcription failed: {}", e),
                                }
                                },
                                Err(e) => error!("Audio recording failed: {}", e),
                            }
                        }
                    }

                    // Periodic system maintenance
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {
                        // Perform periodic maintenance tasks
                        Self::perform_maintenance(&memory_manager).await;
                    }
                }
            }

            info!("Main system loop ended");
        });
    }

    async fn perform_maintenance(memory_manager: &Arc<MemoryManager>) {
        // Log system statistics
        let stats = memory_manager.get_memory_stats();
        info!(
            "Memory stats - Entries: {}, Patterns: {}",
            stats.total_entries, stats.total_patterns
        );

        // Additional maintenance tasks could include:
        // - Cleaning up old memory entries
        // - Optimizing memory storage
        // - Checking system health
        // - Updating threat signatures
    }

    pub async fn handle_text_input(&self, input: String) -> Result<String, String> {
        let _timer = PerformanceTimer::new(
            "handle_text_input".to_string(),
            Arc::clone(&self.performance_monitor),
        );
        info!("Handling text input: {}", input);

        // Build context from memory
        let context_query = ContextQuery {
            query: input.clone(),
            context_type: None,
            time_range: None,
            max_results: 10,
            min_relevance: 0.3,
        };

        let relevant_memories = self.memory_manager.retrieve_context(&context_query).await?;

        let context = Context {
            system_state: Default::default(),
            user_history: Vec::new(),
            relevant_events: Vec::new(),
            knowledge_entries: relevant_memories.into_iter().map(|m| m.content).collect(),
        };

        // Process with Copilot
        let response = self
            .copilot
            .handle_user_input(input.clone(), context.clone())
            .await?;

        // Store interaction
        let interaction = Interaction {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            user_input: input,
            agent_response: response.clone(),
            context,
        };

        self.memory_manager.store_interaction(interaction).await?;

        Ok(response)
    }

    pub fn get_system_status(&self) -> SystemStatus {
        self.guardian.get_system_status()
    }

    pub fn get_threat_history(&self) -> Vec<ThreatEvent> {
        self.guardian.get_threat_history()
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        self.memory_manager.get_memory_stats()
    }

    pub async fn update_config(&self, new_config: OxidePilotConfig) -> Result<(), String> {
        new_config.validate()?;

        let mut config = self.config.lock().unwrap();
        *config = new_config.clone();

        // Update individual components
        self.guardian.update_config(new_config.guardian);
        self.copilot.update_config(new_config.copilot);

        info!("System configuration updated");
        Ok(())
    }

    pub fn get_config(&self) -> OxidePilotConfig {
        self.config.lock().unwrap().clone()
    }

    pub async fn record_audio(&self, duration_secs: f32) -> Result<Vec<u8>, String> {
        self.voice_processor.record_audio(duration_secs).await
    }

    pub async fn play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        self.voice_processor.play_audio(audio_data).await
    }

    pub fn get_audio_devices(&self) -> (Vec<String>, Vec<String>) {
        (
            self.voice_processor.get_input_devices(),
            self.voice_processor.get_output_devices(),
        )
    }

    pub fn get_input_volume(&self) -> Result<f32, String> {
        self.voice_processor.get_input_volume()
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_monitor.get_metrics()
    }

    pub fn get_performance_score(&self) -> f32 {
        self.performance_monitor.get_performance_score()
    }

    pub async fn optimize_performance(&self) -> Vec<String> {
        let optimizer = self.resource_optimizer.lock().unwrap();
        optimizer.optimize_if_needed()
    }

    pub fn get_performance_metrics(&self) -> oxide_core::performance::PerformanceMetrics {
        // Update system metrics
        let system_status = self.guardian.get_system_status();
        self.performance_monitor
            .update_system_metrics(system_status.cpu_usage, system_status.memory_usage.0);

        self.performance_monitor.get_metrics()
    }

    pub fn get_performance_score(&self) -> f32 {
        self.performance_monitor.get_performance_score()
    }

    pub fn optimize_performance(&self) -> Vec<String> {
        let optimizer = self.resource_optimizer.lock().unwrap();
        optimizer.optimize_if_needed()
    }
}

// Default configuration for easy setup
impl Default for OxidePilotConfig {
    fn default() -> Self {
        Self {
            guardian: GuardianConfig {
                enabled: true,
                monitor_interval_secs: 10,
            },
            copilot: CopilotConfig {
                enabled: true,
                wake_word: "Hey Oxide".to_string(),
            },
            ai_providers: AIProvidersConfig {
                google: Some(GoogleConfig {
                    api_key: "your-google-api-key".to_string(),
                }),
                openai: None,
                anthropic: None,
                azure_openai: None,
                ollama: None,
            },
        }
    }
}
