//! Document Processing Worker
//!
//! A specialized worker for processing different document types (PDF, Word, etc.)
//! with unit tests to understand the behavior.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocumentType {
    Pdf,
    Word,
    Text,
    Html,
    Markdown,
    Unknown,
}

impl DocumentType {
    /// Determine document type from filename extension
    pub fn from_filename(filename: &str) -> Self {
        let extension = filename
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "pdf" => DocumentType::Pdf,
            "doc" | "docx" => DocumentType::Word,
            "txt" => DocumentType::Text,
            "html" | "htm" => DocumentType::Html,
            "md" | "markdown" => DocumentType::Markdown,
            _ => DocumentType::Unknown,
        }
    }

    /// Get the MIME type for this document type
    pub fn mime_type(&self) -> &'static str {
        match self {
            DocumentType::Pdf => "application/pdf",
            DocumentType::Word => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            DocumentType::Text => "text/plain",
            DocumentType::Html => "text/html",
            DocumentType::Markdown => "text/markdown",
            DocumentType::Unknown => "application/octet-stream",
        }
    }

    /// Get the estimated processing time in milliseconds
    pub fn estimated_processing_time_ms(&self) -> u64 {
        match self {
            DocumentType::Pdf => 200,      // PDF parsing is more complex
            DocumentType::Word => 150,     // Word parsing is moderate
            DocumentType::Text => 50,      // Text is simple
            DocumentType::Html => 100,     // HTML parsing is moderate
            DocumentType::Markdown => 75,  // Markdown is simple
            DocumentType::Unknown => 100,  // Unknown types take default time
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub content: String,
    pub size_bytes: usize,
    pub document_type: DocumentType,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl Document {
    pub fn new(filename: String, content: String) -> Self {
        let document_type = DocumentType::from_filename(&filename);
        let size_bytes = content.len();
        
        Self {
            id: Uuid::new_v4(),
            filename,
            content,
            size_bytes,
            document_type,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingResult {
    pub document_id: Uuid,
    pub processing_time_ms: u64,
    pub extracted_text: String,
    pub word_count: usize,
    pub page_count: Option<usize>,
    pub language: Option<String>,
    pub keywords: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWorker {
    pub id: Uuid,
    pub worker_type: String,
    pub supported_types: Vec<DocumentType>,
    pub max_concurrent_documents: usize,
    pub current_load: usize,
}

impl DocumentWorker {
    pub fn new(worker_type: String, supported_types: Vec<DocumentType>) -> Self {
        Self {
            id: Uuid::new_v4(),
            worker_type,
            supported_types,
            max_concurrent_documents: 10,
            current_load: 0,
        }
    }

    /// Check if this worker can process the given document type
    pub fn can_process(&self, document_type: &DocumentType) -> bool {
        self.supported_types.contains(document_type)
    }

    /// Check if worker has capacity to process more documents
    pub fn has_capacity(&self) -> bool {
        self.current_load < self.max_concurrent_documents
    }

    /// Process a document and return the result
    pub fn process_document(&mut self, document: Document) -> Result<DocumentProcessingResult, String> {
        if !self.can_process(&document.document_type) {
            return Err(format!(
                "Worker {} cannot process document type: {:?}",
                self.worker_type, document.document_type
            ));
        }

        if !self.has_capacity() {
            return Err(format!(
                "Worker {} is at capacity ({} documents)",
                self.worker_type, self.current_load
            ));
        }

        // Simulate processing time based on document type
        let _processing_time = document.document_type.estimated_processing_time_ms();
        
        // Increment current load
        self.current_load += 1;

        // Process the document (simulated)
        let result = self.extract_content(&document)?;
        
        // Decrement current load
        self.current_load -= 1;

        Ok(result)
    }

    /// Extract content from different document types
    fn extract_content(&self, document: &Document) -> Result<DocumentProcessingResult, String> {
        let start_time = std::time::Instant::now();
        
        let (extracted_text, page_count) = match document.document_type {
            DocumentType::Pdf => {
                // Simulate PDF processing
                let text = self.simulate_pdf_extraction(&document.content)?;
                let pages = self.estimate_pdf_pages(&document.content);
                (text, Some(pages))
            }
            DocumentType::Word => {
                // Simulate Word processing
                let text = self.simulate_word_extraction(&document.content)?;
                let pages = self.estimate_word_pages(&document.content);
                (text, Some(pages))
            }
            DocumentType::Text => {
                // Text is already extracted
                (document.content.clone(), None)
            }
            DocumentType::Html => {
                // Simulate HTML processing
                let text = self.simulate_html_extraction(&document.content)?;
                (text, None)
            }
            DocumentType::Markdown => {
                // Simulate Markdown processing
                let text = self.simulate_markdown_extraction(&document.content)?;
                (text, None)
            }
            DocumentType::Unknown => {
                return Err(format!("Cannot process unknown document type: {}", document.filename));
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        let word_count = self.count_words(&extracted_text);
        let keywords = self.extract_keywords(&extracted_text);
        let language = self.detect_language(&extracted_text);

        Ok(DocumentProcessingResult {
            document_id: document.id,
            processing_time_ms: processing_time,
            extracted_text,
            word_count,
            page_count,
            language,
            keywords,
            metadata: document.metadata.clone(),
            processed_at: Utc::now(),
        })
    }

    // Simulated extraction methods for different document types
    fn simulate_pdf_extraction(&self, content: &str) -> Result<String, String> {
        // Simulate PDF parsing - more complex, might fail
        if content.contains("corrupted") {
            return Err("PDF file appears to be corrupted".to_string());
        }
        
        // Simulate extracting text from PDF
        Ok(format!("[PDF EXTRACTED] {}", content))
    }

    fn simulate_word_extraction(&self, content: &str) -> Result<String, String> {
        // Simulate Word parsing - moderate complexity
        if content.contains("password") {
            return Err("Word document is password protected".to_string());
        }
        
        // Simulate extracting text from Word
        Ok(format!("[WORD EXTRACTED] {}", content))
    }

    fn simulate_html_extraction(&self, content: &str) -> Result<String, String> {
        // Simulate HTML parsing - strip tags
        let text = content
            .replace("<", " ")
            .replace(">", " ")
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">");
        
        Ok(text.trim().to_string())
    }

    fn simulate_markdown_extraction(&self, content: &str) -> Result<String, String> {
        // Simulate Markdown parsing - strip markdown syntax
        let text = content
            .replace("#", "")
            .replace("*", "")
            .replace("_", "")
            .replace("`", "")
            .replace("```", "");
        
        Ok(text.trim().to_string())
    }

    fn estimate_pdf_pages(&self, content: &str) -> usize {
        // Simple estimation based on content length
        (content.len() / 2000).max(1)
    }

    fn estimate_word_pages(&self, content: &str) -> usize {
        // Simple estimation based on content length
        (content.len() / 1500).max(1)
    }

    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction - find words that appear multiple times
        let words: Vec<&str> = text
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| w.len() > 2) // Reduced from 3 to 2 to catch "the"
            .collect();
        
        let mut word_counts: HashMap<&str, usize> = HashMap::new();
        for word in &words {
            *word_counts.entry(word).or_insert(0) += 1;
        }
        
        word_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(word, _)| word.to_lowercase())
            .take(10)
            .collect()
    }

    fn detect_language(&self, text: &str) -> Option<String> {
        // Simple language detection based on common words
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("the") && text_lower.contains("and") {
            Some("en".to_string())
        } else if text_lower.contains("der") && text_lower.contains("und") {
            Some("de".to_string())
        } else if text_lower.contains("le") && text_lower.contains("et") {
            Some("fr".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_type_from_filename() {
        assert_eq!(DocumentType::from_filename("document.pdf"), DocumentType::Pdf);
        assert_eq!(DocumentType::from_filename("report.docx"), DocumentType::Word);
        assert_eq!(DocumentType::from_filename("notes.txt"), DocumentType::Text);
        assert_eq!(DocumentType::from_filename("page.html"), DocumentType::Html);
        assert_eq!(DocumentType::from_filename("readme.md"), DocumentType::Markdown);
        assert_eq!(DocumentType::from_filename("unknown.xyz"), DocumentType::Unknown);
    }

    #[test]
    fn test_document_type_mime_types() {
        assert_eq!(DocumentType::Pdf.mime_type(), "application/pdf");
        assert_eq!(DocumentType::Word.mime_type(), "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        assert_eq!(DocumentType::Text.mime_type(), "text/plain");
        assert_eq!(DocumentType::Html.mime_type(), "text/html");
        assert_eq!(DocumentType::Markdown.mime_type(), "text/markdown");
        assert_eq!(DocumentType::Unknown.mime_type(), "application/octet-stream");
    }

    #[test]
    fn test_document_type_processing_times() {
        assert_eq!(DocumentType::Pdf.estimated_processing_time_ms(), 200);
        assert_eq!(DocumentType::Word.estimated_processing_time_ms(), 150);
        assert_eq!(DocumentType::Text.estimated_processing_time_ms(), 50);
        assert_eq!(DocumentType::Html.estimated_processing_time_ms(), 100);
        assert_eq!(DocumentType::Markdown.estimated_processing_time_ms(), 75);
        assert_eq!(DocumentType::Unknown.estimated_processing_time_ms(), 100);
    }

    #[test]
    fn test_document_creation() {
        let doc = Document::new("test.pdf".to_string(), "PDF content".to_string());
        
        assert_eq!(doc.filename, "test.pdf");
        assert_eq!(doc.content, "PDF content");
        assert_eq!(doc.document_type, DocumentType::Pdf);
        assert_eq!(doc.size_bytes, 11);
    }

    #[test]
    fn test_document_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), json!("John Doe"));
        metadata.insert("version".to_string(), json!(1.0));
        
        let doc = Document::new("test.docx".to_string(), "Word content".to_string())
            .with_metadata(metadata.clone());
        
        assert_eq!(doc.metadata, metadata);
        assert_eq!(doc.document_type, DocumentType::Word);
    }

    #[test]
    fn test_worker_creation() {
        let worker = DocumentWorker::new(
            "pdf_processor".to_string(),
            vec![DocumentType::Pdf, DocumentType::Text]
        );
        
        assert_eq!(worker.worker_type, "pdf_processor");
        assert_eq!(worker.supported_types.len(), 2);
        assert!(worker.can_process(&DocumentType::Pdf));
        assert!(worker.can_process(&DocumentType::Text));
        assert!(!worker.can_process(&DocumentType::Word));
    }

    #[test]
    fn test_worker_capacity() {
        let mut worker = DocumentWorker::new(
            "test_worker".to_string(),
            vec![DocumentType::Text]
        );
        
        assert!(worker.has_capacity());
        assert_eq!(worker.current_load, 0);
        
        // Simulate processing documents
        worker.current_load = 5;
        assert!(worker.has_capacity());
        
        worker.current_load = 10;
        assert!(!worker.has_capacity());
    }

    #[test]
    fn test_worker_cannot_process_unsupported_type() {
        let mut worker = DocumentWorker::new(
            "pdf_only".to_string(),
            vec![DocumentType::Pdf]
        );
        
        let doc = Document::new("test.docx".to_string(), "Word content".to_string());
        
        let result = worker.process_document(doc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot process document type"));
    }

    #[test]
    fn test_worker_at_capacity() {
        let mut worker = DocumentWorker::new(
            "limited_worker".to_string(),
            vec![DocumentType::Text]
        );
        
        // Fill up the worker
        worker.current_load = 10;
        
        let doc = Document::new("test.txt".to_string(), "Text content".to_string());
        
        let result = worker.process_document(doc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("is at capacity"));
    }

    #[test]
    fn test_pdf_processing() {
        let mut worker = DocumentWorker::new(
            "pdf_processor".to_string(),
            vec![DocumentType::Pdf]
        );
        
        let doc = Document::new("test.pdf".to_string(), "The PDF content here and the document is ready".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert_eq!(result.extracted_text, "[PDF EXTRACTED] The PDF content here and the document is ready");
        assert_eq!(result.word_count, 11); // "[PDF EXTRACTED] The PDF content here and the document is ready" = 11 words
        assert_eq!(result.page_count, Some(1));
        assert_eq!(result.language, Some("en".to_string()));
    }

    #[test]
    fn test_word_processing() {
        let mut worker = DocumentWorker::new(
            "word_processor".to_string(),
            vec![DocumentType::Word]
        );
        
        let doc = Document::new("test.docx".to_string(), "Word document content".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert_eq!(result.extracted_text, "[WORD EXTRACTED] Word document content");
        assert_eq!(result.word_count, 5); // "[WORD EXTRACTED] Word document content" = 5 words
        assert_eq!(result.page_count, Some(1));
    }

    #[test]
    fn test_text_processing() {
        let mut worker = DocumentWorker::new(
            "text_processor".to_string(),
            vec![DocumentType::Text]
        );
        
        let doc = Document::new("test.txt".to_string(), "Simple text content".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert_eq!(result.extracted_text, "Simple text content");
        assert_eq!(result.word_count, 3);
        assert_eq!(result.page_count, None);
    }

    #[test]
    fn test_html_processing() {
        let mut worker = DocumentWorker::new(
            "html_processor".to_string(),
            vec![DocumentType::Html]
        );
        
        let doc = Document::new("test.html".to_string(), "<h1>Title</h1><p>Content</p>".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert!(result.extracted_text.contains("Title"));
        assert!(result.extracted_text.contains("Content"));
        assert_eq!(result.word_count, 6); // After HTML processing: " h1 Title /h1 p Content /p" = 6 words
        assert_eq!(result.page_count, None);
    }

    #[test]
    fn test_markdown_processing() {
        let mut worker = DocumentWorker::new(
            "markdown_processor".to_string(),
            vec![DocumentType::Markdown]
        );
        
        let doc = Document::new("test.md".to_string(), "# Title\n*Item 1*\n_Item 2_".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert!(result.extracted_text.contains("Title"));
        assert!(result.extracted_text.contains("Item 1"));
        assert!(result.extracted_text.contains("Item 2"));
        assert_eq!(result.word_count, 5); // After markdown processing: " Title Item 1 Item 2" = 5 words
        assert_eq!(result.page_count, None);
    }

    #[test]
    fn test_corrupted_pdf_handling() {
        let mut worker = DocumentWorker::new(
            "pdf_processor".to_string(),
            vec![DocumentType::Pdf]
        );
        
        let doc = Document::new("corrupted.pdf".to_string(), "This is a corrupted PDF file".to_string());
        
        let result = worker.process_document(doc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("corrupted"));
    }

    #[test]
    fn test_password_protected_word_handling() {
        let mut worker = DocumentWorker::new(
            "word_processor".to_string(),
            vec![DocumentType::Word]
        );
        
        let doc = Document::new("protected.docx".to_string(), "This is a password protected document".to_string());
        
        let result = worker.process_document(doc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("password protected"));
    }

    #[test]
    fn test_keyword_extraction() {
        let mut worker = DocumentWorker::new(
            "text_processor".to_string(),
            vec![DocumentType::Text]
        );
        
        let doc = Document::new("test.txt".to_string(), "The quick brown fox jumps over the lazy dog. The fox is very quick.".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert!(result.keywords.contains(&"the".to_string()));
        assert!(result.keywords.contains(&"quick".to_string()));
        assert!(result.keywords.contains(&"fox".to_string()));
    }

    #[test]
    fn test_language_detection() {
        let mut worker = DocumentWorker::new(
            "text_processor".to_string(),
            vec![DocumentType::Text]
        );
        
        let doc = Document::new("test.txt".to_string(), "The quick brown fox jumps over the lazy dog and the fox is very quick".to_string());
        
        let result = worker.process_document(doc).unwrap();
        
        assert_eq!(result.language, Some("en".to_string()));
    }

    #[test]
    fn test_mixed_document_types() {
        let mut worker = DocumentWorker::new(
            "multi_processor".to_string(),
            vec![DocumentType::Pdf, DocumentType::Word, DocumentType::Text]
        );
        
        // Test PDF
        let pdf_doc = Document::new("test.pdf".to_string(), "PDF content".to_string());
        let pdf_result = worker.process_document(pdf_doc).unwrap();
        assert_eq!(pdf_result.extracted_text, "[PDF EXTRACTED] PDF content");
        
        // Test Word
        let word_doc = Document::new("test.docx".to_string(), "Word content".to_string());
        let word_result = worker.process_document(word_doc).unwrap();
        assert_eq!(word_result.extracted_text, "[WORD EXTRACTED] Word content");
        
        // Test Text
        let text_doc = Document::new("test.txt".to_string(), "Text content".to_string());
        let text_result = worker.process_document(text_doc).unwrap();
        assert_eq!(text_result.extracted_text, "Text content");
    }
}
