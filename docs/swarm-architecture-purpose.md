# Aprio Swarm: Architecture and Purpose

## 🎯 **The Ultimate Purpose**

The Aprio Swarm is designed to be a **high-performance, distributed document processing platform** that can handle:

- **10,000+ documents per second** throughput
- **Sub-10ms latency** for simple operations
- **Dynamic scaling** based on workload
- **Mixed document types** (PDF, Word, Text, HTML, Markdown)
- **Real-time processing** with NATS messaging

## 🏗️ **System Architecture: Who Does What**

```
┌─────────────────────────────────────────────────────────────────┐
│                    APRIO SWARM SYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   Document  │    │   Task      │    │   Worker    │        │
│  │   Reader    │───▶│  Generator  │───▶│  Manager    │        │
│  │             │    │             │    │             │        │
│  └─────────────┘    └─────────────┘    └─────────────┘        │
│         │                   │                   │              │
│         ▼                   ▼                   ▼              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   File      │    │   NATS      │    │  Specialized│        │
│  │   System    │    │  Messaging  │    │   Workers   │        │
│  │             │    │             │    │             │        │
│  └─────────────┘    └─────────────┘    └─────────────┘        │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### **1. Document Reader (Entry Point)**
**Purpose**: Monitors directories and feeds documents into the system

**Responsibilities**:
- 📁 **File System Monitoring**: Watches directories for new files
- 📄 **Document Discovery**: Finds and reads document files
- 🔍 **Metadata Extraction**: Extracts file size, modification time, type
- 📡 **NATS Publishing**: Publishes documents to NATS for processing

**Input**: File system directories
**Output**: Document messages on NATS

### **2. Task Generator (Orchestrator)**
**Purpose**: Analyzes documents and creates processing tasks

**Responsibilities**:
- 📋 **Task Creation**: Generates processing tasks based on document type
- 🎯 **Priority Assignment**: Assigns task priorities
- ⏱️ **Resource Estimation**: Estimates processing time and resources
- 📊 **Load Balancing**: Distributes tasks across available workers

**Input**: Document messages from NATS
**Output**: Processing task messages on NATS

### **3. Worker Manager (Resource Manager)**
**Purpose**: Manages worker lifecycle and capabilities

**Responsibilities**:
- 👥 **Worker Startup**: Starts workers with specific capabilities
- 🔧 **Capability Management**: Tracks what each worker can do
- 📊 **Load Monitoring**: Monitors worker utilization and health
- ⚖️ **Load Balancing**: Routes tasks to available workers

**Input**: Worker configuration and health status
**Output**: Worker assignments and health reports

### **4. Specialized Workers (Processing Engines)**
**Purpose**: Actually process documents based on their capabilities

**Worker Types**:
- 📝 **Text Processors**: Handle .txt, .md files
- 📄 **Document Analyzers**: Handle .pdf, .docx files
- 🌐 **Web Processors**: Handle .html files
- 🔍 **Vector Indexers**: Create embeddings for all document types

## 📡 **Document Data Flow: How Information Reaches Workers**

### **The Critical Question: How does a PDF reach a PDF worker?**

Here's the complete flow:

```
1. File System → Document Reader
   ┌─────────────────────────────────────┐
   │ Document Reader reads PDF file      │
   │ - File path: /data/report.pdf       │
   │ - File size: 2.5MB                  │
   │ - Content: [binary PDF data]        │
   │ - Metadata: creation time, etc.     │
   └─────────────────────────────────────┘
                    │
                    ▼
2. Document Reader → NATS
   ┌─────────────────────────────────────┐
   │ NATS Message: "swarm.documents.in"  │
   │ {                                    │
   │   "document_id": "uuid-123",        │
   │   "filename": "report.pdf",         │
   │   "document_type": "Pdf",           │
   │   "content": "[base64-encoded]",    │
   │   "metadata": {                     │
   │     "file_size": 2621440,           │
   │     "file_path": "/data/report.pdf",│
   │     "modified_time": "2024-01-15T10:30:00Z"
   │   }                                 │
   │ }                                    │
   └─────────────────────────────────────┘
                    │
                    ▼
3. Task Generator → NATS
   ┌─────────────────────────────────────┐
   │ NATS Message: "swarm.tasks.pdf"     │
   │ {                                    │
   │   "task_id": "uuid-456",            │
   │   "document_id": "uuid-123",        │
   │   "task_type": "pdf_processing",    │
   │   "priority": "high",               │
   │   "estimated_duration_ms": 500,     │
   │   "required_capabilities": ["pdf_parsing"]
   │ }                                    │
   └─────────────────────────────────────┘
                    │
                    ▼
