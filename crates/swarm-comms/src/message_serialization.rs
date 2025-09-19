//! Message Serialization Utilities
//!
//! This module provides utilities for serializing and deserializing
//! messages for NATS communication.

use super::*;
use swarm_core::{Document, DocumentType, DocumentContent, Task, TaskType, TaskPriority, TaskStatus, TaskPayload, TaskResult, WorkerStatus, Message, DocumentProcessingType, DocumentProcessingOptions};
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Message serialization utilities
pub struct MessageSerializer;

impl MessageSerializer {
    /// Serialize a document to a message
    pub fn serialize_document(document: &Document) -> Result<Message> {
        let payload = serde_json::to_vec(document)?;
        
        Ok(Message {
            id: Uuid::new_v4(),
            subject: "swarm.documents.incoming".to_string(),
            payload,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("document-type".to_string(), format!("{:?}", document.document_type));
                headers.insert("document-id".to_string(), document.id.to_string());
                headers
            },
            timestamp: Utc::now(),
            ttl_ms: Some(300_000), // 5 minutes TTL
        })
    }
    
    /// Deserialize a message to a document
    pub fn deserialize_document(message: &Message) -> Result<Document> {
        let document: Document = serde_json::from_slice(&message.payload)?;
        Ok(document)
    }
    
    /// Serialize a task to a message
    pub fn serialize_task(task: &Task) -> Result<Message> {
        let payload = serde_json::to_vec(task)?;
        
        Ok(Message {
            id: Uuid::new_v4(),
            subject: "swarm.tasks.assignments".to_string(),
            payload,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("task-type".to_string(), format!("{:?}", task.task_type));
                headers.insert("task-id".to_string(), task.id.to_string());
                headers.insert("priority".to_string(), format!("{:?}", task.priority));
                headers
            },
            timestamp: Utc::now(),
            ttl_ms: Some(600_000), // 10 minutes TTL
        })
    }
    
    /// Deserialize a message to a task
    pub fn deserialize_task(message: &Message) -> Result<Task> {
        let task: Task = serde_json::from_slice(&message.payload)?;
        Ok(task)
    }
    
    /// Serialize a task result to a message
    pub fn serialize_task_result(result: &TaskResult) -> Result<Message> {
        let payload = serde_json::to_vec(result)?;
        
        Ok(Message {
            id: Uuid::new_v4(),
            subject: "swarm.tasks.results".to_string(),
            payload,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("task-id".to_string(), result.task_id.to_string());
                headers.insert("status".to_string(), format!("{:?}", result.status));
                headers
            },
            timestamp: Utc::now(),
            ttl_ms: Some(900_000), // 15 minutes TTL
        })
    }
    
    /// Deserialize a message to a task result
    pub fn deserialize_task_result(message: &Message) -> Result<TaskResult> {
        let result: TaskResult = serde_json::from_slice(&message.payload)?;
        Ok(result)
    }
    
    /// Serialize a worker status to a message
    pub fn serialize_worker_status(worker_id: Uuid, status: &WorkerStatus) -> Result<Message> {
        let payload = serde_json::to_vec(status)?;
        
        Ok(Message {
            id: Uuid::new_v4(),
            subject: "swarm.workers.status".to_string(),
            payload,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("worker-id".to_string(), worker_id.to_string());
                headers
            },
            timestamp: Utc::now(),
            ttl_ms: Some(60_000), // 1 minute TTL
        })
    }
    
    /// Deserialize a message to a worker status
    pub fn deserialize_worker_status(message: &Message) -> Result<WorkerStatus> {
        let status: WorkerStatus = serde_json::from_slice(&message.payload)?;
        Ok(status)
    }
    
    /// Create a heartbeat message
    pub fn create_heartbeat(component_id: &str, component_type: &str) -> Result<Message> {
        let heartbeat_data = serde_json::json!({
            "component_id": component_id,
            "component_type": component_type,
            "timestamp": Utc::now(),
            "status": "alive"
        });
        
        let payload = serde_json::to_vec(&heartbeat_data)?;
        
        Ok(Message {
            id: Uuid::new_v4(),
            subject: "swarm.heartbeat".to_string(),
            payload,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("component-id".to_string(), component_id.to_string());
                headers.insert("component-type".to_string(), component_type.to_string());
                headers
            },
            timestamp: Utc::now(),
            ttl_ms: Some(30_000), // 30 seconds TTL
        })
    }
    
    /// Create an error message
    pub fn create_error_message(
        error_type: &str,
        error_message: &str,
        component_id: &str,
        context: Option<HashMap<String, String>>,
    ) -> Result<Message> {
        let error_data = serde_json::json!({
            "error_type": error_type,
            "error_message": error_message,
            "component_id": component_id,
            "context": context,
            "timestamp": Utc::now()
        });
        
        let payload = serde_json::to_vec(&error_data)?;
        
        Ok(Message {
            id: Uuid::new_v4(),
            subject: "swarm.errors".to_string(),
            payload,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("error-type".to_string(), error_type.to_string());
                headers.insert("component-id".to_string(), component_id.to_string());
                headers
            },
            timestamp: Utc::now(),
            ttl_ms: Some(1_800_000), // 30 minutes TTL
        })
    }
}

/// Message validation utilities
pub struct MessageValidator;

