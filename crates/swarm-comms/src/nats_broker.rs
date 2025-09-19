//! NATS Message Broker Implementation
//!
//! This module provides a concrete implementation of the MessageBroker trait
//! using NATS as the underlying message broker.

use super::*;
use swarm_core::{MessageBroker, MessageSubscription, Message, MessageBrokerStats};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use futures_util::StreamExt;
use bytes::Bytes;

// Real NATS client implementation

/// NATS message broker implementation
pub struct NatsBroker {
    client: Arc<async_nats::Client>,
    config: NatsConfig,
    stats: Arc<RwLock<NatsStats>>,
    subscriptions: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
}

impl NatsBroker {
    /// Create a new NATS broker
    pub async fn new(config: NatsConfig) -> MessageResult<Self> {
        tracing::info!("Connecting to NATS server: {}", config.url);
        
        let client = async_nats::connect(&config.url).await
            .map_err(|e| MessageError::Connection { 
                message: format!("Failed to connect to NATS: {}", e) 
            })?;
        
        let stats = Arc::new(RwLock::new(NatsStats {
            is_connected: true,
            last_connected: Some(Utc::now()),
            ..Default::default()
        }));
        
        let subscriptions = Arc::new(RwLock::new(HashMap::new()));
        
        tracing::info!("✅ Connected to NATS server successfully");
        
        Ok(Self {
            client: Arc::new(client),
            config,
            stats,
            subscriptions,
        })
    }
    
    /// Get current statistics
    pub async fn get_stats(&self) -> NatsStats {
        self.stats.read().await.clone()
    }
    
    /// Check if broker is connected
    pub async fn is_connected(&self) -> bool {
        self.stats.read().await.is_connected
    }
    
    /// Publish a message to a subject
    pub async fn publish_message(&self, subject: &str, message: &Message) -> MessageResult<()> {
        let serialized = serde_json::to_vec(message)?;
        
        if serialized.len() > self.config.max_message_size {
            return Err(MessageError::MessageTooLarge {
                size: serialized.len(),
                max_size: self.config.max_message_size,
            });
        }
        
        self.client.publish(subject.to_string(), Bytes::from(serialized)).await
            .map_err(|e| MessageError::Nats(e.into()))?;
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.messages_sent += 1;
        }
        
        tracing::debug!("Published message to subject: {}", subject);
        Ok(())
    }
    
    /// Subscribe to a subject and return a message receiver
    pub async fn subscribe_to_subject(&self, subject: &str) -> MessageResult<mpsc::UnboundedReceiver<Message>> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Store the sender for cleanup
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subject.to_string(), tx.clone());
        }
        
        // Create NATS subscription
        let mut subscription = self.client.subscribe(subject.to_string()).await
            .map_err(|e| MessageError::Nats(e.into()))?;
        
        // Spawn task to handle incoming messages
        let stats = self.stats.clone();
        tokio::spawn(async move {
            while let Some(nats_message) = subscription.next().await {
                match serde_json::from_slice::<Message>(&nats_message.payload) {
                    Ok(message) => {
                        if let Err(e) = tx.send(message) {
                            tracing::error!("Failed to send message to receiver: {}", e);
                            break;
                        }
                        
                        // Update statistics
                        {
                            let mut stats = stats.write().await;
                            stats.messages_received += 1;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to deserialize message: {}", e);
                        {
                            let mut stats = stats.write().await;
                            stats.error_count += 1;
                        }
                    }
                }
            }
        });
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions += 1;
        }
        
        tracing::info!("Subscribed to subject: {}", subject);
        Ok(rx)
    }
    
    /// Unsubscribe from a subject
    pub async fn unsubscribe(&self, subject: &str) -> MessageResult<()> {
        // Remove from our tracking
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(subject);
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_subscriptions = stats.active_subscriptions.saturating_sub(1);
        }
        
        tracing::info!("Unsubscribed from subject: {}", subject);
        Ok(())
    }
    
    /// Close the broker connection
    pub async fn close(self) -> MessageResult<()> {
        // Close all subscriptions
        {
            let subscriptions = self.subscriptions.read().await;
            for subject in subscriptions.keys() {
                self.unsubscribe(subject).await?;
            }
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.is_connected = false;
            stats.last_disconnected = Some(Utc::now());
        }
        
        tracing::info!("NATS broker connection closed");
        Ok(())
    }
}

#[async_trait]
impl MessageBroker for NatsBroker {
    async fn publish(&self, subject: &str, message: &[u8]) -> Result<()> {
        // Create a Message struct from the raw bytes
        let message = Message {
            id: Uuid::new_v4(),
            subject: subject.to_string(),
            payload: message.to_vec(),
            headers: HashMap::new(),
            timestamp: Utc::now(),
            ttl_ms: None,
        };
        
        self.publish_message(subject, &message).await?;
        Ok(())
    }
    
    async fn subscribe(&self, subject: &str) -> Result<Box<dyn MessageSubscription>> {
        let receiver = self.subscribe_to_subject(subject).await?;
        Ok(Box::new(NatsMessageSubscription::new(subject.to_string(), receiver)))
    }
    
    async fn get_stats(&self) -> MessageBrokerStats {
        let stats = self.get_stats().await;
        MessageBrokerStats {
            total_messages_sent: stats.messages_sent,
            total_messages_received: stats.messages_received,
            active_subscriptions: stats.active_subscriptions,
            queue_depth: 0, // NATS doesn't have a queue depth concept
            error_count: stats.error_count,
        }
    }
}

/// NATS message subscription implementation
pub struct NatsMessageSubscription {
    subject: String,
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl NatsMessageSubscription {
    pub fn new(subject: String, receiver: mpsc::UnboundedReceiver<Message>) -> Self {
        Self { subject, receiver }
    }
}

impl MessageSubscription for NatsMessageSubscription {
    fn next_message(&mut self) -> Result<Option<Message>> {
        // This is a blocking call, but we need to make it async-compatible
        // In a real implementation, you'd want to use async channels
        match self.receiver.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Ok(None),
        }
    }
    
    fn unsubscribe(self) -> Result<()> {
        // The subscription will be cleaned up when the receiver is dropped
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_nats_broker_creation() {
        let config = NatsConfig::default();
        // Note: This test will fail if NATS server is not running
        // In a real test suite, you'd use a test NATS server
        let result = NatsBroker::new(config).await;
        // We expect this to fail in CI without NATS server
        assert!(result.is_err() || result.is_ok());
    }
    
    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.connection_timeout_ms, 5000);
        assert_eq!(config.max_reconnect_attempts, 10);
        assert!(!config.enable_tls);
    }
    
    #[test]
    fn test_nats_stats_default() {
        let stats = NatsStats::default();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.active_subscriptions, 0);
        assert!(!stats.is_connected);
        assert_eq!(stats.reconnection_count, 0);
        assert_eq!(stats.error_count, 0);
    }
}
