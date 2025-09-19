//! Swarm Communications - NATS Messaging Implementation
//!
//! This crate provides concrete implementations of messaging capabilities
//! for the Aprio Swarm system using NATS as the message broker.

use swarm_core::prelude::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::Utc;

// Core modules
pub mod nats_broker;
pub mod message_subscription;
pub mod message_serialization;

// Re-export main components
pub use nats_broker::*;
pub use message_subscription::*;
pub use message_serialization::*;

/// NATS connection configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,
    
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
    
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Enable TLS
    pub enable_tls: bool,
    
    /// TLS certificate path (if TLS enabled)
    pub tls_cert_path: Option<String>,
    
    /// TLS key path (if TLS enabled)
    pub tls_key_path: Option<String>,
    
    /// TLS CA path (if TLS enabled)
    pub tls_ca_path: Option<String>,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            connection_timeout_ms: 5000,
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 1000,
            max_message_size: 1024 * 1024, // 1MB
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_path: None,
        }
    }
}

/// NATS connection statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NatsStats {
    /// Total messages sent
    pub messages_sent: u64,
    
    /// Total messages received
    pub messages_received: u64,
    
    /// Active subscriptions
    pub active_subscriptions: usize,
    
    /// Connection status
    pub is_connected: bool,
    
    /// Last connection time
    pub last_connected: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Last disconnection time
    pub last_disconnected: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Reconnection count
    pub reconnection_count: u32,
    
    /// Error count
    pub error_count: u64,
}

impl Default for NatsStats {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            active_subscriptions: 0,
            is_connected: false,
            last_connected: None,
            last_disconnected: None,
            reconnection_count: 0,
            error_count: 0,
        }
    }
}

/// Message serialization error
#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),
    
    #[error("General error: {0}")]
    General(#[from] anyhow::Error),
    
    #[error("Connection error: {message}")]
    Connection { message: String },
    
    #[error("Subscription error: {message}")]
    Subscription { message: String },
    
    #[error("Message too large: {size} bytes (max: {max_size} bytes)")]
    MessageTooLarge { size: usize, max_size: usize },
    
    #[error("Timeout error: {message}")]
    Timeout { message: String },
}

/// Result type for messaging operations
pub type MessageResult<T> = Result<T, MessageError>;
