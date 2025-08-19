use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use writemagic_shared::{BufferPool, WorkingMemory, with_working_memory};
use writemagic_ai::{CompletionRequest, Message, MessageRole, ClaudeProvider};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark buffer pool performance vs standard allocation
fn bench_buffer_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_allocation");
    
    let pool = BufferPool::new(1024, 10);
    
    group.bench_function("pooled_buffer", |b| {
        b.iter(|| {
            let mut buffer = pool.acquire();
            let data = black_box(b"Hello, World! This is a test message for benchmarking buffer performance.");
            buffer.as_mut_vec().extend_from_slice(data);
            black_box(buffer.as_slice());
        });
    });
    
    group.bench_function("standard_allocation", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(1024);
            let data = black_box(b"Hello, World! This is a test message for benchmarking buffer performance.");
            buffer.extend_from_slice(data);
            black_box(&buffer);
        });
    });
    
    group.finish();
}

/// Benchmark working memory performance
fn bench_working_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("working_memory");
    
    group.bench_function("with_working_memory", |b| {
        b.iter(|| {
            with_working_memory(|wm| {
                let data = black_box(b"Test data for working memory benchmarking");
                wm.request_buffer.try_extend_from_slice(data).unwrap();
                black_box(&wm.request_buffer);
            })
        });
    });
    
    group.bench_function("standard_allocation", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            let data = black_box(b"Test data for working memory benchmarking");
            buffer.extend_from_slice(data);
            black_box(&buffer);
        });
    });
    
    group.finish();
}

/// Benchmark Arc vs service container patterns
fn bench_service_patterns(c: &mut Criterion) {
    use writemagic_shared::{ServiceContainer, ProviderRegistry};
    
    let mut group = c.benchmark_group("service_patterns");
    
    // Setup Arc pattern
    #[derive(Clone)]
    struct TestService {
        value: u32,
    }
    
    let arc_service = Arc::new(TestService { value: 42 });
    
    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            let service = Arc::clone(&arc_service);
            black_box(service.value);
        });
    });
    
    // Setup service container pattern
    let mut container = ServiceContainer::new();
    container.register(TestService { value: 42 });
    
    group.bench_function("service_container", |b| {
        b.iter(|| {
            let service = container.get::<TestService>().unwrap();
            black_box(service.value);
        });
    });
    
    // Setup provider registry
    let registry = ProviderRegistry::new()
        .with_claude(TestService { value: 1 })
        .with_openai(TestService { value: 2 });
    
    group.bench_function("provider_registry", |b| {
        b.iter(|| {
            let claude = registry.claude().unwrap();
            let openai = registry.openai().unwrap();
            black_box(claude.value + openai.value);
        });
    });
    
    group.finish();
}

/// Benchmark AI completion request parsing and processing
fn bench_ai_request_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ai_request_processing");
    
    // Generate test data of varying sizes
    let sizes = vec![100, 1000, 10000];
    
    for size in sizes {
        let messages = vec![Message {
            role: MessageRole::User,
            content: "a".repeat(size),
        }];
        
        let request = CompletionRequest::new(messages, "claude-3-haiku-20240307".to_string());
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("request_serialization", size),
            &request,
            |b, req| {
                b.iter(|| {
                    let json = serde_json::to_string(req).unwrap();
                    black_box(json);
                });
            },
        );
        
        let json = serde_json::to_string(&request).unwrap();
        group.bench_with_input(
            BenchmarkId::new("request_deserialization", size),
            &json,
            |b, json_str| {
                b.iter(|| {
                    let req: CompletionRequest = serde_json::from_str(json_str).unwrap();
                    black_box(req);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    use writemagic_shared::{WritemagicError, ErrorResponse};
    
    let mut group = c.benchmark_group("error_handling");
    
    group.bench_function("error_creation", |b| {
        b.iter(|| {
            let error = WritemagicError::validation("Test validation error");
            black_box(error);
        });
    });
    
    group.bench_function("error_to_response", |b| {
        b.iter(|| {
            let error = WritemagicError::validation("Test validation error");
            let response = error.to_error_response(Some("req-123".to_string()));
            black_box(response);
        });
    });
    
    group.bench_function("structured_error_serialization", |b| {
        b.iter(|| {
            let error = WritemagicError::validation("Test validation error");
            let response = error.to_error_response(Some("req-123".to_string()));
            let json = serde_json::to_string(&response).unwrap();
            black_box(json);
        });
    });
    
    group.finish();
}

/// Benchmark zero-copy string processing
fn bench_zero_copy_processing(c: &mut Criterion) {
    use writemagic_shared::buffer_pool::zero_copy::process_text_data;
    use std::borrow::Cow;
    
    let mut group = c.benchmark_group("zero_copy_processing");
    
    let text = "This is a test string that doesn't need preprocessing";
    let text_with_escapes = "This is a test\nstring with\ttabs and \"quotes\"";
    
    group.bench_function("cow_no_processing", |b| {
        b.iter(|| {
            let result = process_text_data(Cow::Borrowed(black_box(text)));
            black_box(result);
        });
    });
    
    group.bench_function("cow_with_processing", |b| {
        b.iter(|| {
            let result = process_text_data(Cow::Borrowed(black_box(text_with_escapes)));
            black_box(result);
        });
    });
    
    group.bench_function("string_copy", |b| {
        b.iter(|| {
            let mut owned = black_box(text).to_string();
            owned.push_str(" modified");
            black_box(owned);
        });
    });
    
    group.finish();
}

/// Benchmark concurrent access patterns
fn bench_concurrent_patterns(c: &mut Criterion) {
    use std::sync::{Arc, RwLock};
    use dashmap::DashMap;
    
    let mut group = c.benchmark_group("concurrent_patterns");
    
    let arc_rwlock_data = Arc::new(RwLock::new(std::collections::HashMap::<u32, String>::new()));
    let dashmap_data = Arc::new(DashMap::<u32, String>::new());
    
    // Pre-populate with some data
    {
        let mut map = arc_rwlock_data.write().unwrap();
        for i in 0..100 {
            map.insert(i, format!("value_{}", i));
        }
    }
    
    for i in 0..100 {
        dashmap_data.insert(i, format!("value_{}", i));
    }
    
    group.bench_function("arc_rwlock_read", |b| {
        b.iter(|| {
            let map = arc_rwlock_data.read().unwrap();
            let value = map.get(&black_box(42u32));
            black_box(value);
        });
    });
    
    group.bench_function("dashmap_read", |b| {
        b.iter(|| {
            let value = dashmap_data.get(&black_box(42u32));
            black_box(value);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_buffer_pool,
    bench_working_memory,
    bench_service_patterns,
    bench_ai_request_processing,
    bench_error_handling,
    bench_zero_copy_processing,
    bench_concurrent_patterns
);
criterion_main!(benches);