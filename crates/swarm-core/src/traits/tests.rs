//! Unit tests for core traits
//!
//! This module contains comprehensive unit tests for all core traits.
//! Each trait has its own test module to ensure proper isolation and organization.

use super::*;
use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

// ============================================================================
// MOCK IMPLEMENTATIONS FOR TESTING
// ============================================================================

/// Mock worker implementation for testing
pub struct MockWorker {
    pub id: Uuid,
    pub name: String,
    pub worker_type: WorkerType,
    pub status: WorkerStatus,
    pub max_concurrent_tasks: usize,
    pub current_load: usize,
    pub capabilities: Vec<WorkerCapability>,
}

#[async_trait]
impl Worker for MockWorker {
    fn id(&self) -> Uuid {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn worker_type(&self) -> WorkerType {
        self.worker_type.clone()
    }
    
    fn status(&self) -> WorkerStatus {
        self.status.clone()
    }
    
    fn max_concurrent_tasks(&self) -> usize {
        self.max_concurrent_tasks
    }
    
    fn current_load(&self) -> usize {
        self.current_load
    }
    
    fn capabilities(&self) -> &[WorkerCapability] {
        &self.capabilities
    }
    
    fn can_handle(&self, task_type: &TaskType) -> bool {
        self.capabilities.iter().any(|cap| {
            cap.supported_task_types.contains(task_type)
        })
    }
    
    async fn process_task(&mut self, _task: Task) -> Result<TaskResult> {
        // Mock implementation
        Ok(TaskResult {
            task_id: Uuid::new_v4(),
            status: TaskStatus::Completed,
            result: None,
            error: None,
            processing_time_ms: 100,
            completed_at: Utc::now(),
            metadata: HashMap::new(),
        })
    }
    
    async fn health_check(&self) -> Result<WorkerHealth> {
        Ok(WorkerHealth {
            worker_id: self.id,
            status: self.status.clone(),
            current_load: self.current_load,
            max_capacity: self.max_concurrent_tasks,
            memory_usage_mb: 512,
            cpu_usage_percent: 50.0,
            last_heartbeat: Utc::now(),
            error_count: 0,
            success_count: 100,
        })
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        self.status = WorkerStatus::Shutdown;
        Ok(())
    }
}

/// Mock task processor implementation
pub struct MockTaskProcessor {
    pub supported_task_types: Vec<TaskType>,
}

#[async_trait]
impl TaskProcessor for MockTaskProcessor {
    async fn process(&self, _task: &Task) -> Result<TaskResult> {
        Ok(TaskResult {
            task_id: Uuid::new_v4(),
            status: TaskStatus::Completed,
            result: None,
            error: None,
            processing_time_ms: 50,
            completed_at: Utc::now(),
            metadata: HashMap::new(),
        })
    }
    
    fn supported_task_types(&self) -> &[TaskType] {
        &self.supported_task_types
    }
    
    fn estimate_processing_time(&self, _task: &Task) -> std::time::Duration {
        std::time::Duration::from_millis(100)
    }
}

/// Mock document processor implementation
pub struct MockDocumentProcessor {
    pub supported_document_types: Vec<DocumentType>,
}

#[async_trait]
impl DocumentProcessor for MockDocumentProcessor {
    async fn process_document(&self, _document: &Document) -> Result<DocumentProcessingResult> {
        Ok(DocumentProcessingResult {
            document_id: Uuid::new_v4(),
            extracted_text: Some("Mock extracted text".to_string()),
            metadata: HashMap::new(),
            language: Some("en".to_string()),
            keywords: vec!["mock".to_string(), "test".to_string()],
            sentiment: Some(0.5),
            classification: Some("test".to_string()),
            embeddings: None,
            processing_time_ms: 75,
            processed_at: Utc::now(),
        })
    }
    
    fn supported_document_types(&self) -> &[DocumentType] {
        &self.supported_document_types
    }
    
    fn can_process(&self, document_type: &DocumentType) -> bool {
        self.supported_document_types.contains(document_type)
    }
}

// ============================================================================
// WORKER TRAIT TESTS
// ============================================================================

#[cfg(test)]
mod worker_tests {
    use super::*;
    
    fn create_mock_worker() -> MockWorker {
        MockWorker {
            id: Uuid::new_v4(),
            name: "test-worker".to_string(),
            worker_type: WorkerType::DocumentProcessor {
                supported_types: vec![DocumentType::Pdf, DocumentType::Text],
            },
            status: WorkerStatus::Running,
            max_concurrent_tasks: 5,
            current_load: 2,
            capabilities: vec![WorkerCapability {
                name: "document_processing".to_string(),
                version: "1.0.0".to_string(),
                supported_task_types: vec![
                    TaskType::DocumentProcessing {
                        document_type: DocumentType::Pdf,
                        processing_type: DocumentProcessingType::TextExtraction,
                    },
                ],
                max_concurrent_tasks: 5,
                performance_profile: PerformanceProfile {
                    avg_processing_time_ms: 100,
                    memory_usage_mb: 512,
                    cpu_intensity: 0.5,
                    throughput_per_second: 10.0,
                },
                metadata: HashMap::new(),
            }],
        }
    }
    
