use chrono::Utc;
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use oxide_copilot::ai::AIOrchestrator;
use oxide_copilot::copilot::CopilotAgent;
use oxide_copilot::functions::FunctionRegistry;
use oxide_core::config::OxidePilotConfig;
use oxide_core::performance::PerformanceMonitor;
// TODO: Implement PerformanceTimer and ResourceOptimizer
// use oxide_core::performance::{PerformanceTimer, ResourceOptimizer};
use oxide_core::input_validation::InputValidator;
use oxide_core::security_manager::{SecurityEvent, SecurityManager, SecurityPolicy};
use oxide_core::types::{Context, Interaction};
use oxide_guardian::guardian::{Guardian, SystemStatus, ThreatEvent};
#[cfg(feature = "surrealdb-metrics")]
use oxide_guardian::{MetricsCollector as GuardianMetricsCollector, MetricsConfig as GuardianMetricsConfig};
use oxide_guardian::scanner::FileScanReport;
use oxide_memory::memory::{ContextQuery, MemoryManager, MemoryStats};
#[cfg(feature = "surrealdb-metrics")]
use oxide_memory::MemoryBackend;
#[cfg(feature = "surrealdb-metrics")]
use oxide_memory::SurrealBackend;
use oxide_voice::voice::{GoogleSTTProvider, GoogleTTSProvider, VoiceProcessor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
#[cfg(feature = "surrealdb-metrics")]
use tokio::task::JoinHandle;
// use std::env; // Reserved for future use
// use crate::cognee_supervisor::CogneeSupervisor; // Reserved for future use

#[cfg(feature = "surrealdb-metrics")]
struct MetricsRuntime {
    collector: Arc<Mutex<GuardianMetricsCollector>>,
    task: Mutex<Option<JoinHandle<()>>>,
}

#[cfg(feature = "surrealdb-metrics")]
impl MetricsRuntime {
    fn new(collector: GuardianMetricsCollector) -> Self {
        Self {
            collector: Arc::new(Mutex::new(collector)),
            task: Mutex::new(None),
        }
    }

    async fn start(&self) {
        let mut task_guard = self.task.lock().await;
        if task_guard.is_some() {
            return;
        }

        let collector = Arc::clone(&self.collector);
        let handle = tokio::spawn(async move {
            let mut guard = collector.lock().await;
            if let Err(err) = guard.start().await {
                error!("Guardian metrics collector terminated: {:#}", err);
            }
        });

        *task_guard = Some(handle);
    }

    async fn stop(&self) {
        let mut task_guard = self.task.lock().await;
        if let Some(handle) = task_guard.take() {
            handle.abort();
            if let Err(err) = handle.await {
                if !err.is_cancelled() {
                    error!("Guardian metrics collector join error: {err}");
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct OxideSystem {
    config: Arc<Mutex<OxidePilotConfig>>,
    guardian: Arc<Guardian>,
    copilot: Arc<CopilotAgent>,
    memory_manager: Arc<MemoryManager>,
    voice_processor: Arc<VoiceProcessor>,
    performance_monitor: Arc<PerformanceMonitor>,
    // TODO: Implement ResourceOptimizer
    // resource_optimizer: Arc<Mutex<ResourceOptimizer>>,
    security_manager: Arc<SecurityManager>,
    input_validator: Arc<InputValidator>,
    is_running: Arc<Mutex<bool>>,
    #[cfg(feature = "surrealdb-metrics")]
    surreal_backend: Option<Arc<SurrealBackend>>,
    #[cfg(feature = "surrealdb-metrics")]
    metrics_runtime: Option<Arc<MetricsRuntime>>,
}

#[allow(dead_code)] // Some methods reserved for future use
impl OxideSystem {
    pub async fn new(
        config: OxidePilotConfig,
        #[cfg(feature = "surrealdb-metrics")] surreal_backend: Option<Arc<SurrealBackend>>,
    ) -> Result<Self, String> {
        info!("Initializing Oxide Pilot System...");

        // Validate configuration
        config
            .validate()
            .map_err(|e| format!("Configuration validation failed: {e}"))?;

        // Load environment (.env support)
        let _ = dotenv::dotenv();

        #[cfg(feature = "surrealdb-metrics")]
        let (
            surreal_backend_arc,
            surreal_memory_backend,
            surreal_metrics_enabled,
            surreal_metrics_interval,
            surreal_db_path,
        ) = {
            let parse_bool = |value: &str| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            };

            let surreal_cfg = config.surreal.clone();
            if let Some(cfg) = surreal_cfg.as_ref() {
                if cfg.distributed {
                    info!(
                        "SurrealDB distributed mode requested (TiKV endpoints: {:?})",
                        cfg.tikv_endpoints
                    );
                }
                if cfg.enable_js_functions {
                    info!("SurrealDB custom JS functions explicitly enabled");
                }
                if cfg.enable_computed_views {
                    info!("SurrealDB computed views explicitly enabled");
                }
            }
            let env_disable = std::env::var("OXIDE_SURREAL_DISABLE")
                .ok()
                .filter(|v| parse_bool(v));
            let env_enable = std::env::var("OXIDE_SURREAL_ENABLE")
                .ok()
                .filter(|v| parse_bool(v));

            let should_enable = if env_disable.is_some() {
                false
            } else if let Some(_) = env_enable {
                true
            } else {
                surreal_cfg
                    .as_ref()
                    .map(|c| c.enabled)
                    .unwrap_or(true)
            };

            let db_path = surreal_cfg
                .as_ref()
                .and_then(|c| c.db_path.clone())
                .or_else(|| std::env::var("OXIDE_DB_PATH").ok())
                .unwrap_or_else(|| "./data/oxide.db".to_string());

            let mut backend = surreal_backend;

            if should_enable {
                if backend.is_none() {
                    match SurrealBackend::new(&db_path).await {
                        Ok(instance) => {
                            info!("Initialized SurrealDB backend at {}", db_path);
                            backend = Some(Arc::new(instance));
                        }
                        Err(e) => {
                            warn!(
                                "Failed to initialize SurrealDB backend at {}: {}",
                                db_path, e
                            );
                        }
                    }
                } else {
                    info!("Using shared SurrealDB backend at {}", db_path);
                }
            } else {
                info!("SurrealDB backend disabled by configuration/environment");
                backend = None;
            }

            let memory_backend = backend
                .as_ref()
                .map(|arc| arc.clone() as Arc<dyn MemoryBackend>);

            let metrics_enabled = surreal_cfg
                .as_ref()
                .map(|c| c.collect_metrics)
                .unwrap_or(false)
                && backend.is_some();

            let metrics_interval = surreal_cfg
                .as_ref()
                .and_then(|c| c.metrics_interval_secs)
                .or(Some(30));

            (
                backend,
                memory_backend,
                metrics_enabled,
                metrics_interval,
                db_path,
            )
        };

#[cfg(feature = "surrealdb-metrics")]
        if surreal_metrics_enabled {
            if let Some(interval) = surreal_metrics_interval {
                info!(
                    "SurrealDB metrics collection enabled ({} second interval)",
                    interval
                );
            }
        }

        #[cfg(feature = "surrealdb-metrics")]
        let metrics_runtime = if surreal_metrics_enabled {
            if let Some(backend_arc) = surreal_backend_arc.clone() {
                let mut metrics_config = GuardianMetricsConfig::default();
                if let Some(interval) = surreal_metrics_interval {
                    if interval > 0 {
                        metrics_config.interval_secs = interval;
                    }
                } else {
                    metrics_config.interval_secs = config.guardian.monitor_interval_secs.max(1);
                }

                info!(
                    "Configuring Guardian metrics collector (interval {}s)",
                    metrics_config.interval_secs
                );

                Some(Arc::new(MetricsRuntime::new(
                    GuardianMetricsCollector::new(backend_arc.clone(), metrics_config),
                )))
            } else {
                warn!("Metrics collection enabled but Surreal backend is unavailable");
                None
            }
        } else {
            None
        };

        #[cfg(all(feature = "cognee", feature = "surrealdb-metrics"))]
        {
            let _ = &surreal_memory_backend;
            let _ = &surreal_db_path;
        }

        // Initialize memory manager (feature-gated Cognee)
        let memory_manager: Arc<MemoryManager> = {
            #[cfg(feature = "cognee")]
            {
                // Resolve effective Cognee config from env then config, with sane defaults
                let cfg_cognee = config.cognee.clone();
                let env_enable = env::var("OXIDE_COGNEE_ENABLE")
                    .ok()
                    .map(|v| v == "1" || v.eq_ignore_ascii_case("true"));
                let enabled_bool = env_enable
                    .or_else(|| cfg_cognee.as_ref().map(|c| c.enabled))
                    .unwrap_or(false);

                if enabled_bool {
                    let base_url = env::var("OXIDE_COGNEE_URL")
                        .ok()
                        .or_else(|| cfg_cognee.as_ref().map(|c| c.url.clone()))
                        .unwrap_or_else(|| "http://127.0.0.1:8765".to_string());

                    // Prefer plaintext env token; fall back to None for now if only encrypted config is present
                    // TODO: decrypt config token with EncryptionManager once key management is wired
                    let token = env::var("OXIDE_COGNEE_TOKEN").ok();

                    // Health check the Cognee sidecar before selecting it
                    match CogneeSupervisor::new(base_url.clone(), token.clone()) {
                        Ok(supervisor) => {
                            match tokio::time::timeout(
                                Duration::from_millis(800),
                                supervisor.health_check(),
                            )
                            .await
                            {
                                Ok(Ok(())) => {
                                    info!("Memory backend: Cognee ({}). Fallback: JSON", base_url);
                                    Arc::new(MemoryManager::with_cognee(
                                        Some("oxide_data".to_string()),
                                        base_url,
                                        token,
                                    ))
                                }
                                Ok(Err(e)) => {
                                    warn!("Cognee health check failed: {}. Attempting to start sidecar...", e);
                                    // Try to autostart sidecar locally (dev-friendly). Best-effort with short timeout.
                                    let (host, port) = {
                                        fn parse_host_port(url: &str) -> (String, u16) {
                                            let mut rest = url;
                                            if let Some(idx) = url.find("://") {
                                                rest = &url[idx + 3..];
                                            }
                                            let rest = rest.split('/').next().unwrap_or(rest);
                                            let mut parts = rest.split(':');
                                            let h = parts.next().unwrap_or("127.0.0.1").to_string();
                                            let p = parts
                                                .next()
                                                .and_then(|s| s.parse::<u16>().ok())
                                                .unwrap_or(8765);
                                            (h, p)
                                        }
                                        parse_host_port(&base_url)
                                    };
                                    let working_dir = {
                                        use std::path::PathBuf;
                                        let candidates = [
                                            PathBuf::from("../cognee-sidecar"),
                                            PathBuf::from("cognee-sidecar"),
                                        ];
                                        let found = candidates.iter().find(|p| {
                                            p.join("cognee_sidecar").join("app.py").exists()
                                        });
                                        found.cloned()
                                    };
                                    let ensured = tokio::time::timeout(
                                        Duration::from_secs(3),
                                        supervisor.ensure_running(
                                            Some("python".to_string()),
                                            Some(host),
                                            Some(port),
                                            working_dir,
                                        ),
                                    )
                                    .await;
                                    match ensured {
                                        Ok(Ok(())) => {
                                            info!("Cognee sidecar started and healthy. Selecting Cognee backend.");
                                            Arc::new(MemoryManager::with_cognee(
                                                Some("oxide_data".to_string()),
                                                base_url,
                                                token,
                                            ))
                                        }
                                        Ok(Err(e2)) => {
                                            warn!("Failed to start Cognee sidecar: {}. Falling back to JSON backend.", e2);
                                            Arc::new(MemoryManager::new(Some(
                                                "oxide_data".to_string(),
                                            )))
                                        }
                                        Err(_) => {
                                            warn!("Timed out starting Cognee sidecar. Falling back to JSON backend.");
                                            Arc::new(MemoryManager::new(Some(
                                                "oxide_data".to_string(),
                                            )))
                                        }
                                    }
                                }
                                Err(_) => {
                                    warn!("Cognee health check timed out. Attempting to start sidecar...");
                                    let (host, port) = {
                                        fn parse_host_port(url: &str) -> (String, u16) {
                                            let mut rest = url;
                                            if let Some(idx) = url.find("://") {
                                                rest = &url[idx + 3..];
                                            }
                                            let rest = rest.split('/').next().unwrap_or(rest);
                                            let mut parts = rest.split(':');
                                            let h = parts.next().unwrap_or("127.0.0.1").to_string();
                                            let p = parts
                                                .next()
                                                .and_then(|s| s.parse::<u16>().ok())
                                                .unwrap_or(8765);
                                            (h, p)
                                        }
                                        parse_host_port(&base_url)
                                    };
                                    let working_dir = {
                                        use std::path::PathBuf;
                                        let candidates = [
                                            PathBuf::from("../cognee-sidecar"),
                                            PathBuf::from("cognee-sidecar"),
                                        ];
                                        let found = candidates.iter().find(|p| {
                                            p.join("cognee_sidecar").join("app.py").exists()
                                        });
                                        found.cloned()
                                    };
                                    let ensured = tokio::time::timeout(
                                        Duration::from_secs(3),
                                        supervisor.ensure_running(
                                            Some("python".to_string()),
                                            Some(host),
                                            Some(port),
                                            working_dir,
                                        ),
                                    )
                                    .await;
                                    match ensured {
                                        Ok(Ok(())) => {
                                            info!("Cognee sidecar started and healthy. Selecting Cognee backend.");
                                            Arc::new(MemoryManager::with_cognee(
                                                Some("oxide_data".to_string()),
                                                base_url,
                                                token,
                                            ))
                                        }
                                        Ok(Err(e2)) => {
                                            warn!("Failed to start Cognee sidecar: {}. Falling back to JSON backend.", e2);
                                            Arc::new(MemoryManager::new(Some(
                                                "oxide_data".to_string(),
                                            )))
                                        }
                                        Err(_) => {
                                            warn!("Timed out starting Cognee sidecar. Falling back to JSON backend.");
                                            Arc::new(MemoryManager::new(Some(
                                                "oxide_data".to_string(),
                                            )))
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to initialize Cognee supervisor: {}. Falling back to JSON backend.", e);
                            Arc::new(MemoryManager::new(Some("oxide_data".to_string())))
                        }
                    }
                } else {
                    info!("Memory backend: JSON (Cognee disabled)");
                    Arc::new(MemoryManager::new(Some("oxide_data".to_string())))
                }
            }
            #[cfg(not(feature = "cognee"))]
            {
                #[cfg(feature = "surrealdb-metrics")]
                {
                    if let Some(backend) = surreal_memory_backend.clone() {
                        info!("Memory backend: SurrealDB (embedded) [{}]", surreal_db_path);
                        Arc::new(MemoryManager::with_backend(
                            Some("oxide_data".to_string()),
                            backend,
                        ))
                    } else {
                        info!("Memory backend: JSON (SurrealDB disabled or unavailable)");
                        Arc::new(MemoryManager::new(Some("oxide_data".to_string())))
                    }
                }
                #[cfg(not(feature = "surrealdb-metrics"))]
                {
                    info!("Memory backend: JSON (binary built without Cognee feature)");
                    Arc::new(MemoryManager::new(Some("oxide_data".to_string())))
                }
            }
        };
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

        let input_devices = voice_processor.get_input_devices().await;
        let output_devices = voice_processor.get_output_devices().await;
        info!("Audio devices - Input: {input_devices:?}, Output: {output_devices:?}");

        // Initialize Performance Monitor
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        // TODO: Implement ResourceOptimizer
        // let resource_optimizer = Arc::new(Mutex::new(ResourceOptimizer::new(Arc::clone(
        //     &performance_monitor,
        // ))));

        // Initialize security components
        let encryption_key = oxide_core::encryption::EncryptionManager::generate_key();
        let security_manager = Arc::new(
            SecurityManager::new(&encryption_key)
                .map_err(|e| format!("Failed to initialize security manager: {e}"))?,
        );
        let input_validator = Arc::new(InputValidator::new());

        let system = Self {
            config: Arc::new(Mutex::new(config)),
            guardian,
            copilot,
            memory_manager,
            voice_processor,
            performance_monitor,
            // resource_optimizer,
            security_manager,
            input_validator,
            is_running: Arc::new(Mutex::new(false)),
            #[cfg(feature = "surrealdb-metrics")]
            surreal_backend: surreal_backend_arc,
            #[cfg(feature = "surrealdb-metrics")]
            metrics_runtime,
        };

        info!("Oxide Pilot System initialized successfully");
        Ok(system)
    }

    pub async fn start(&self) -> Result<(), String> {
        info!("Starting Oxide Pilot System...");

        {
            let mut running = self.is_running.lock().await;
            if *running {
                return Err("System is already running".to_string());
            }
            *running = true;
        }

        // Start Guardian monitoring
        self.guardian.start_monitoring();
        info!("Guardian Agent started");

        #[cfg(feature = "surrealdb-metrics")]
        match (&self.surreal_backend, &self.metrics_runtime) {
            (Some(_backend), Some(runtime)) => {
                runtime.start().await;
                info!("Guardian metrics collector started");
            }
            (Some(_), None) => {
                debug!("SurrealDB backend active without metrics collector runtime");
            }
            _ => {}
        }

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
            let mut running = self.is_running.lock().await;
            *running = false;
        }

        #[cfg(feature = "surrealdb-metrics")]
        if let Some(runtime) = &self.metrics_runtime {
            runtime.stop().await;
            info!("Guardian metrics collector stopped");
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
        let voice_processor: Arc<VoiceProcessor> = Arc::clone(&self.voice_processor);

        tokio::spawn(async move {
            info!("Main system loop started");

            while {
                let running = is_running.lock().await;
                *running
            } {
                tokio::select! {
                    // Handle wake word detection
                    wake_word = voice_receiver.recv() => {
                        if let Some(word) = wake_word {
                            info!("Wake word detected: {word}");

                            // Record real audio for transcription
                            info!("Recording audio for transcription...");
                            match voice_processor.record_audio(3.0).await {
                                Ok(audio_data) => {
                                    info!("Recorded {} bytes of audio", audio_data.len());
                                    match voice_processor.transcribe_audio(audio_data).await {
                                Ok(transcription) => {
                                    if !transcription.is_empty() {
                                        info!("User said: {transcription}");

                                        // Process user input with Copilot
                                        let context = Context {
                                            active_window: None,
                                            system_status: Some(serde_json::json!({
                                                "source": "voice_input",
                                                "timestamp": Utc::now()
                                            })),
                                            recent_events: Vec::new(),
                                        };

                                        match copilot.handle_user_input(transcription.clone(), context.clone()).await {
                                            Ok(response) => {
                                                info!("Copilot response: {response}");

                                                // Store interaction in memory
                                                let interaction = Interaction {
                                                    id: uuid::Uuid::new_v4(),
                                                    timestamp: Utc::now(),
                                                    user_input: transcription,
                                                    agent_response: response.clone(),
                                                    context,
                                                };

                                                if let Err(e) = memory_manager.store_interaction(interaction).await {
                                                    error!("Failed to store interaction: {e}");
                                                }

                                                // Synthesize and play speech response
                                                match voice_processor.synthesize_speech(&response).await {
                                                    Ok(audio_data) => {
                                                        info!("Speech synthesized, {} bytes", audio_data.len());
                                                        if let Err(e) = voice_processor.play_audio(&audio_data).await {
                                                            error!("Failed to play audio: {e}");
                                                        } else {
                                                            info!("Audio played successfully");
                                                        }
                                                    },
                                                    Err(e) => error!("Failed to synthesize speech: {e}"),
                                                }
                                            },
                                            Err(e) => error!("Copilot error: {e}"),
                                        }
                                    }
                                    },
                                    Err(e) => error!("Transcription failed: {e}"),
                                }
                                },
                                Err(e) => error!("Audio recording failed: {e}"),
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
        let stats = memory_manager.get_memory_stats().await;
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
        // TODO: Implement PerformanceTimer
        // let _timer = PerformanceTimer::new(
        //     "handle_text_input".to_string(),
        //     Arc::clone(&self.performance_monitor),
        // );
        info!("Handling text input: {input}");

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
            active_window: None,
            system_status: Some(serde_json::json!({
                "memory_entries": relevant_memories.len(),
                "timestamp": Utc::now()
            })),
            recent_events: Vec::new(),
        };

        // Process with Copilot
        let response = self
            .copilot
            .handle_user_input(input.clone(), context.clone())
            .await
            .map_err(|e| e.to_string())?;

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

    pub async fn get_memory_stats(&self) -> MemoryStats {
        self.memory_manager.get_memory_stats().await
    }

    pub async fn update_config(&self, new_config: OxidePilotConfig) -> Result<(), String> {
        new_config.validate()?;

        {
            let mut config = self.config.lock().await;
            *config = new_config.clone();
        }

        // Update individual components
        self.guardian.update_config(new_config.guardian);
        self.copilot.update_config(new_config.copilot).await;

        info!("System configuration updated");
        Ok(())
    }

    pub async fn get_config(&self) -> OxidePilotConfig {
        self.config.lock().await.clone()
    }

    pub async fn record_audio(&self, duration_secs: f32) -> Result<Vec<u8>, String> {
        self.voice_processor.record_audio(duration_secs).await
    }

    pub async fn play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        self.voice_processor.play_audio(audio_data).await
    }

    pub async fn get_audio_devices(&self) -> (Vec<String>, Vec<String>) {
        (
            self.voice_processor.get_input_devices().await,
            self.voice_processor.get_output_devices().await,
        )
    }

    pub async fn get_input_volume(&self) -> Result<f32, String> {
        self.voice_processor.get_input_volume().await
    }

    pub async fn get_performance_metrics(&self) -> oxide_core::performance::PerformanceMetrics {
        // Update system metrics
        let system_status = self.guardian.get_system_status();
        let memory_usage_mb = (system_status.memory_usage.0 as f32) / (1024.0 * 1024.0);
        self.performance_monitor
            .update_system_metrics(system_status.cpu_usage, memory_usage_mb)
            .await;

        self.performance_monitor.get_metrics().await
    }

    pub async fn get_performance_score(&self) -> f32 {
        self.performance_monitor.get_performance_score().await
    }

    pub async fn optimize_performance(&self) -> Vec<String> {
        // TODO: Implement ResourceOptimizer
        // let optimizer = self.resource_optimizer.lock().await;
        // optimizer.optimize_if_needed().await
        vec!["Performance optimization not yet implemented".to_string()]
    }

    // TODO: Implement PerformanceAlert and PerformanceProfile types
    // pub async fn get_performance_alerts(&self) -> Vec<oxide_core::performance::PerformanceAlert> {
    //     self.performance_monitor.get_alerts().await
    // }

    pub async fn clear_performance_alerts(&self) {
        // TODO: Implement clear_alerts method
        // self.performance_monitor.clear_alerts().await
    }

    // pub async fn get_operation_profiles(&self) -> std::collections::HashMap<String, oxide_core::performance::PerformanceProfile> {
    //     self.performance_monitor.get_operation_profiles().await
    // }

    pub async fn set_performance_monitoring(&self, _enabled: bool) {
        // TODO: Implement set_monitoring_enabled method
        // self.performance_monitor.set_monitoring_enabled(enabled).await
    }

    // File scanning API plumbing for frontend commands
    pub async fn scan_file(
        &self,
        path: String,
        use_cloud: bool,
        quarantine: bool,
    ) -> Result<FileScanReport, String> {
        // Check antivirus feature toggle (defaults to enabled if not set)
        let av_enabled = {
            let cfg = self.config.lock().await;
            cfg.guardian.antivirus_enabled.unwrap_or(true)
        };
        if !av_enabled {
            return Err("Antivirus scanning is disabled in settings".to_string());
        }

        // Optional rate limiting for cloud lookups
        if use_cloud {
            self.security_manager
                .check_rate_limit("antivirus_cloud_scan")
                .await
                .map_err(|e| e.to_string())?;
        }

        // Resolve VirusTotal API key if cloud scanning requested
        let vt_key: Option<String> = if use_cloud {
            // Prefer env override
            if let Ok(k) = std::env::var("VIRUSTOTAL_API_KEY") {
                if !k.is_empty() {
                    Some(k)
                } else {
                    None
                }
            } else {
                // Fallback to encrypted key from config
                let enc = {
                    let cfg = self.config.lock().await;
                    cfg.guardian.virustotal_api_key.clone()
                };
                if let Some(ed) = enc {
                    let bytes = self
                        .decrypt_data(&ed)
                        .map_err(|e| format!("Failed to decrypt VirusTotal API key: {e}"))?;
                    let s = String::from_utf8(bytes).map_err(|_| {
                        "Decrypted VirusTotal API key is not valid UTF-8".to_string()
                    })?;
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                } else {
                    None
                }
            }
        } else {
            None
        };

        // Offload blocking scan (file IO + potential blocking HTTP) to a blocking thread
        let guardian = self.guardian.clone();
        let path_cloned = path.clone();
        tokio::task::spawn_blocking(move || guardian.scan_file(&path_cloned, vt_key, quarantine))
            .await
            .map_err(|e| format!("Scan task join error: {e}"))?
    }

    /// Returns true if a VirusTotal API key is configured via env or encrypted config.
    pub async fn has_virustotal_key(&self) -> bool {
        if let Ok(k) = std::env::var("VIRUSTOTAL_API_KEY") {
            if !k.is_empty() {
                return true;
            }
        }

        // Check encrypted key in config
        let enc = {
            let cfg = self.config.lock().await;
            cfg.guardian.virustotal_api_key.clone()
        };
        if let Some(ed) = enc {
            if let Ok(bytes) = self.decrypt_data(&ed) {
                if let Ok(s) = String::from_utf8(bytes) {
                    return !s.is_empty();
                }
            }
        }
        false
    }

    // Security-related methods
    pub async fn validate_input(&self, field_name: &str, value: &str) -> Result<String, String> {
        self.input_validator
            .validate(field_name, value)
            .map_err(|e| e.to_string())
    }

    pub async fn create_security_session(
        &self,
        user_id: String,
        permissions: Vec<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<String, String> {
        let session = self
            .security_manager
            .create_session(user_id, permissions, ip_address, user_agent)
            .await
            .map_err(|e| e.to_string())?;

        Ok(session.session_id)
    }

    pub async fn validate_security_session(&self, session_id: &str) -> Result<bool, String> {
        match self.security_manager.validate_session(session_id).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn check_security_permission(
        &self,
        session_id: &str,
        permission: &str,
    ) -> Result<bool, String> {
        self.security_manager
            .check_permission(session_id, permission)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn invalidate_security_session(&self, session_id: &str) -> Result<(), String> {
        self.security_manager
            .invalidate_session(session_id)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_security_events(&self, limit: Option<usize>) -> Vec<SecurityEvent> {
        self.security_manager.get_security_events(limit).await
    }

    pub async fn update_security_policy(&self, policy: SecurityPolicy) {
        self.security_manager.update_security_policy(policy).await
    }

    pub async fn get_security_policy(&self) -> SecurityPolicy {
        self.security_manager.get_security_policy().await
    }

    // Convenience: decrypt an `EncryptedData` blob using the system SecurityManager
    pub fn decrypt_data(
        &self,
        encrypted: &oxide_core::encryption::EncryptedData,
    ) -> Result<Vec<u8>, String> {
        self.security_manager
            .decrypt_data(encrypted)
            .map_err(|e| e.to_string())
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), String> {
        self.security_manager
            .check_rate_limit(identifier)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn cleanup_security_sessions(&self) {
        self.security_manager.cleanup_expired_sessions().await
    }
}

// Default configuration for easy setup
// Note: OxidePilotConfig is defined in oxide-core, so we can't implement Default here
// This implementation should be moved to oxide-core where OxidePilotConfig is defined