4. Worker Manager → PDF Worker
   ┌─────────────────────────────────────┐
   │ PDF Worker receives:                │
   │ - Task assignment                   │
   │ - Document content (base64)         │
   │ - Processing instructions           │
   │ - Priority level                    │
   └─────────────────────────────────────┘
```

### **Key Design Decisions**

#### **1. Document Content in NATS Messages**
**Question**: Does the entire document come as a stream in a NATS message?

**Answer**: **Yes, but with important considerations**:

```rust
// Document message structure
pub struct DocumentMessage {
    pub document_id: Uuid,
    pub filename: String,
    pub document_type: DocumentType,
    pub content: String,        // Base64-encoded content
    pub metadata: HashMap<String, serde_json::Value>,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
}
```

**For Large Documents**:
- **Small files (< 1MB)**: Full content in NATS message
- **Large files (> 1MB)**: Content reference + streaming
- **Very large files (> 10MB)**: File system reference only

#### **2. Streaming vs. Full Content**

```rust
pub enum DocumentContent {
    // Small documents: full content
    FullContent(String),
    
    // Medium documents: chunked streaming
    Streaming {
        chunk_size: usize,
        total_chunks: usize,
        stream_id: Uuid,
    },
    
    // Large documents: file reference
    FileReference {
        file_path: String,
        access_token: String,
    },
}
```

#### **3. Worker Capability Matching**

```rust
// PDF Worker capabilities
pub struct PdfWorkerCapability {
    pub name: String,
    pub supported_types: vec![DocumentType::Pdf],
    pub max_file_size_mb: 100,
    pub processing_features: vec![
        "text_extraction",
        "page_counting", 
        "metadata_extraction",
        "image_extraction"
    ],
    pub performance_profile: PerformanceProfile {
        avg_processing_time_ms: 200,
        memory_usage_mb: 512,
        cpu_intensity: 0.7,
    },
}
```

## 🚀 **Real-World Use Cases**

### **1. Document Processing Pipeline**
```
Input: 10,000 PDF invoices per hour
Process: Extract text, validate data, store in database
Output: Structured invoice data for accounting system
```

### **2. Content Analysis System**
```
Input: Mixed document types (PDFs, Word docs, emails)
Process: Extract content, detect language, generate keywords
Output: Searchable content index
```

### **3. Legal Document Review**
```
Input: Large PDF contracts and legal documents
Process: Extract clauses, identify key terms, flag risks
Output: Legal analysis reports
```

## 📊 **Performance Characteristics**

### **Throughput Targets**
- **Text files**: 50,000+ documents/second
- **PDF files**: 5,000+ documents/second  
- **Word documents**: 3,000+ documents/second
- **Mixed workload**: 10,000+ documents/second

### **Latency Targets**
- **Simple text extraction**: < 5ms
- **PDF text extraction**: < 50ms
- **Complex document analysis**: < 200ms
- **End-to-end processing**: < 500ms

### **Scalability**
- **Horizontal scaling**: Add more workers as needed
- **Vertical scaling**: Increase worker capacity
- **Auto-scaling**: Scale based on queue depth and processing time
- **Load balancing**: Distribute work across available workers

## 🔧 **Technical Implementation**

### **NATS Message Flow**
```
swarm.documents.incoming     → Document Reader publishes
swarm.tasks.{type}          → Task Generator publishes  
swarm.workers.{id}.tasks    → Worker Manager routes
swarm.results.{task_id}     → Workers publish results
swarm.monitoring.health     → Health and metrics
```

### **Worker Communication**
```rust
// Worker receives task
pub struct WorkerTask {
    pub task_id: Uuid,
    pub document_id: Uuid,
    pub task_type: String,
    pub content: DocumentContent,
    pub priority: TaskPriority,
    pub deadline: Option<DateTime<Utc>>,
}

// Worker publishes result
pub struct WorkerResult {
    pub task_id: Uuid,
    pub document_id: Uuid,
    pub status: TaskStatus,
    pub result: Option<DocumentProcessingResult>,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}
```

## 🎯 **The Bottom Line**

**The Aprio Swarm exists to solve the problem of processing massive volumes of documents efficiently and reliably.**

**Key Benefits**:
1. **Scalability**: Handle any volume of documents
2. **Flexibility**: Process any document type
3. **Reliability**: Fault-tolerant distributed processing
4. **Performance**: Sub-second processing times
5. **Cost-effectiveness**: Pay only for what you use

**The system is designed to be the backbone of any application that needs to process documents at scale, from simple text extraction to complex document analysis and AI-powered content understanding.**