impl MessageValidator {
    /// Validate a message structure
    pub fn validate_message(message: &Message) -> Result<()> {
        if message.subject.is_empty() {
            return Err(anyhow::anyhow!("Message subject cannot be empty"));
        }
        
        if message.payload.is_empty() {
            return Err(anyhow::anyhow!("Message payload cannot be empty"));
        }
        
        if message.id.is_nil() {
            return Err(anyhow::anyhow!("Message ID cannot be nil"));
        }
        
        Ok(())
    }
    
    /// Validate message size
    pub fn validate_message_size(message: &Message, max_size: usize) -> Result<()> {
        let total_size = message.payload.len() + 
                        message.subject.len() + 
                        message.headers.values().map(|v| v.len()).sum::<usize>();
        
        if total_size > max_size {
            return Err(anyhow::anyhow!(
                "Message size {} exceeds maximum size {}", 
                total_size, 
                max_size
            ));
        }
        
        Ok(())
    }
    
    /// Validate message TTL
    pub fn validate_message_ttl(message: &Message) -> Result<()> {
        if let Some(ttl_ms) = message.ttl_ms {
            if ttl_ms == 0 {
                return Err(anyhow::anyhow!("Message TTL cannot be zero"));
            }
            
            if ttl_ms > 3_600_000 { // 1 hour
                return Err(anyhow::anyhow!("Message TTL cannot exceed 1 hour"));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serialize_document() {
        let document = Document {
            id: Uuid::new_v4(),
            filename: "test.txt".to_string(),
            document_type: DocumentType::Text,
            content: DocumentContent::Text("test content".to_string()),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            size_bytes: 100,
        };
        
        let message = MessageSerializer::serialize_document(&document).unwrap();
        assert_eq!(message.subject, "swarm.documents.incoming");
        assert!(message.headers.contains_key("document-type"));
        assert!(message.headers.contains_key("document-id"));
        
        let deserialized = MessageSerializer::deserialize_document(&message).unwrap();
        assert_eq!(deserialized.filename, document.filename);
        assert_eq!(deserialized.document_type, document.document_type);
    }
    
    #[test]
    fn test_serialize_task() {
        let task = Task {
            id: Uuid::new_v4(),
            task_type: TaskType::DocumentProcessing {
                document_type: DocumentType::Pdf,
                processing_type: DocumentProcessingType::TextExtraction,
            },
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            payload: TaskPayload::Document {
                document: Document {
                    id: Uuid::new_v4(),
                    filename: "test.pdf".to_string(),
                    document_type: DocumentType::Pdf,
                    content: DocumentContent::Binary(vec![1, 2, 3]),
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
        
        let message = MessageSerializer::serialize_task(&task).unwrap();
        assert_eq!(message.subject, "swarm.tasks.assignments");
        assert!(message.headers.contains_key("task-type"));
        assert!(message.headers.contains_key("task-id"));
        assert!(message.headers.contains_key("priority"));
        
        let deserialized = MessageSerializer::deserialize_task(&message).unwrap();
        assert_eq!(deserialized.id, task.id);
        assert_eq!(deserialized.priority, task.priority);
    }
    
    #[test]
    fn test_create_heartbeat() {
        let message = MessageSerializer::create_heartbeat("worker-1", "document-processor").unwrap();
        assert_eq!(message.subject, "swarm.heartbeat");
        assert!(message.headers.contains_key("component-id"));
        assert!(message.headers.contains_key("component-type"));
        
        let heartbeat_data: serde_json::Value = serde_json::from_slice(&message.payload).unwrap();
        assert_eq!(heartbeat_data["component_id"], "worker-1");
        assert_eq!(heartbeat_data["component_type"], "document-processor");
        assert_eq!(heartbeat_data["status"], "alive");
    }
    
    #[test]
    fn test_create_error_message() {
        let mut context = HashMap::new();
        context.insert("file".to_string(), "test.txt".to_string());
        context.insert("line".to_string(), "42".to_string());
        
        let message = MessageSerializer::create_error_message(
            "processing_error",
            "Failed to process document",
            "worker-1",
            Some(context),
        ).unwrap();
        
        assert_eq!(message.subject, "swarm.errors");
        assert!(message.headers.contains_key("error-type"));
        assert!(message.headers.contains_key("component-id"));
        
        let error_data: serde_json::Value = serde_json::from_slice(&message.payload).unwrap();
        assert_eq!(error_data["error_type"], "processing_error");
        assert_eq!(error_data["error_message"], "Failed to process document");
        assert_eq!(error_data["component_id"], "worker-1");
    }
    
    #[test]
    fn test_validate_message() {
        let valid_message = Message {
            id: Uuid::new_v4(),
            subject: "test.subject".to_string(),
            payload: b"test".to_vec(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
            ttl_ms: None,
        };
        
        assert!(MessageValidator::validate_message(&valid_message).is_ok());
        
        let invalid_message = Message {
            id: Uuid::new_v4(),
            subject: "".to_string(), // Empty subject
            payload: b"test".to_vec(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
            ttl_ms: None,
        };
        
        assert!(MessageValidator::validate_message(&invalid_message).is_err());
    }
    
    #[test]
    fn test_validate_message_size() {
        let message = Message {
            id: Uuid::new_v4(),
            subject: "test.subject".to_string(),
            payload: vec![0; 1000], // 1000 bytes
            headers: HashMap::new(),
            timestamp: Utc::now(),
            ttl_ms: None,
        };
        
        assert!(MessageValidator::validate_message_size(&message, 2000).is_ok());
        assert!(MessageValidator::validate_message_size(&message, 500).is_err());
    }
}
