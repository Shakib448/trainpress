use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde::{Deserialize, Serialize};
use trainpress::response::Json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallPayload {
    id: u64,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediumPayload {
    id: u64,
    name: String,
    email: String,
    age: u32,
    active: bool,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LargePayload {
    id: u64,
    name: String,
    email: String,
    age: u32,
    active: bool,
    tags: Vec<String>,
    metadata: Vec<KeyValue>,
    nested: NestedData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NestedData {
    field1: String,
    field2: i32,
    field3: Vec<String>,
}

fn json_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("json");

    // Benchmark: Small payload serialization
    group.bench_function("small_payload_serialize", |b| {
        let payload = SmallPayload {
            id: 1,
            name: "John Doe".to_string(),
        };

        b.iter(|| {
            let json = Json(black_box(payload.clone()));
            black_box(serde_json::to_string(&json.0).unwrap())
        });
    });

    // Benchmark: Medium payload serialization
    group.bench_function("medium_payload_serialize", |b| {
        let payload = MediumPayload {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            active: true,
            tags: vec!["rust".to_string(), "web".to_string(), "backend".to_string()],
        };

        b.iter(|| {
            let json = Json(black_box(payload.clone()));
            black_box(serde_json::to_string(&json.0).unwrap())
        });
    });

    // Benchmark: Large payload serialization
    group.bench_function("large_payload_serialize", |b| {
        let payload = LargePayload {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            active: true,
            tags: vec!["rust".to_string(), "web".to_string(), "backend".to_string()],
            metadata: vec![
                KeyValue { key: "key1".to_string(), value: "value1".to_string() },
                KeyValue { key: "key2".to_string(), value: "value2".to_string() },
                KeyValue { key: "key3".to_string(), value: "value3".to_string() },
            ],
            nested: NestedData {
                field1: "nested value".to_string(),
                field2: 42,
                field3: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            },
        };

        b.iter(|| {
            let json = Json(black_box(payload.clone()));
            black_box(serde_json::to_string(&json.0).unwrap())
        });
    });

    // Benchmark: Small payload deserialization
    group.bench_function("small_payload_deserialize", |b| {
        let json_str = r#"{"id":1,"name":"John Doe"}"#;

        b.iter(|| {
            black_box(serde_json::from_str::<SmallPayload>(black_box(json_str)).unwrap())
        });
    });

    // Benchmark: Medium payload deserialization
    group.bench_function("medium_payload_deserialize", |b| {
        let json_str = r#"{"id":1,"name":"John Doe","email":"john@example.com","age":30,"active":true,"tags":["rust","web","backend"]}"#;

        b.iter(|| {
            black_box(serde_json::from_str::<MediumPayload>(black_box(json_str)).unwrap())
        });
    });

    // Benchmark: Large payload deserialization
    group.bench_function("large_payload_deserialize", |b| {
        let json_str = r#"{"id":1,"name":"John Doe","email":"john@example.com","age":30,"active":true,"tags":["rust","web","backend"],"metadata":[{"key":"key1","value":"value1"},{"key":"key2","value":"value2"},{"key":"key3","value":"value3"}],"nested":{"field1":"nested value","field2":42,"field3":["a","b","c"]}}"#;

        b.iter(|| {
            black_box(serde_json::from_str::<LargePayload>(black_box(json_str)).unwrap())
        });
    });

    // Benchmark: Array serialization with different sizes
    for size in &[10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("array_serialize", size),
            size,
            |b, &size| {
                let items: Vec<SmallPayload> = (0..size)
                    .map(|i| SmallPayload {
                        id: i,
                        name: format!("User {}", i),
                    })
                    .collect();

                b.iter(|| {
                    black_box(serde_json::to_string(&black_box(&items)).unwrap())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, json_benchmarks);
criterion_main!(benches);
