//! Core Types and Data Structures
//!
//! This module defines all the fundamental types used throughout the swarm system.
//! These types are designed to be serializable, type-safe, and efficient.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// TASK TYPES
// ============================================================================

/// Represents a unit of work in the swarm system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub payload: TaskPayload,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Different types of tasks that can be processed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Document processing tasks
    DocumentProcessing {
        document_type: DocumentType,
        processing_type: DocumentProcessingType,
    },
    /// Text analysis tasks
    TextAnalysis {
        analysis_type: TextAnalysisType,
    },
    /// Vector indexing tasks
    VectorIndexing {
        index_type: VectorIndexType,
    },
    /// Custom task types
    Custom {
        name: String,
        version: String,
    },
}

/// Types of document processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocumentProcessingType {
    TextExtraction,
    MetadataExtraction,
    LanguageDetection,
    KeywordExtraction,
    SentimentAnalysis,
    Classification,
    VectorEmbedding,
}

/// Types of text analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TextAnalysisType {
    LanguageDetection,
    KeywordExtraction,
    SentimentAnalysis,
    NamedEntityRecognition,
    TopicModeling,
    Summarization,
}

/// Types of vector indexing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VectorIndexType {
    DenseEmbedding,
    SparseEmbedding,
    HybridEmbedding,
    SemanticSearch,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

/// Task payload containing the actual work data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPayload {
    /// Document processing payload
    Document {
        document: Document,
        processing_options: DocumentProcessingOptions,
    },
    /// Text analysis payload
    Text {
        content: String,
        analysis_options: TextAnalysisOptions,
    },
    /// Vector indexing payload
    Vector {
        vectors: Vec<Vec<f32>>,
        metadata: HashMap<String, serde_json::Value>,
    },
    /// Custom payload
    Custom {
        data: Vec<u8>,
        format: String,
    },
}

/// Document processing options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentProcessingOptions {
    pub extract_text: bool,
    pub extract_metadata: bool,
    pub detect_language: bool,
    pub extract_keywords: bool,
    pub generate_embeddings: bool,
    pub preserve_formatting: bool,
}

impl Default for DocumentProcessingOptions {
    fn default() -> Self {
        Self {
            extract_text: true,
            extract_metadata: true,
            detect_language: true,
            extract_keywords: true,
            generate_embeddings: false,
            preserve_formatting: false,
        }
    }
}

/// Text analysis options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextAnalysisOptions {
    pub language: Option<String>,
    pub max_keywords: usize,
    pub min_keyword_length: usize,
    pub sentiment_threshold: f32,
}

impl Default for TextAnalysisOptions {
    fn default() -> Self {
        Self {
            language: None,
            max_keywords: 10,
            min_keyword_length: 3,
            sentiment_threshold: 0.1,
        }
    }
}

/// Result of task processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub result: Option<TaskResultData>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
    pub completed_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task result data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskResultData {
    /// Document processing result
    DocumentProcessing(DocumentProcessingResult),
    /// Text analysis result
    TextAnalysis(TextAnalysisResult),
    /// Vector indexing result
    VectorIndexing(VectorIndexingResult),
    /// Custom result
    Custom {
        data: Vec<u8>,
        format: String,
    },
}

// ============================================================================
// WORKER TYPES
// ============================================================================

/// Worker configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerConfig {
    pub id: Uuid,
    pub name: String,
    pub worker_type: WorkerType,
    pub max_concurrent_tasks: usize,
    pub capabilities: Vec<WorkerCapability>,
    pub performance_profile: PerformanceProfile,
    pub health_check_interval_ms: u64,
    pub shutdown_timeout_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Different types of workers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkerType {
    /// Document processing workers
    DocumentProcessor {
        supported_types: Vec<DocumentType>,
    },
    /// Text analysis workers
    TextAnalyzer {
        supported_analyses: Vec<TextAnalysisType>,
    },
    /// Vector indexing workers
    VectorIndexer {
        supported_indexes: Vec<VectorIndexType>,
    },
    /// Custom workers
    Custom {
        name: String,
        version: String,
    },
}

/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerCapability {
    pub name: String,
    pub version: String,
    pub supported_task_types: Vec<TaskType>,
    pub max_concurrent_tasks: usize,
    pub performance_profile: PerformanceProfile,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance profile for workers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceProfile {
    pub avg_processing_time_ms: u64,
    pub memory_usage_mb: u64,
    pub cpu_intensity: f32,
    pub throughput_per_second: f32,
}

/// Worker status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkerStatus {
    Starting,
    Running,
    Busy,
    Idle,
    Error(String),
    Shutdown,
}

/// Worker health information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerHealth {
    pub worker_id: Uuid,
    pub status: WorkerStatus,
    pub current_load: usize,
    pub max_capacity: usize,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub last_heartbeat: DateTime<Utc>,
    pub error_count: u32,
    pub success_count: u32,
}

// ============================================================================
// DOCUMENT TYPES
// ============================================================================

