//! Worker types and configuration

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub max_concurrent_tasks: usize,
    pub worker_type: String,
    pub capabilities: Vec<String>,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            worker_type: "generic".to_string(),
            capabilities: vec!["basic".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Running,
    Busy,
    Error(String),
    Shutdown,
}

pub trait Worker {
    fn id(&self) -> Uuid;
    fn status(&self) -> &WorkerStatus;
    fn config(&self) -> &WorkerConfig;
}