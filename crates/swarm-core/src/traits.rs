//! Core Traits and Interfaces
//!
//! This module defines the fundamental traits that all swarm components must implement.
//! These traits provide clean abstractions and enable dependency injection and testing.

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Core trait for all workers in the swarm system
#[async_trait]
pub trait Worker: Send + Sync {
    /// Unique identifier for this worker
    fn id(&self) -> Uuid;
    
    /// Human-readable name for this worker
    fn name(&self) -> &str;
    
    /// Worker type identifier
    fn worker_type(&self) -> WorkerType;
    
    /// Current status of the worker
    fn status(&self) -> WorkerStatus;
    
    /// Maximum number of concurrent tasks this worker can handle
    fn max_concurrent_tasks(&self) -> usize;
    
    /// Current number of active tasks
    fn current_load(&self) -> usize;
    
    /// Check if worker has capacity for more tasks
    fn has_capacity(&self) -> bool {
        self.current_load() < self.max_concurrent_tasks()
    }
    
    /// Get worker capabilities
    fn capabilities(&self) -> &[WorkerCapability];
    
    /// Check if worker can handle a specific task type
    fn can_handle(&self, task_type: &TaskType) -> bool;
    
    /// Process a task asynchronously
    async fn process_task(&mut self, task: Task) -> Result<TaskResult>;
    
    /// Health check for the worker
    async fn health_check(&self) -> Result<WorkerHealth>;
    
    /// Graceful shutdown
    async fn shutdown(&mut self) -> Result<()>;
}

/// Trait for task processing
#[async_trait]
pub trait TaskProcessor: Send + Sync {
    /// Process a single task
    async fn process(&self, task: &Task) -> Result<TaskResult>;
    
    /// Get supported task types
    fn supported_task_types(&self) -> &[TaskType];
    
    /// Estimate processing time for a task
    fn estimate_processing_time(&self, task: &Task) -> std::time::Duration;
}

/// Trait for document processing
#[async_trait]
pub trait DocumentProcessor: Send + Sync {
    /// Process a document
    async fn process_document(&self, document: &Document) -> Result<DocumentProcessingResult>;
    
    /// Get supported document types
    fn supported_document_types(&self) -> &[DocumentType];
    
    /// Check if processor can handle a document type
    fn can_process(&self, document_type: &DocumentType) -> bool;
}

/// Trait for worker coordination
#[async_trait]
pub trait WorkerCoordinator: Send + Sync {
    /// Register a worker with the coordinator
    async fn register_worker(&mut self, worker: Box<dyn Worker>) -> Result<()>;
    
    /// Unregister a worker
    async fn unregister_worker(&mut self, worker_id: Uuid) -> Result<()>;
    
    /// Get available workers for a task type
    async fn get_available_workers(&self, task_type: &TaskType) -> Vec<Uuid>;
    
    /// Assign a task to a worker
    async fn assign_task(&mut self, task: Task) -> Result<Uuid>;
    
    /// Get worker status
    async fn get_worker_status(&self, worker_id: Uuid) -> Result<WorkerStatus>;
    
    /// Get coordinator statistics
    async fn get_stats(&self) -> CoordinatorStats;
}

/// Trait for task scheduling
#[async_trait]
pub trait TaskScheduler: Send + Sync {
    /// Schedule a task for processing
    async fn schedule_task(&mut self, task: Task) -> Result<()>;
    
    /// Get next available task
    async fn get_next_task(&mut self) -> Result<Option<Task>>;
    
    /// Get task queue statistics
    async fn get_queue_stats(&self) -> TaskQueueStats;
    
    /// Cancel a scheduled task
    async fn cancel_task(&mut self, task_id: Uuid) -> Result<()>;
}

/// Trait for document reading and discovery
#[async_trait]
pub trait DocumentReader: Send + Sync {
    /// Start reading documents from configured sources
    async fn start(&mut self) -> Result<()>;
    
    /// Stop reading documents
    async fn stop(&mut self) -> Result<()>;
    
    /// Get next available document
    async fn get_next_document(&mut self) -> Result<Option<Document>>;
    
    /// Get reader statistics
    async fn get_stats(&self) -> DocumentReaderStats;
}

/// Trait for messaging between components
#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Publish a message to a subject
    async fn publish(&self, subject: &str, message: &[u8]) -> Result<()>;
    
    /// Subscribe to a subject
    async fn subscribe(&self, subject: &str) -> Result<Box<dyn MessageSubscription>>;
    
    /// Get broker statistics
    async fn get_stats(&self) -> MessageBrokerStats;
}

/// Message subscription handle
pub trait MessageSubscription: Send + Sync {
    /// Get next message from subscription
    fn next_message(&mut self) -> Result<Option<Message>>;
    
    /// Unsubscribe from the subject
    fn unsubscribe(self) -> Result<()>;
}

/// Trait for monitoring and metrics
pub trait MetricsCollector: Send + Sync {
    /// Record a metric value
    fn record_metric(&self, name: &str, value: f64, tags: &[(&str, &str)]);
    
    /// Increment a counter
    fn increment_counter(&self, name: &str, tags: &[(&str, &str)]);
    
    /// Record a histogram value
    fn record_histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]);
    
    /// Get all collected metrics
    fn get_metrics(&self) -> HashMap<String, MetricValue>;
}

/// Trait for configuration management
pub trait ConfigProvider: Send + Sync {
    /// Get a configuration value
    fn get<T>(&self, key: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned;
    
    /// Set a configuration value
    fn set<T>(&mut self, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize;
    
    /// Check if a configuration key exists
    fn has(&self, key: &str) -> bool;
    
    /// Get all configuration keys
    fn keys(&self) -> Vec<String>;
}

// Re-export commonly used types
pub use crate::types::{
    Task, TaskResult, TaskStatus, TaskPriority, TaskType,
    WorkerConfig, WorkerStatus, WorkerType, WorkerCapability,
    Document, DocumentType, DocumentProcessingResult,
    Message, MessageBrokerStats, CoordinatorStats, TaskQueueStats,
    DocumentReaderStats, WorkerHealth, MetricValue,
};

// Include tests module
#[cfg(test)]
mod tests;
