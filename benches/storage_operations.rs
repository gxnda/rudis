use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rudis::storage::memory::{RedisValue, StorageEngine};

fn bench_set_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_set_string");

    group.bench_function("set_small_value", |b| {
        b.iter_batched(
            || StorageEngine::default(),
            |engine| {
                engine.set(
                    black_box(Bytes::from_static(b"key")),
                    black_box(RedisValue::String(Bytes::from_static(b"value"))),
                    None,
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("set_medium_value_1kb", |b| {
        b.iter_batched(
            || {
                let engine = StorageEngine::default();
                let value = black_box(Bytes::from(vec![b'x'; 1024]));
                (engine, value)
            },
            |(engine, value)| {
                engine.set(
                    black_box(Bytes::from_static(b"key")),
                    black_box(RedisValue::String(value.clone())),
                    None,
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("set_large_value_1mb", |b| {
        b.iter_batched(
            || {
                let engine = StorageEngine::default();
                let value = black_box(Bytes::from(vec![b'x'; 1024 * 1024]));
                (engine, value)
            },
            |(engine, value)| {
                engine.set(
                    black_box(Bytes::from_static(b"key")),
                    black_box(RedisValue::String(value.clone())),
                    None,
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_get_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_get_string");

    group.bench_function("get_existing_key", |b| {
        let engine = StorageEngine::default();
        engine.set(
            Bytes::from_static(b"key"),
            RedisValue::String(Bytes::from_static(b"value")),
            None,
        );

        b.iter(|| {
            let _ = engine.get(black_box(&Bytes::from_static(b"key")));
        });
    });

    group.bench_function("get_nonexistent_key", |b| {
        let engine = StorageEngine::default();
        b.iter(|| {
            let _ = engine.get(black_box(&Bytes::from_static(b"nonexistent")));
        });
    });

    group.finish();
}

fn bench_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_delete");

    group.bench_function("delete_single_key", |b| {
        let engine = StorageEngine::default();
        engine.set(
            Bytes::from_static(b"key"),
            RedisValue::String(Bytes::from_static(b"value")),
            None,
        );

        b.iter(|| {
            engine.del(black_box(&Bytes::from_static(b"key")));
            engine.set(
                Bytes::from_static(b"key"),
                RedisValue::String(Bytes::from_static(b"value")),
                None,
            );
        });
    });

    group.bench_function("delete_multiple_keys_sequential", |b| {
        let engine = StorageEngine::default();
        for i in 0..100 {
            let key = format!("key_{}", i);
            engine.set(
                Bytes::from(key),
                RedisValue::String(Bytes::from_static(b"value")),
                None,
            );
        }

        b.iter(|| {
            for i in 0..100 {
                engine.del(black_box(&Bytes::from(format!("key_{}", i))));
            }
            for i in 0..100 {
                let key = format!("key_{}", i);
                engine.set(
                    Bytes::from(key),
                    RedisValue::String(Bytes::from_static(b"value")),
                    None,
                );
            }
        });
    });

    group.finish();
}

fn bench_exists(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_exists");

    group.bench_function("exists_existing_key", |b| {
        let engine = StorageEngine::default();
        engine.set(
            Bytes::from_static(b"key"),
            RedisValue::String(Bytes::from_static(b"value")),
            None,
        );

        b.iter(|| {
            let _ = engine.exists(black_box(&Bytes::from_static(b"key")));
        });
    });

    group.bench_function("exists_nonexistent_key", |b| {
        let engine = StorageEngine::default();
        b.iter(|| {
            let _ = engine.exists(black_box(&Bytes::from_static(b"nonexistent")));
        });
    });

    group.finish();
}

fn bench_incr_decr(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_incr_decr");

    group.bench_function("incr_existing_integer", |b| {
        let engine = StorageEngine::default();
        engine.set(Bytes::from_static(b"counter"), RedisValue::Integer(0), None);

        b.iter(|| {
            let _ = engine.incr(&black_box(Bytes::from_static(b"counter")));
        });
    });

    group.bench_function("incr_new_key", |b| {
        let engine = StorageEngine::default();
        b.iter(|| {
            let _ = engine.incr(&black_box(Bytes::from_static(b"counter")));
            engine.del(&Bytes::from_static(b"counter"));
        });
    });

    group.bench_function("incr_by_multiple", |b| {
        let engine = StorageEngine::default();
        engine.set(Bytes::from_static(b"counter"), RedisValue::Integer(0), None);

        b.iter(|| {
            let _ = engine.incr_by(&black_box(Bytes::from_static(b"counter")), 100);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_set_string,
    bench_get_string,
    bench_delete,
    bench_exists,
    bench_incr_decr,
);
criterion_main!(benches);
