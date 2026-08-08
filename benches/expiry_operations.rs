use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rudis::storage::memory::{RedisValue, StorageEngine};
use std::time::Duration;

fn bench_set_with_expiry(c: &mut Criterion) {
    let mut group = c.benchmark_group("expiry_set");

    group.bench_function("set_with_short_expiry_1s", |b| {
        b.iter_batched(
            || StorageEngine::default(),
            |engine| {
                engine.set(
                    black_box(Bytes::from_static(b"key")),
                    black_box(RedisValue::String(Bytes::from_static(b"value"))),
                    black_box(Some(std::time::Instant::now() + Duration::from_secs(1))),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("set_with_long_expiry_1h", |b| {
        b.iter_batched(
            || StorageEngine::default(),
            |engine| {
                engine.set(
                    black_box(Bytes::from_static(b"key")),
                    black_box(RedisValue::String(Bytes::from_static(b"value"))),
                    black_box(Some(std::time::Instant::now() + Duration::from_secs(3600))),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_expire_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("expiry_expire");

    group.bench_function("expire_existing_key", |b| {
        b.iter_batched(
            || {
                let engine = StorageEngine::default();
                engine.set(
                    Bytes::from_static(b"key"),
                    RedisValue::String(Bytes::from_static(b"value")),
                    None,
                );
                engine
            },
            |engine| {
                engine.set_expire_in(
                    black_box(&Bytes::from_static(b"key")),
                    black_box(Duration::from_secs(60)),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("expire_nonexistent_key", |b| {
        b.iter_batched(
            || StorageEngine::default(),
            |engine| {
                engine.set_expire_in(
                    black_box(&Bytes::from_static(b"nonexistent")),
                    black_box(Duration::from_secs(60)),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_ttl_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("expiry_ttl");

    group.bench_function("ttl_key_without_expiry", |b| {
        b.iter_batched(
            || {
                let engine = StorageEngine::default();
                engine.set(
                    Bytes::from_static(b"key"),
                    RedisValue::String(Bytes::from_static(b"value")),
                    None,
                );
                engine
            },
            |engine| {
                let _ = engine.get_expire(black_box(&Bytes::from_static(b"key")));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("ttl_key_with_expiry", |b| {
        b.iter_batched(
            || {
                let engine = StorageEngine::default();
                engine.set(
                    Bytes::from_static(b"key"),
                    RedisValue::String(Bytes::from_static(b"value")),
                    Some(std::time::Instant::now() + Duration::from_secs(60)),
                );
                engine
            },
            |engine| {
                let _ = engine.get_expire(black_box(&Bytes::from_static(b"key")));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_persist_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("expiry_persist");

    group.bench_function("persist_key_with_expiry", |b| {
        b.iter_batched(
            || {
                let engine = StorageEngine::default();
                engine.set(
                    Bytes::from_static(b"key"),
                    RedisValue::String(Bytes::from_static(b"value")),
                    Some(std::time::Instant::now() + Duration::from_secs(60)),
                );
                engine
            },
            |engine| {
                engine.set_expire(black_box(&Bytes::from_static(b"key")), None);
                engine.set(
                    Bytes::from_static(b"key"),
                    RedisValue::String(Bytes::from_static(b"value")),
                    Some(std::time::Instant::now() + Duration::from_secs(60)),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_keys_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("keys_pattern");

    group.bench_function("keys_match_simple_pattern_10_keys", |b| {
        let engine = StorageEngine::default();
        for i in 0..10 {
            engine.set(
                Bytes::from(format!("key_{}", i)),
                RedisValue::String(Bytes::from_static(b"value")),
                None,
            );
        }

        b.iter(|| {
            let _ = engine.get_matching_keys(black_box("key_*"));
        });
    });

    group.bench_function("keys_match_pattern_1000_keys", |b| {
        let engine = StorageEngine::default();
        for i in 0..1000 {
            engine.set(
                Bytes::from(format!("key_{}", i)),
                RedisValue::String(Bytes::from_static(b"value")),
                None,
            );
        }

        b.iter(|| {
            let _ = engine.get_matching_keys(black_box("key_*"));
        });
    });

    group.bench_function("keys_match_complex_pattern", |b| {
        let engine = StorageEngine::default();
        for i in 0..100 {
            engine.set(
                Bytes::from(format!("user:{}:name", i)),
                RedisValue::String(Bytes::from_static(b"value")),
                None,
            );
        }

        b.iter(|| {
            let _ = engine.get_matching_keys(black_box("user:*:name"));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_set_with_expiry,
    bench_expire_command,
    bench_ttl_command,
    bench_persist_command,
    bench_keys_pattern_matching,
);
criterion_main!(benches);
