//! Swarm coordination and management

use crate::{Task, TaskResult, WorkerConfig, SwarmResult};
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

pub struct SwarmCoordinator {
    workers: HashMap<Uuid, WorkerHandle>,
    task_queue: Vec<Task>,
    result_receiver: Option<mpsc::UnboundedReceiver<TaskResult>>,
    result_sender: mpsc::UnboundedSender<TaskResult>,
}

struct WorkerHandle {
    task_sender: mpsc::UnboundedSender<Task>,
    worker_id: Uuid,
    config: WorkerConfig,
}

impl SwarmCoordinator {
    pub fn new() -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();
        Self {
            workers: HashMap::new(),
            task_queue: Vec::new(),
            result_receiver: Some(result_receiver),
            result_sender,
        }
    }

    pub fn register_worker(&mut self, worker_id: Uuid, config: WorkerConfig) -> mpsc::UnboundedReceiver<Task> {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        let handle = WorkerHandle {
            task_sender,
            worker_id,
            config,
        };
        
        self.workers.insert(worker_id, handle);
        info!("Registered worker {}", worker_id);
        task_receiver
    }

    pub fn unregister_worker(&mut self, worker_id: Uuid) -> bool {
        if self.workers.remove(&worker_id).is_some() {
            info!("Unregistered worker {}", worker_id);
            true
        } else {
            warn!("Attempted to unregister unknown worker {}", worker_id);
            false
        }
    }

    pub fn submit_task(&mut self, task: Task) {
        debug!("Submitted task {} of type {}", task.id, task.task_type);
        self.task_queue.push(task);
    }

    pub async fn start(&mut self) -> SwarmResult<()> {
        info!("Starting swarm coordinator with {} workers", self.workers.len());
        
        // Start result processing task
        if let Some(mut result_receiver) = self.result_receiver.take() {
            let result_processor = tokio::spawn(async move {
                while let Some(result) = result_receiver.recv().await {
                    match result.status {
                        crate::TaskStatus::Completed => {
                            info!("Task {} completed in {}ms", result.task_id, result.processing_time_ms);
                        }
                        crate::TaskStatus::Failed => {
                            error!("Task {} failed", result.task_id);
                        }
                        _ => {}
                    }
                }
            });
            
            // Start task distribution loop
            self.distribute_tasks().await?;
            
            // Wait for result processor to finish
            let _ = result_processor.await;
        }
        
        Ok(())
    }

    async fn distribute_tasks(&mut self) -> SwarmResult<()> {
        info!("Starting task distribution loop");
        
        while !self.task_queue.is_empty() || !self.workers.is_empty() {
            // Distribute pending tasks to available workers
            self.distribute_pending_tasks().await;
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        info!("Task distribution complete");
        Ok(())
    }

    async fn distribute_pending_tasks(&mut self) {
        if self.task_queue.is_empty() || self.workers.is_empty() {
            return;
        }

        let mut tasks_to_remove = Vec::new();
        
        for (task_index, task) in self.task_queue.iter().enumerate() {
            // Find an available worker (simple round-robin for now)
            if let Some((worker_id, handle)) = self.workers.iter().next() {
                debug!("Distributing task {} to worker {}", task.id, worker_id);
                
                if let Err(e) = handle.task_sender.send(task.clone()) {
                    error!("Failed to send task to worker {}: {}", worker_id, e);
                    // Remove the worker if it's no longer responding
                    tasks_to_remove.push(task_index);
                } else {
                    tasks_to_remove.push(task_index);
                    break; // Only send one task at a time for simplicity
                }
            }
        }
        
        // Remove distributed tasks (in reverse order to maintain indices)
        for &index in tasks_to_remove.iter().rev() {
            self.task_queue.remove(index);
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn pending_tasks(&self) -> usize {
        self.task_queue.len()
    }

    pub fn get_result_sender(&self) -> mpsc::UnboundedSender<TaskResult> {
        self.result_sender.clone()
    }
}

impl Default for SwarmCoordinator {
    fn default() -> Self {
        Self::new()
    }
}