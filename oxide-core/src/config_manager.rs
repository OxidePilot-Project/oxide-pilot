use crate::config::OxidePilotConfig;
use std::fs;
use std::path::Path;
use notify::{Watcher, RecursiveMode, recommended_watcher};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;

pub fn load_config(path: &Path) -> Result<OxidePilotConfig, String> {
    let config_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: OxidePilotConfig = serde_json::from_str(&config_str).map_err(|e| e.to_string())?;
    config.validate()?;
    Ok(config)
}

pub fn watch_config(path: &Path, sender: Sender<OxidePilotConfig>) -> Result<(), String> {
    let (tx, rx): (Sender<notify::Result<notify::Event>>, Receiver<notify::Result<notify::Event>>) = channel();

    let mut watcher = recommended_watcher(tx).map_err(|e| e.to_string())?;

    watcher.watch(path, RecursiveMode::NonRecursive).map_err(|e| e.to_string())?;

    // Initial load
    match load_config(path) {
        Ok(config) => {
            sender.send(config).map_err(|e| e.to_string())?;
        },
        Err(e) => {
            eprintln!("Initial config load error: {e}");
        }
    }

    for res in rx {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    // Debounce to avoid multiple events for a single save
                    std::thread::sleep(Duration::from_millis(100));
                    match load_config(path) {
                        Ok(config) => {
                            sender.send(config).map_err(|e| e.to_string())?;
                        },
                        Err(e) => {
                            eprintln!("Config reload error: {e}");
                        }
                    }
                }
            },
            Err(e) => eprintln!("watch error: {e:?}"),
        }
    }
    Ok(())
}