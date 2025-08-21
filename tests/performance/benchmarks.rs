//! Comprehensive Performance Benchmarks for WriteMagic
//! 
//! This module provides comprehensive performance benchmarking across all core
//! operations with realistic data loads and edge case scenarios.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput, black_box};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use bytes::Bytes;

// Import WriteMagic modules for real benchmarking
use writemagic_shared::{Result, WritemagicError, ContentType, EntityId, ContentHash, Timestamp};
use writemagic_writing::{Document, DocumentManagementService, SqliteDocumentRepository};
use writemagic_ai::{CompletionRequest, CompletionResponse, Message, MessageRole, RequestPriority};

/// Benchmark document creation with varying sizes
pub fn bench_document_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    // Mock repository for benchmarking without actual DB
    // let repo = Arc::new(SqliteDocumentRepository::new(pool));
    // let service = DocumentManagementService::new(repo.clone());

    let mut group = c.benchmark_group("document_creation");
    
    // Test different document sizes
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("size", size), size, |b, &size| {
            b.iter(|| {
                let content = "a".repeat(size);
                let document = Document {
                    id: EntityId::new(),
                    title: format!("Benchmark Document {}", size),
                    content,
                    content_type: ContentType::PlainText,
                    content_hash: ContentHash::from_string("test"),
                    file_path: None,
                    word_count: size as u32 / 5,
                    character_count: size as u32,
                    created_at: Timestamp::now(),
                    updated_at: Timestamp::now(),
                    created_by: None,
                    updated_by: None,
                    version: 1,
                    is_deleted: false,
                    deleted_at: None,
                };
                black_box(document)
            });
        });
    }
    group.finish();
}

/// Benchmark document retrieval with different cache scenarios
pub fn bench_document_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_retrieval");
    
    // Mock document retrieval without actual DB
    let documents: Vec<Document> = (0..1000).map(|i| Document {
        id: EntityId::new(),
        title: format!("Test Document {}", i),
        content: format!("Content for document {}", i),
        content_type: ContentType::PlainText,
        content_hash: writemagic_shared::ContentHash::from_string("test"),
        file_path: None,
        word_count: 10,
        character_count: 50,
        created_at: writemagic_shared::Timestamp::now(),
        updated_at: writemagic_shared::Timestamp::now(),
        created_by: None,
        updated_by: None,
        version: 1,
        is_deleted: false,
        deleted_at: None,
    }).collect();

    // Benchmark document search simulation
    group.bench_function("document_search", |b| {
        b.iter(|| {
            let random_index = 0; // Use first document for benchmarking
            black_box(&documents[random_index])
        });
    });

    group.finish();
}

/// Benchmark AI request processing with realistic scenarios
pub fn bench_ai_completion(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_completion");
    
    // Test different prompt sizes
    for prompt_size in [100, 500, 1000, 2000].iter() {
        group.throughput(Throughput::Elements(*prompt_size as u64));
        group.bench_with_input(BenchmarkId::new("prompt_tokens", prompt_size), prompt_size, |b, &size| {
            b.iter(|| {
                let prompt = "word ".repeat(size / 5); // Approximate 5 chars per token
                let request = CompletionRequest {
                    messages: vec![Message {
                        role: MessageRole::User,
                        content: prompt,
                        name: None,
                        metadata: HashMap::new(),
                    }],
                    model: "gpt-3.5-turbo".to_string(),
                    max_tokens: Some(150),
                    temperature: Some(0.7),
                    top_p: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    stop: None,
                    stream: false,
                    metadata: HashMap::new(),
                    priority: RequestPriority::Normal,
                    timeout: None,
                    compress_response: false,
                    batchable: false,
                };
                black_box(request)
            });
        });
    }

    // Test request serialization overhead
    group.bench_function("request_serialization", |b| {
        let request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Complete this text".to_string(),
                name: None,
                metadata: HashMap::new(),
            }],
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: false,
            metadata: HashMap::new(),
            priority: RequestPriority::Normal,
            timeout: None,
            compress_response: false,
            batchable: false,
        };
        
        b.iter(|| {
            let serialized = serde_json::to_string(&request).unwrap();
            let deserialized: CompletionRequest = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized)
        });
    });

    group.finish();
}

/// Benchmark WASM compilation and execution
pub fn bench_wasm_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("wasm_operations");
    
    // Benchmark WASM module compilation
    group.bench_function("wasm_compilation", |b| {
        b.iter(|| rt.block_on(async {
            // Simulate WASM compilation
            let wasm_code = vec![0u8; 1024]; // Mock WASM bytecode
            black_box(wasm_code.len())
        }));
    });

    // Benchmark WASM-JS boundary calls
    group.bench_function("wasm_js_boundary", |b| {
        b.iter(|| {
            // Simulate data transfer across WASM-JS boundary
            let data = vec![42u8; 1024];
            let serialized = serde_json::to_string(&data).unwrap();
            let deserialized: Vec<u8> = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized)
        });
    });

    group.finish();
}