    #[test]
    fn test_worker_id() {
        let worker = create_mock_worker();
        assert!(!worker.id().is_nil());
    }
    
    #[test]
    fn test_worker_name() {
        let worker = create_mock_worker();
        assert_eq!(worker.name(), "test-worker");
    }
    
    #[test]
    fn test_worker_status() {
        let worker = create_mock_worker();
        assert_eq!(worker.status(), WorkerStatus::Running);
    }
    
    #[test]
    fn test_worker_capacity() {
        let worker = create_mock_worker();
        assert_eq!(worker.max_concurrent_tasks(), 5);
        assert_eq!(worker.current_load(), 2);
        assert!(worker.has_capacity());
    }
    
    #[test]
    fn test_worker_no_capacity() {
        let mut worker = create_mock_worker();
        worker.current_load = 5;
        assert!(!worker.has_capacity());
    }
    
    #[test]
    fn test_worker_capabilities() {
        let worker = create_mock_worker();
        assert_eq!(worker.capabilities().len(), 1);
        assert_eq!(worker.capabilities()[0].name, "document_processing");
    }
    
    #[test]
    fn test_worker_can_handle() {
        let worker = create_mock_worker();
        let task_type = TaskType::DocumentProcessing {
            document_type: DocumentType::Pdf,
            processing_type: DocumentProcessingType::TextExtraction,
        };
        assert!(worker.can_handle(&task_type));
    }
    
    #[test]
    fn test_worker_cannot_handle() {
        let worker = create_mock_worker();
        let task_type = TaskType::TextAnalysis {
            analysis_type: TextAnalysisType::LanguageDetection,
        };
        assert!(!worker.can_handle(&task_type));
    }
    
    #[tokio::test]
    async fn test_worker_process_task() {
        let mut worker = create_mock_worker();
        let task = Task {
            id: Uuid::new_v4(),
            task_type: TaskType::DocumentProcessing {
                document_type: DocumentType::Pdf,
                processing_type: DocumentProcessingType::TextExtraction,
            },
            priority: TaskPriority::Normal,
            status: TaskStatus::Pending,
            payload: TaskPayload::Document {
                document: Document {
                    id: Uuid::new_v4(),
                    filename: "test.pdf".to_string(),
                    document_type: DocumentType::Pdf,
                    content: DocumentContent::Text("test content".to_string()),
                    metadata: HashMap::new(),
                    created_at: Utc::now(),
                    size_bytes: 100,
                },
                processing_options: DocumentProcessingOptions::default(),
            },
            created_at: Utc::now(),
            deadline: None,
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
        };
        
        let result = worker.process_task(task).await.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.processing_time_ms, 100);
    }
    
    #[tokio::test]
    async fn test_worker_health_check() {
        let worker = create_mock_worker();
        let health = worker.health_check().await.unwrap();
        
        assert_eq!(health.worker_id, worker.id);
        assert_eq!(health.status, WorkerStatus::Running);
        assert_eq!(health.current_load, 2);
        assert_eq!(health.max_capacity, 5);
        assert_eq!(health.memory_usage_mb, 512);
        assert_eq!(health.cpu_usage_percent, 50.0);
    }
    
    #[tokio::test]
    async fn test_worker_shutdown() {
        let mut worker = create_mock_worker();
        assert_eq!(worker.status(), WorkerStatus::Running);
        
        worker.shutdown().await.unwrap();
        assert_eq!(worker.status(), WorkerStatus::Shutdown);
    }
}

// ============================================================================
// TASK PROCESSOR TRAIT TESTS
// ============================================================================

#[cfg(test)]
mod task_processor_tests {
    use super::*;
    
    fn create_mock_task_processor() -> MockTaskProcessor {
        MockTaskProcessor {
            supported_task_types: vec![
                TaskType::DocumentProcessing {
                    document_type: DocumentType::Pdf,
                    processing_type: DocumentProcessingType::TextExtraction,
                },
            ],
        }
    }
    
    #[test]
    fn test_supported_task_types() {
        let processor = create_mock_task_processor();
        assert_eq!(processor.supported_task_types().len(), 1);
    }
    
    #[test]
    fn test_estimate_processing_time() {
        let processor = create_mock_task_processor();
        let task = Task {
            id: Uuid::new_v4(),
            task_type: TaskType::DocumentProcessing {
                document_type: DocumentType::Pdf,
                processing_type: DocumentProcessingType::TextExtraction,
            },
            priority: TaskPriority::Normal,
            status: TaskStatus::Pending,
            payload: TaskPayload::Document {
                document: Document {
                    id: Uuid::new_v4(),
                    filename: "test.pdf".to_string(),
                    document_type: DocumentType::Pdf,
                    content: DocumentContent::Text("test".to_string()),
                    metadata: HashMap::new(),
                    created_at: Utc::now(),
                    size_bytes: 100,
                },
                processing_options: DocumentProcessingOptions::default(),
            },
            created_at: Utc::now(),
            deadline: None,
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
        };
        
        let duration = processor.estimate_processing_time(&task);
        assert_eq!(duration.as_millis(), 100);
    }
    
