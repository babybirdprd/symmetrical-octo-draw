use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use crate::model::{Action, AgentConfig};

// Global broadcast channel for agent actions
pub static AGENT_CHANNEL: Lazy<broadcast::Sender<Action>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(100);
    tx
});

// Global history for reliable polling
pub static AGENT_HISTORY: Lazy<Mutex<Vec<Action>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

// Global agent configuration
pub static AGENT_CONFIG: Lazy<Mutex<AgentConfig>> = Lazy::new(|| {
    Mutex::new(AgentConfig::default())
});
