//! Swarm Core - Distributed worker coordination system
//!
//! High-performance Rust-based worker system for the Aprio platform.
//! Provides distributed processing capabilities for AI agents.
//!
//! This crate defines the core traits, types, and interfaces that all
//! swarm components must implement. It provides clean abstractions for
//! workers, tasks, documents, and coordination.

// Core modules
pub mod traits;
pub mod types;
pub mod error;

// Legacy modules (to be refactored)
pub mod coordinator;
pub mod worker;
pub mod task;
pub mod document_worker;
pub mod document_reader;
pub mod worker_manager;

// Re-export core traits and types
pub use traits::*;
pub use types::*;
pub use error::{SwarmError, SwarmResult};

// Legacy re-exports (for backward compatibility)
pub use coordinator::SwarmCoordinator;
pub use worker::{Worker as LegacyWorker, WorkerConfig as LegacyWorkerConfig, WorkerStatus as LegacyWorkerStatus};
pub use task::{Task as LegacyTask, TaskResult as LegacyTaskResult, TaskStatus as LegacyTaskStatus, TaskPriority as LegacyTaskPriority};
pub use document_worker::{DocumentWorker, Document as LegacyDocument, DocumentType as LegacyDocumentType, DocumentProcessingResult as LegacyDocumentProcessingResult};
pub use document_reader::{DocumentReader, DocumentReaderConfig, DocumentReaderStats, ProcessingTask};
pub use worker_manager::{WorkerManager, WorkerCapability as LegacyWorkerCapability, WorkerStartupConfig, WorkerInstance, WorkerManagerStats, PerformanceProfile};

/// Core traits and types for the swarm system
pub mod prelude {
    // Core traits
    pub use crate::traits::{
        Worker, TaskProcessor, DocumentProcessor, WorkerCoordinator,
        TaskScheduler, DocumentReader, MessageBroker, MetricsCollector, ConfigProvider,
    };
    
    // Core types
    pub use crate::types::{
        Task, TaskResult, TaskStatus, TaskPriority, TaskType, TaskPayload,
        WorkerConfig, WorkerType, WorkerStatus, WorkerCapability, WorkerHealth,
        Document, DocumentType, DocumentContent, DocumentProcessingResult,
        Message, MessageBrokerStats, CoordinatorStats, TaskQueueStats,
        DocumentReaderStats, MetricValue, PerformanceProfile,
    };
    
    // Error types
    pub use crate::error::{SwarmError, SwarmResult};
    
    // Legacy types (for backward compatibility)
    pub use crate::{
        SwarmCoordinator, LegacyWorker, LegacyWorkerConfig, LegacyWorkerStatus,
        LegacyTask, LegacyTaskResult, LegacyTaskStatus, LegacyTaskPriority,
        DocumentWorker, LegacyDocument, LegacyDocumentType, LegacyDocumentProcessingResult,
        DocumentReader as LegacyDocumentReader, DocumentReaderConfig, DocumentReaderStats as LegacyDocumentReaderStats, ProcessingTask,
        WorkerManager, LegacyWorkerCapability, WorkerStartupConfig, WorkerInstance, WorkerManagerStats,
    };
}