/// Document representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub document_type: DocumentType,
    pub content: DocumentContent,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub size_bytes: usize,
}

/// Document content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentContent {
    /// Text content
    Text(String),
    /// Binary content (base64 encoded)
    Binary(Vec<u8>),
    /// Reference to external storage
    Reference {
        storage_id: String,
        path: String,
        access_token: Option<String>,
    },
}

/// Document types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocumentType {
    Pdf,
    Word,
    Text,
    Html,
    Markdown,
    Excel,
    PowerPoint,
    Image,
    Audio,
    Video,
    Unknown,
}

/// Document processing result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentProcessingResult {
    pub document_id: Uuid,
    pub extracted_text: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub language: Option<String>,
    pub keywords: Vec<String>,
    pub sentiment: Option<f32>,
    pub classification: Option<String>,
    pub embeddings: Option<Vec<f32>>,
    pub processing_time_ms: u64,
    pub processed_at: DateTime<Utc>,
}

/// Text analysis result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextAnalysisResult {
    pub text_id: Uuid,
    pub language: Option<String>,
    pub keywords: Vec<String>,
    pub sentiment: Option<f32>,
    pub entities: Vec<NamedEntity>,
    pub topics: Vec<String>,
    pub summary: Option<String>,
    pub processing_time_ms: u64,
    pub processed_at: DateTime<Utc>,
}

/// Named entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NamedEntity {
    pub text: String,
    pub entity_type: String,
    pub confidence: f32,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Vector indexing result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorIndexingResult {
    pub index_id: Uuid,
    pub vector_count: usize,
    pub index_size_bytes: usize,
    pub index_type: VectorIndexType,
    pub processing_time_ms: u64,
    pub processed_at: DateTime<Utc>,
}

// ============================================================================
// MESSAGING TYPES
// ============================================================================

/// Message for inter-component communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub id: Uuid,
    pub subject: String,
    pub payload: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub ttl_ms: Option<u64>,
}

/// Message broker statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageBrokerStats {
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub active_subscriptions: usize,
    pub queue_depth: usize,
    pub error_count: u64,
}

// ============================================================================
// STATISTICS TYPES
// ============================================================================

/// Coordinator statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinatorStats {
    pub total_workers: usize,
    pub active_workers: usize,
    pub total_tasks_processed: u64,
    pub tasks_per_second: f32,
    pub average_processing_time_ms: f32,
    pub error_rate: f32,
}

/// Task queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskQueueStats {
    pub pending_tasks: usize,
    pub processing_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_wait_time_ms: f32,
    pub queue_depth_by_priority: HashMap<TaskPriority, usize>,
}

/// Document reader statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentReaderStats {
    pub total_documents_read: u64,
    pub documents_per_second: f32,
    pub error_count: u64,
    pub last_read_time: Option<DateTime<Utc>>,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary {
        count: u64,
        sum: f64,
        quantiles: Vec<(f64, f64)>,
    },
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::DocumentProcessing { document_type, processing_type } => {
                write!(f, "DocumentProcessing({:?}, {:?})", document_type, processing_type)
            }
            TaskType::TextAnalysis { analysis_type } => {
                write!(f, "TextAnalysis({:?})", analysis_type)
            }
            TaskType::VectorIndexing { index_type } => {
                write!(f, "VectorIndexing({:?})", index_type)
            }
            TaskType::Custom { name, version } => {
                write!(f, "Custom({}, {})", name, version)
            }
        }
    }
}

impl std::fmt::Display for WorkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerType::DocumentProcessor { supported_types } => {
                write!(f, "DocumentProcessor({:?})", supported_types)
            }
            WorkerType::TextAnalyzer { supported_analyses } => {
                write!(f, "TextAnalyzer({:?})", supported_analyses)
            }
            WorkerType::VectorIndexer { supported_indexes } => {
                write!(f, "VectorIndexer({:?})", supported_indexes)
            }
            WorkerType::Custom { name, version } => {
                write!(f, "Custom({}, {})", name, version)
            }
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Normal => write!(f, "Normal"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Assigned => write!(f, "Assigned"),
            TaskStatus::Processing => write!(f, "Processing"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
            TaskStatus::Retrying => write!(f, "Retrying"),
        }
    }
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

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Pdf => write!(f, "PDF"),
            DocumentType::Word => write!(f, "Word"),
            DocumentType::Text => write!(f, "Text"),
            DocumentType::Html => write!(f, "HTML"),
            DocumentType::Markdown => write!(f, "Markdown"),
            DocumentType::Excel => write!(f, "Excel"),
            DocumentType::PowerPoint => write!(f, "PowerPoint"),
            DocumentType::Image => write!(f, "Image"),
            DocumentType::Audio => write!(f, "Audio"),
            DocumentType::Video => write!(f, "Video"),
            DocumentType::Unknown => write!(f, "Unknown"),
        }
    }
}
