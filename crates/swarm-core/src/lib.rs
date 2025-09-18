//! Swarm Core - Distributed worker coordination system
//!
//! High-performance Rust-based worker system for the Living Twin platform.
//! Provides distributed processing capabilities for AI agents.

pub mod coordinator;
pub mod worker;
pub mod task;
pub mod error;

pub use coordinator::SwarmCoordinator;
pub use worker::{Worker, WorkerConfig, WorkerStatus};
pub use task::{Task, TaskResult, TaskStatus};
pub use error::{SwarmError, SwarmResult};

/// Core traits and types for the swarm system
pub mod prelude {
    pub use crate::{SwarmCoordinator, Worker, WorkerConfig, WorkerStatus};
    pub use crate::{Task, TaskResult, TaskStatus};
    pub use crate::{SwarmError, SwarmResult};
}
