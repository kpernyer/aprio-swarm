//! Message Subscription Implementation
//!
//! This module provides utilities for managing message subscriptions
//! and handling message streams.

use super::*;
use swarm_core::Message;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::Utc;

/// Message subscription manager
pub struct MessageSubscriptionManager {
    subscriptions: HashMap<String, mpsc::UnboundedSender<Message>>,
    message_handlers: HashMap<String, Box<dyn Fn(Message) -> Result<()> + Send + Sync>>,
}

impl MessageSubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            message_handlers: HashMap::new(),
        }
    }
    
    /// Register a message handler for a subject
    pub fn register_handler<F>(&mut self, subject: &str, handler: F)
    where
        F: Fn(Message) -> Result<()> + Send + Sync + 'static,
    {
        self.message_handlers.insert(subject.to_string(), Box::new(handler));
    }
    
    /// Create a subscription for a subject
    pub fn create_subscription(&mut self, subject: &str) -> mpsc::UnboundedReceiver<Message> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscriptions.insert(subject.to_string(), tx);
        rx
    }
    
    /// Send a message to a subscription
    pub fn send_message(&self, subject: &str, message: Message) -> Result<()> {
        if let Some(sender) = self.subscriptions.get(subject) {
            sender.send(message)
                .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        }
        Ok(())
    }
    
    /// Get all active subscription subjects
    pub fn get_subscription_subjects(&self) -> Vec<String> {
        self.subscriptions.keys().cloned().collect()
    }
    
    /// Remove a subscription
    pub fn remove_subscription(&mut self, subject: &str) {
        self.subscriptions.remove(subject);
        self.message_handlers.remove(subject);
    }
}

impl Default for MessageSubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Message routing configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageRoutingConfig {
    /// Default subject prefix
    pub subject_prefix: String,
    
    /// Document processing subjects
    pub document_subjects: DocumentSubjects,
    
    /// Task processing subjects
    pub task_subjects: TaskSubjects,
    
    /// Worker management subjects
    pub worker_subjects: WorkerSubjects,
}

/// Document-related subjects
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentSubjects {
    /// Incoming documents
    pub incoming: String,
    
    /// Document processing results
    pub results: String,
    
    /// Document errors
    pub errors: String,
}

/// Task-related subjects
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskSubjects {
    /// Task assignments
    pub assignments: String,
    
    /// Task results
    pub results: String,
    
    /// Task status updates
    pub status: String,
}

/// Worker-related subjects
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerSubjects {
    /// Worker registration
    pub registration: String,
    
    /// Worker health checks
    pub health: String,
    
    /// Worker status updates
    pub status: String,
}

impl Default for MessageRoutingConfig {
    fn default() -> Self {
        Self {
            subject_prefix: "swarm".to_string(),
            document_subjects: DocumentSubjects {
                incoming: "swarm.documents.incoming".to_string(),
                results: "swarm.documents.results".to_string(),
                errors: "swarm.documents.errors".to_string(),
            },
            task_subjects: TaskSubjects {
                assignments: "swarm.tasks.assignments".to_string(),
                results: "swarm.tasks.results".to_string(),
                status: "swarm.tasks.status".to_string(),
            },
            worker_subjects: WorkerSubjects {
                registration: "swarm.workers.registration".to_string(),
                health: "swarm.workers.health".to_string(),
                status: "swarm.workers.status".to_string(),
            },
        }
    }
}

/// Message routing utilities
pub struct MessageRouter {
    config: MessageRoutingConfig,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(config: MessageRoutingConfig) -> Self {
        Self { config }
    }
    
    /// Get document incoming subject
    pub fn document_incoming_subject(&self) -> &str {
        &self.config.document_subjects.incoming
    }
    
    /// Get document results subject
    pub fn document_results_subject(&self) -> &str {
        &self.config.document_subjects.results
    }
    
    /// Get document errors subject
    pub fn document_errors_subject(&self) -> &str {
        &self.config.document_subjects.errors
    }
    
    /// Get task assignment subject
    pub fn task_assignment_subject(&self) -> &str {
        &self.config.task_subjects.assignments
    }
    
    /// Get task results subject
    pub fn task_results_subject(&self) -> &str {
        &self.config.task_subjects.results
    }
    
    /// Get task status subject
    pub fn task_status_subject(&self) -> &str {
        &self.config.task_subjects.status
    }
    
    /// Get worker registration subject
    pub fn worker_registration_subject(&self) -> &str {
        &self.config.worker_subjects.registration
    }
    
    /// Get worker health subject
    pub fn worker_health_subject(&self) -> &str {
        &self.config.worker_subjects.health
    }
    
    /// Get worker status subject
    pub fn worker_status_subject(&self) -> &str {
        &self.config.worker_subjects.status
    }
    
    /// Create a subject with prefix
    pub fn create_subject(&self, components: &[&str]) -> String {
        let mut subject = self.config.subject_prefix.clone();
        for component in components {
            subject.push('.');
            subject.push_str(component);
        }
        subject
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_subscription_manager() {
        let mut manager = MessageSubscriptionManager::new();
        
        // Create a subscription
        let mut receiver = manager.create_subscription("test.subject");
        
        // Send a message
        let message = Message {
            id: Uuid::new_v4(),
            subject: "test.subject".to_string(),
            payload: b"test message".to_vec(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
            ttl_ms: None,
        };
        
        manager.send_message("test.subject", message.clone()).unwrap();
        
        // Receive the message
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.subject, message.subject);
        assert_eq!(received.payload, message.payload);
    }
    
    #[test]
    fn test_message_routing_config() {
        let config = MessageRoutingConfig::default();
        assert_eq!(config.subject_prefix, "swarm");
        assert_eq!(config.document_subjects.incoming, "swarm.documents.incoming");
        assert_eq!(config.task_subjects.assignments, "swarm.tasks.assignments");
        assert_eq!(config.worker_subjects.registration, "swarm.workers.registration");
    }
    
    #[test]
    fn test_message_router() {
        let config = MessageRoutingConfig::default();
        let router = MessageRouter::new(config);
        
        assert_eq!(router.document_incoming_subject(), "swarm.documents.incoming");
        assert_eq!(router.task_assignment_subject(), "swarm.tasks.assignments");
        assert_eq!(router.worker_registration_subject(), "swarm.workers.registration");
        
        let custom_subject = router.create_subject(&["custom", "subject"]);
        assert_eq!(custom_subject, "swarm.custom.subject");
    }
}
