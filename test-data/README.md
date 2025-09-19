# Document Processing Test Data

This directory contains sample documents for testing the Aprio Swarm document processing system.

## Test Documents

### Text Documents
- `sample.txt` - Basic text document with structured content
- `notes.txt` - Simple notes for testing

### Markdown Documents  
- `README.md` - This file (Markdown documentation)
- `project-spec.md` - Project specification document

### Word Documents
- `report.docx` - Business report document
- `proposal.docx` - Project proposal document

### PDF Documents
- `manual.pdf` - Technical manual document
- `invoice.pdf` - Sample invoice document

## Document Processing Pipeline

1. **Document Reader** scans this directory
2. **File Type Detection** identifies document format
3. **Metadata Extraction** reads file properties
4. **Task Generation** creates processing jobs
5. **Worker Assignment** routes to appropriate workers
6. **Content Processing** extracts and analyzes content
7. **Result Storage** saves processed results

## Expected Processing Results

Each document type should produce:
- **Text**: Direct content extraction, word count, language detection
- **Markdown**: Syntax removal, content extraction, structure analysis
- **Word**: Complex parsing, formatting preservation, metadata extraction
- **PDF**: Advanced parsing, page counting, layout analysis

## Testing Scenarios

- Single document processing
- Batch document processing
- Mixed document type streams
- Error handling (corrupted files, unsupported formats)
- Worker capacity management
- Load balancing across worker types
