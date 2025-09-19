//! Error types for the swarm system

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwarmError {
    #[error("Worker error: {0}")]
    Worker(String),

    #[error("Task error: {0}")]
    Task(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Coordination error: {0}")]
    Coordination(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type SwarmResult<T> = Result<T, SwarmError>;