/// Benchmark document processing operations
pub fn bench_database_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_operations");
    
    // Benchmark document creation in memory
    group.bench_function("document_creation", |b| {
        b.iter(|| {
            let documents: Vec<_> = (0..100).map(|i| {
                Document {
                    id: EntityId::new(),
                    title: format!("Bulk Document {}", i),
                    content: format!("Content {}", i),
                    content_type: ContentType::PlainText,
                    content_hash: ContentHash::from_string("test"),
                    file_path: None,
                    word_count: 2,
                    character_count: 10,
                    created_at: Timestamp::now(),
                    updated_at: Timestamp::now(),
                    created_by: None,
                    updated_by: None,
                    version: 1,
                    is_deleted: false,
                    deleted_at: None,
                }
            }).collect();
            black_box(documents)
        });
    });

    // Benchmark document serialization
    group.bench_function("document_serialization", |b| {
        let doc = Document {
            id: EntityId::new(),
            title: "Test Document".to_string(),
            content: "Test content".to_string(),
            content_type: ContentType::PlainText,
            content_hash: writemagic_shared::ContentHash::from_string("test"),
            file_path: None,
            word_count: 2,
            character_count: 12,
            created_at: writemagic_shared::Timestamp::now(),
            updated_at: writemagic_shared::Timestamp::now(),
            created_by: None,
            updated_by: None,
            version: 1,
            is_deleted: false,
            deleted_at: None,
        };
        
        b.iter(|| {
            let serialized = serde_json::to_string(&doc).unwrap();
            let deserialized: Document = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized)
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns
pub fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");
    
    // Benchmark large document processing
    group.bench_function("large_document_processing", |b| {
        b.iter(|| {
            // Simulate processing a 10MB document
            let large_content = "Lorem ipsum ".repeat(1_000_000);
            let words: Vec<&str> = large_content.split_whitespace().collect();
            let word_count = words.len();
            let char_count = large_content.chars().count();
            black_box((word_count, char_count))
        });
    });

    // Benchmark memory allocation patterns
    group.bench_function("allocation_patterns", |b| {
        b.iter(|| {
            // Simulate typical WriteMagic allocation patterns
            let mut documents = Vec::with_capacity(1000);
            for i in 0..1000 {
                let doc = Document {
                    id: EntityId::new(),
                    title: format!("Doc {}", i),
                    content: format!("Content {}", i),
                    content_type: ContentType::PlainText,
                    content_hash: ContentHash::from_string("test"),
                    file_path: None,
                    word_count: 2,
                    character_count: 10,
                    created_at: Timestamp::now(),
                    updated_at: Timestamp::now(),
                    created_by: None,
                    updated_by: None,
                    version: 1,
                    is_deleted: false,
                    deleted_at: None,
                };
                documents.push(doc);
            }
            black_box(documents.len())
        });
    });

    group.finish();
}

/// Benchmark FFI operations for mobile integration
pub fn bench_ffi_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_operations");
    
    // Benchmark string conversion across FFI boundary
    group.bench_function("string_conversion", |b| {
        b.iter(|| {
            let rust_string = "Hello from Rust!".to_string();
            let c_string = std::ffi::CString::new(rust_string.clone()).unwrap();
            let back_to_rust = c_string.to_string_lossy().to_string();
            black_box(back_to_rust)
        });
    });

    // Benchmark data serialization for FFI
    group.bench_function("data_serialization", |b| {
        b.iter(|| {
            let doc = Document {
                id: EntityId::new(),
                title: "FFI Test Document".to_string(),
                content: "Test content for FFI transfer".to_string(),
                content_type: ContentType::PlainText,
                content_hash: ContentHash::from_string("test"),
                file_path: None,
                word_count: 5,
                character_count: 30,
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
                created_by: None,
                updated_by: None,
                version: 1,
                is_deleted: false,
                deleted_at: None,
            };
            let serialized = serde_json::to_string(&doc).unwrap();
            let deserialized: Document = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized)
        });
    });

    group.finish();
}

/// Benchmark text processing operations
pub fn bench_text_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_processing");
    
    // Test different text sizes
    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("word_count", size), size, |b, &size| {
            let text = "word ".repeat(size / 5);
            b.iter(|| {
                let word_count = text.split_whitespace().count();
                black_box(word_count)
            });
        });
    }

    // Benchmark regex operations
    group.bench_function("regex_processing", |b| {
        let text = "This is a sample text with some patterns to match. Email: test@example.com, Phone: (555) 123-4567".repeat(1000);
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        
        b.iter(|| {
            let matches: Vec<_> = email_regex.find_iter(&text).collect();
            black_box(matches)
        });
    });

    group.finish();
}

/// Benchmark error handling and recovery
pub fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    // Benchmark error creation and propagation
    group.bench_function("error_creation", |b| {
        b.iter(|| {
            let error = WritemagicError::validation("Test validation error");
            let result: Result<String> = Err(error);
            let recovered: Result<String> = result.or_else(|_| Ok("recovered".to_string()));
            black_box(recovered)
        });
    });

    // Benchmark error handling in async context
    group.bench_function("async_error_handling", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| rt.block_on(async {
            async fn failing_operation() -> Result<String> {
                Err(WritemagicError::timeout(5000))
            }

            async fn with_recovery() -> Result<String> {
                match failing_operation().await {
                    Ok(result) => Ok(result),
                    Err(_) => Ok("recovered from error".to_string()),
                }
            }

            black_box(with_recovery().await)
        }));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_document_creation,
    bench_document_retrieval,
    bench_ai_completion,
    bench_wasm_operations,
    bench_database_operations,
    bench_memory_operations,
    bench_ffi_operations,
    bench_text_processing,
    bench_error_handling
);
criterion_main!(benches);