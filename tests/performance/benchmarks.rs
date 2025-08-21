//! Comprehensive Performance Benchmarks for WriteMagic
//! 
//! This module provides comprehensive performance benchmarking across all core
//! operations with realistic data loads and edge case scenarios.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput, black_box};
use std::sync::Arc;
use tokio::runtime::Runtime;
use bytes::Bytes;

// Import WriteMagic modules for real benchmarking
use writemagic_shared::{Result, WritemagicError, ContentType};
use writemagic_writing::{Document};
// Note: Using mock types since actual services aren't implemented yet
// use writemagic_ai::{AIProvider};

/// Benchmark document creation with varying sizes
pub fn bench_document_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    // TODO: Enable when DocumentService is available
    // let repo = Arc::new(SQLiteDocumentRepository::new_in_memory().unwrap());
    // let service = DocumentService::new(repo.clone());

    let mut group = c.benchmark_group("document_creation");
    
    // Test different document sizes
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("size", size), size, |b, &size| {
            b.iter(|| rt.block_on(async {
                let content = "a".repeat(size);
                let document = Document::new(
                    format!("Benchmark Document {}", size),
                    content,
                    ContentType::PlainText,
                    None,
                );
                // Mock document creation - would use actual service
                black_box(document)
            }));
        });
    }
    group.finish();
}

/// Benchmark document retrieval with different cache scenarios
pub fn bench_document_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let repo = Arc::new(SQLiteDocumentRepository::new_in_memory().unwrap());
    let service = DocumentService::new(repo.clone());

    // Pre-populate with test documents
    rt.block_on(async {
        for i in 0..1000 {
            let document = Document::new(
                format!("Test Document {}", i),
                format!("Content for document {}", i),
                ContentType::PlainText,
                None,
            );
            service.create_document(document).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("document_retrieval");
    
    // Benchmark cold retrieval (first access)
    group.bench_function("cold_retrieval", |b| {
        b.iter(|| rt.block_on(async {
            let doc_id = format!("doc_{}", fastrand::usize(0..1000));
            black_box(service.get_document(&doc_id).await)
        }));
    });

    // Benchmark warm retrieval (cached)
    group.bench_function("warm_retrieval", |b| {
        let popular_doc_id = "doc_42".to_string();
        b.iter(|| rt.block_on(async {
            black_box(service.get_document(&popular_doc_id).await)
        }));
    });

    group.finish();
}

/// Benchmark AI provider operations with realistic scenarios
pub fn bench_ai_completion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let provider = MockAIProvider::new();

    let mut group = c.benchmark_group("ai_completion");
    
    // Test different prompt sizes
    for prompt_size in [100, 500, 1000, 2000].iter() {
        group.throughput(Throughput::Elements(*prompt_size as u64));
        group.bench_with_input(BenchmarkId::new("prompt_tokens", prompt_size), prompt_size, |b, &size| {
            b.iter(|| rt.block_on(async {
                let prompt = "word ".repeat(size / 5); // Approximate 5 chars per token
                let request = AIRequest::new(prompt, 150, 0.7);
                black_box(provider.complete(&request).await)
            }));
        });
    }

    // Test concurrent AI requests
    group.bench_function("concurrent_requests", |b| {
        b.iter(|| rt.block_on(async {
            let requests: Vec<_> = (0..10).map(|i| {
                let provider = provider.clone();
                tokio::spawn(async move {
                    let request = AIRequest::new(
                        format!("Complete this text about topic {}: ", i),
                        100,
                        0.7
                    );
                    provider.complete(&request).await
                })
            }).collect();

            let results = futures::future::join_all(requests).await;
            black_box(results)
        }));
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

/// Benchmark database operations under load
pub fn bench_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let repo = Arc::new(SQLiteDocumentRepository::new_in_memory().unwrap());

    let mut group = c.benchmark_group("database_operations");
    
    // Benchmark bulk inserts
    group.bench_function("bulk_insert", |b| {
        b.iter(|| rt.block_on(async {
            let documents: Vec<_> = (0..100).map(|i| {
                Document::new(
                    format!("Bulk Document {}", i),
                    format!("Content {}", i),
                    ContentType::PlainText,
                    None,
                )
            }).collect();

            for doc in documents {
                repo.save(&doc).await.unwrap();
            }
        }));
    });

    // Benchmark concurrent database access
    group.bench_function("concurrent_access", |b| {
        b.iter(|| rt.block_on(async {
            let tasks: Vec<_> = (0..20).map(|i| {
                let repo = repo.clone();
                tokio::spawn(async move {
                    // Mix of reads and writes
                    if i % 2 == 0 {
                        let doc = Document::new(
                            format!("Concurrent Doc {}", i),
                            format!("Content {}", i),
                            ContentType::PlainText,
                            None,
                        );
                        repo.save(&doc).await
                    } else {
                        repo.find_by_id(&format!("doc_{}", i)).await
                    }
                })
            }).collect();

            let results = futures::future::join_all(tasks).await;
            black_box(results)
        }));
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
                let doc = Document::new(
                    format!("Doc {}", i),
                    format!("Content {}", i),
                    ContentType::PlainText,
                    None,
                );
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
            let doc = Document::new(
                "FFI Test Document".to_string(),
                "Test content for FFI transfer".to_string(),
                ContentType::PlainText,
                None,
            );
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