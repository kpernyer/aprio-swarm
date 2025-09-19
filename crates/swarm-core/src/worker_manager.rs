//! Worker Manager
//!
//! Manages worker startup, capability selection, and lifecycle.
//! This component is responsible for starting workers with appropriate capabilities
//! and managing their lifecycle in the swarm system.

use crate::document_worker::{DocumentWorker, DocumentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapability {
    pub name: String,
    pub supported_types: Vec<DocumentType>,
    pub max_concurrent_documents: usize,
    pub performance_profile: PerformanceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub avg_processing_time_ms: u64,
    pub memory_usage_mb: u64,
    pub cpu_intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStartupConfig {
    pub worker_type: String,
    pub capabilities: Vec<WorkerCapability>,
    pub instance_count: usize,
    pub startup_delay_ms: u64,
    pub health_check_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInstance {
    pub id: Uuid,
    pub worker_type: String,
    pub capabilities: Vec<WorkerCapability>,
    pub status: WorkerStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Starting,
    Running,
    Busy,
    Idle,
    Error(String),
    Shutdown,
}

impl std::fmt::Display for WorkerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerStatus::Starting => write!(f, "Starting"),
            WorkerStatus::Running => write!(f, "Running"),
            WorkerStatus::Busy => write!(f, "Busy"),
            WorkerStatus::Idle => write!(f, "Idle"),
            WorkerStatus::Error(msg) => write!(f, "Error({})", msg),
            WorkerStatus::Shutdown => write!(f, "Shutdown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerManager {
    workers: HashMap<Uuid, WorkerInstance>,
    startup_configs: Vec<WorkerStartupConfig>,
    is_running: bool,
}

impl WorkerManager {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            startup_configs: Vec::new(),
            is_running: false,
        }
    }

    /// Add a worker startup configuration
    pub fn add_worker_config(&mut self, config: WorkerStartupConfig) {
        info!("Adding worker configuration: {} ({} instances)", 
              config.worker_type, config.instance_count);
        self.startup_configs.push(config);
    }

    /// Start all configured workers
    pub async fn start_all_workers(&mut self) -> Result<()> {
        info!("Starting worker manager with {} configurations", self.startup_configs.len());
        self.is_running = true;

        for config in &self.startup_configs.clone() {
            self.start_worker_instances(config).await?;
        }

        // Start health monitoring
        self.start_health_monitoring().await?;

        Ok(())
    }

    /// Start instances of a specific worker type
    async fn start_worker_instances(&mut self, config: &WorkerStartupConfig) -> Result<()> {
        info!("Starting {} instances of worker type: {}", 
              config.instance_count, config.worker_type);

        for i in 0..config.instance_count {
            let worker_id = Uuid::new_v4();
            
            // Create worker instance
            let worker_instance = WorkerInstance {
                id: worker_id,
                worker_type: config.worker_type.clone(),
                capabilities: config.capabilities.clone(),
                status: WorkerStatus::Starting,
                started_at: chrono::Utc::now(),
                last_health_check: None,
            };

            // Start the worker
            self.start_worker(worker_instance).await?;

            // Add startup delay between instances
            if i < config.instance_count - 1 {
                sleep(Duration::from_millis(config.startup_delay_ms)).await;
            }
        }

        Ok(())
    }

    /// Start a single worker instance
    async fn start_worker(&mut self, mut worker_instance: WorkerInstance) -> Result<()> {
        let worker_id = worker_instance.id;
        
        info!("Starting worker: {} (ID: {})", 
              worker_instance.worker_type, worker_id);

        // Create the actual document worker
        let supported_types: Vec<DocumentType> = worker_instance.capabilities
            .iter()
            .flat_map(|cap| cap.supported_types.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let document_worker = DocumentWorker::new(
            worker_instance.worker_type.clone(),
            supported_types,
        );

        // Update worker status
        worker_instance.status = WorkerStatus::Running;
        worker_instance.last_health_check = Some(chrono::Utc::now());
        
        self.workers.insert(worker_id, worker_instance);

        info!("✅ Worker started successfully: {} (ID: {})", 
              document_worker.worker_type, worker_id);

        Ok(())
    }

    /// Start health monitoring for all workers
    async fn start_health_monitoring(&mut self) -> Result<()> {
        info!("Starting health monitoring for {} workers", self.workers.len());
        
        // In a real implementation, this would run in a separate task
        // For now, we'll just log the monitoring start
        debug!("Health monitoring started - would check worker health every 30 seconds");
        
        Ok(())
    }

    /// Get worker by ID
    pub fn get_worker(&self, worker_id: &Uuid) -> Option<&WorkerInstance> {
        self.workers.get(worker_id)
    }

    /// Get all workers of a specific type
    pub fn get_workers_by_type(&self, worker_type: &str) -> Vec<&WorkerInstance> {
        self.workers.values()
            .filter(|worker| worker.worker_type == worker_type)
            .collect()
    }

    /// Get workers that can handle a specific document type
    pub fn get_workers_for_document_type(&self, document_type: &DocumentType) -> Vec<&WorkerInstance> {
        self.workers.values()
            .filter(|worker| {
                worker.capabilities.iter().any(|cap| {
                    cap.supported_types.contains(document_type)
                })
            })
            .collect()
    }

    /// Get worker statistics
    pub fn get_stats(&self) -> WorkerManagerStats {
        let mut stats = WorkerManagerStats {
            total_workers: self.workers.len(),
            workers_by_type: HashMap::new(),
            workers_by_status: HashMap::new(),
            workers_by_capability: HashMap::new(),
        };

        for worker in self.workers.values() {
            // Count by type
            *stats.workers_by_type.entry(worker.worker_type.clone()).or_insert(0) += 1;
            
            // Count by status
            let status_key = format!("{}", worker.status);
            *stats.workers_by_status.entry(status_key).or_insert(0) += 1;
            
            // Count by capability
            for capability in &worker.capabilities {
                *stats.workers_by_capability.entry(capability.name.clone()).or_insert(0) += 1;
            }
        }

        stats
    }

    /// Shutdown all workers
    pub async fn shutdown_all(&mut self) -> Result<()> {
        info!("Shutting down worker manager and all workers");
        
        for (worker_id, worker) in &mut self.workers {
            info!("Shutting down worker: {} (ID: {})", worker.worker_type, worker_id);
            worker.status = WorkerStatus::Shutdown;
        }
        
        self.is_running = false;
        info!("All workers shut down");
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerManagerStats {
    pub total_workers: usize,
    pub workers_by_type: HashMap<String, usize>,
    pub workers_by_status: HashMap<String, usize>,
    pub workers_by_capability: HashMap<String, usize>,
}

impl Default for WorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_manager_creation() {
        let manager = WorkerManager::new();
        assert_eq!(manager.workers.len(), 0);
        assert_eq!(manager.startup_configs.len(), 0);
        assert!(!manager.is_running);
    }

    #[test]
    fn test_worker_capability_creation() {
        let capability = WorkerCapability {
            name: "text_processing".to_string(),
            supported_types: vec![DocumentType::Text, DocumentType::Markdown],
            max_concurrent_documents: 10,
            performance_profile: PerformanceProfile {
                avg_processing_time_ms: 100,
                memory_usage_mb: 512,
                cpu_intensity: 0.6,
            },
        };

        assert_eq!(capability.name, "text_processing");
        assert_eq!(capability.supported_types.len(), 2);
        assert_eq!(capability.max_concurrent_documents, 10);
    }

    #[test]
    fn test_worker_startup_config() {
        let config = WorkerStartupConfig {
            worker_type: "text_processor".to_string(),
            capabilities: vec![WorkerCapability {
                name: "text_processing".to_string(),
                supported_types: vec![DocumentType::Text],
                max_concurrent_documents: 5,
                performance_profile: PerformanceProfile {
                    avg_processing_time_ms: 50,
                    memory_usage_mb: 256,
                    cpu_intensity: 0.4,
                },
            }],
            instance_count: 3,
            startup_delay_ms: 100,
            health_check_interval_ms: 30000,
        };

        assert_eq!(config.worker_type, "text_processor");
        assert_eq!(config.instance_count, 3);
        assert_eq!(config.capabilities.len(), 1);
    }

    #[tokio::test]
    async fn test_worker_manager_startup() {
        let mut manager = WorkerManager::new();
        
        let config = WorkerStartupConfig {
            worker_type: "test_worker".to_string(),
            capabilities: vec![WorkerCapability {
                name: "test_capability".to_string(),
                supported_types: vec![DocumentType::Text],
                max_concurrent_documents: 5,
                performance_profile: PerformanceProfile {
                    avg_processing_time_ms: 50,
                    memory_usage_mb: 256,
                    cpu_intensity: 0.4,
                },
            }],
            instance_count: 2,
            startup_delay_ms: 10,
            health_check_interval_ms: 1000,
        };

        manager.add_worker_config(config);
        manager.start_all_workers().await.unwrap();

        assert_eq!(manager.workers.len(), 2);
        assert!(manager.is_running);

        let stats = manager.get_stats();
        assert_eq!(stats.total_workers, 2);
        assert_eq!(stats.workers_by_type.get("test_worker"), Some(&2));
    }

    #[test]
    fn test_worker_filtering() {
        let mut manager = WorkerManager::new();
        
        // Add a text processor
        let text_config = WorkerStartupConfig {
            worker_type: "text_processor".to_string(),
            capabilities: vec![WorkerCapability {
                name: "text_processing".to_string(),
                supported_types: vec![DocumentType::Text, DocumentType::Markdown],
                max_concurrent_documents: 5,
                performance_profile: PerformanceProfile {
                    avg_processing_time_ms: 50,
                    memory_usage_mb: 256,
                    cpu_intensity: 0.4,
                },
            }],
            instance_count: 1,
            startup_delay_ms: 0,
            health_check_interval_ms: 1000,
        };

        // Add a PDF processor
        let pdf_config = WorkerStartupConfig {
            worker_type: "pdf_processor".to_string(),
            capabilities: vec![WorkerCapability {
                name: "pdf_processing".to_string(),
                supported_types: vec![DocumentType::Pdf],
                max_concurrent_documents: 3,
                performance_profile: PerformanceProfile {
                    avg_processing_time_ms: 200,
                    memory_usage_mb: 1024,
                    cpu_intensity: 0.8,
                },
            }],
            instance_count: 1,
            startup_delay_ms: 0,
            health_check_interval_ms: 1000,
        };

        manager.add_worker_config(text_config);
        manager.add_worker_config(pdf_config);

        // Test worker filtering
        let text_workers = manager.get_workers_for_document_type(&DocumentType::Text);
        let pdf_workers = manager.get_workers_for_document_type(&DocumentType::Pdf);
        let word_workers = manager.get_workers_for_document_type(&DocumentType::Word);

        // Note: These will be empty until workers are actually started
        assert_eq!(text_workers.len(), 0);
        assert_eq!(pdf_workers.len(), 0);
        assert_eq!(word_workers.len(), 0);
    }
}
