//! Swarm Worker Implementation
//!
//! Individual worker node implementation for the Aprio swarm system.

use swarm_core::prelude::*;
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SwarmWorker {
    id: Uuid,
    status: Arc<Mutex<WorkerStatus>>,
    config: WorkerConfig,
    task_receiver: Option<mpsc::UnboundedReceiver<Task>>,
    result_sender: Option<mpsc::UnboundedSender<TaskResult>>,
}

impl SwarmWorker {
    pub fn new(config: WorkerConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            status: Arc::new(Mutex::new(WorkerStatus::Idle)),
            config,
            task_receiver: None,
            result_sender: None,
        }
    }

    pub fn with_channels(
        config: WorkerConfig,
        task_receiver: mpsc::UnboundedReceiver<Task>,
        result_sender: mpsc::UnboundedSender<TaskResult>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            status: Arc::new(Mutex::new(WorkerStatus::Idle)),
            config,
            task_receiver: Some(task_receiver),
            result_sender: Some(result_sender),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting swarm worker {}", self.id);
        *self.status.lock().await = WorkerStatus::Running;

        if let (Some(mut task_receiver), Some(result_sender)) = 
            (self.task_receiver.take(), self.result_sender.take()) {
            
            // Main worker loop
            while let Some(task) = task_receiver.recv().await {
                debug!("Worker {} received task {}", self.id, task.id);
                
                // Process the task
                let result = self.process_task(task).await;
                
                // Send result back
                if let Err(e) = result_sender.send(result) {
                    error!("Failed to send task result: {}", e);
                    break;
                }
            }
        }

        *self.status.lock().await = WorkerStatus::Shutdown;
        info!("Worker {} shutdown complete", self.id);
        Ok(())
    }

    async fn process_task(&self, task: Task) -> TaskResult {
        let start_time = std::time::Instant::now();
        
        // Update status to busy
        *self.status.lock().await = WorkerStatus::Busy;
        
        debug!("Processing task {} of type {}", task.id, task.task_type);
        
        // Simulate some work based on task type
        let result = match task.task_type.as_str() {
            "echo" => {
                // Simple echo task - just return the payload
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(Some(task.payload))
            }
            "compute" => {
                // Simple computation task
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                let value = task.payload.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                Ok(Some(serde_json::json!({ "result": value * 2 })))
            }
            "error" => {
                // Simulate an error
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                Err("Simulated processing error".to_string())
            }
            _ => {
                // Unknown task type
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                Err(format!("Unknown task type: {}", task.task_type))
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        // Update status back to idle
        *self.status.lock().await = WorkerStatus::Idle;

        match result {
            Ok(data) => TaskResult {
                task_id: task.id,
                status: TaskStatus::Completed,
                result: data,
                completed_at: chrono::Utc::now().to_rfc3339(),
                processing_time_ms: processing_time,
            },
            Err(error_msg) => TaskResult {
                task_id: task.id,
                status: TaskStatus::Failed(error_msg),
                result: None,
                completed_at: chrono::Utc::now().to_rfc3339(),
                processing_time_ms: processing_time,
            },
        }
    }
}

impl Worker for SwarmWorker {
    fn id(&self) -> Uuid {
        self.id
    }

    fn status(&self) -> &WorkerStatus {
        // Note: This is a bit tricky with async, but we'll return a snapshot
        // In a real implementation, you might want to restructure this
        &WorkerStatus::Idle // This will be updated when we refactor
    }

    fn config(&self) -> &WorkerConfig {
        &self.config
    }
}