    #[tokio::test]
    async fn test_process_task() {
        let processor = create_mock_task_processor();
        let task = Task {
            id: Uuid::new_v4(),
            task_type: TaskType::DocumentProcessing {
                document_type: DocumentType::Pdf,
                processing_type: DocumentProcessingType::TextExtraction,
            },
            priority: TaskPriority::Normal,
            status: TaskStatus::Pending,
            payload: TaskPayload::Document {
                document: Document {
                    id: Uuid::new_v4(),
                    filename: "test.pdf".to_string(),
                    document_type: DocumentType::Pdf,
                    content: DocumentContent::Text("test".to_string()),
                    metadata: HashMap::new(),
                    created_at: Utc::now(),
                    size_bytes: 100,
                },
                processing_options: DocumentProcessingOptions::default(),
            },
            created_at: Utc::now(),
            deadline: None,
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
        };
        
        let result = processor.process(&task).await.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.processing_time_ms, 50);
    }
}

// ============================================================================
// DOCUMENT PROCESSOR TRAIT TESTS
// ============================================================================

#[cfg(test)]
mod document_processor_tests {
    use super::*;
    
    fn create_mock_document_processor() -> MockDocumentProcessor {
        MockDocumentProcessor {
            supported_document_types: vec![DocumentType::Pdf, DocumentType::Text],
        }
    }
    
    #[test]
    fn test_supported_document_types() {
        let processor = create_mock_document_processor();
        assert_eq!(processor.supported_document_types().len(), 2);
        assert!(processor.supported_document_types().contains(&DocumentType::Pdf));
        assert!(processor.supported_document_types().contains(&DocumentType::Text));
    }
    
    #[test]
    fn test_can_process() {
        let processor = create_mock_document_processor();
        assert!(processor.can_process(&DocumentType::Pdf));
        assert!(processor.can_process(&DocumentType::Text));
        assert!(!processor.can_process(&DocumentType::Word));
    }
    
    #[tokio::test]
    async fn test_process_document() {
        let processor = create_mock_document_processor();
        let document = Document {
            id: Uuid::new_v4(),
            filename: "test.pdf".to_string(),
            document_type: DocumentType::Pdf,
            content: DocumentContent::Text("test content".to_string()),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            size_bytes: 100,
        };
        
        let result = processor.process_document(&document).await.unwrap();
        assert_eq!(result.extracted_text, Some("Mock extracted text".to_string()));
        assert_eq!(result.language, Some("en".to_string()));
        assert_eq!(result.keywords, vec!["mock", "test"]);
        assert_eq!(result.sentiment, Some(0.5));
        assert_eq!(result.processing_time_ms, 75);
    }
}

// ============================================================================
// TYPE DISPLAY TESTS
// ============================================================================

#[cfg(test)]
mod display_tests {
    use super::*;
    
    #[test]
    fn test_task_type_display() {
        let task_type = TaskType::DocumentProcessing {
            document_type: DocumentType::Pdf,
            processing_type: DocumentProcessingType::TextExtraction,
        };
        assert_eq!(format!("{}", task_type), "DocumentProcessing(Pdf, TextExtraction)");
    }
    
    #[test]
    fn test_worker_type_display() {
        let worker_type = WorkerType::DocumentProcessor {
            supported_types: vec![DocumentType::Pdf],
        };
        assert_eq!(format!("{}", worker_type), "DocumentProcessor([Pdf])");
    }
    
    #[test]
    fn test_task_priority_display() {
        assert_eq!(format!("{}", TaskPriority::Low), "Low");
        assert_eq!(format!("{}", TaskPriority::High), "High");
        assert_eq!(format!("{}", TaskPriority::Critical), "Critical");
    }
    
    #[test]
    fn test_task_status_display() {
        assert_eq!(format!("{}", TaskStatus::Pending), "Pending");
        assert_eq!(format!("{}", TaskStatus::Completed), "Completed");
        assert_eq!(format!("{}", TaskStatus::Failed), "Failed");
    }
    
    #[test]
    fn test_worker_status_display() {
        assert_eq!(format!("{}", WorkerStatus::Running), "Running");
        assert_eq!(format!("{}", WorkerStatus::Busy), "Busy");
        assert_eq!(format!("{}", WorkerStatus::Error("test".to_string())), "Error(test)");
    }
    
    #[test]
    fn test_document_type_display() {
        assert_eq!(format!("{}", DocumentType::Pdf), "PDF");
        assert_eq!(format!("{}", DocumentType::Text), "Text");
        assert_eq!(format!("{}", DocumentType::Word), "Word");
    }
}
