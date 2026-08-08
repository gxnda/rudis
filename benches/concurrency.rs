use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};
use rudis::storage::memory::{RedisValue, StorageEngine};
use std::sync::Arc;
use std::thread;

fn bench_concurrent_sets_100_threads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency_sets");
    group.sample_size(50);

    group.bench_function("100_concurrent_sets_1000_ops_each", |b| {
        b.iter(|| {
            let engine = Arc::new(StorageEngine::default());
            let mut handles = vec![];

            for thread_id in 0..100 {
                let engine = Arc::clone(&engine);
                let handle = thread::spawn(move || {
                    for i in 0..1000 {
                        let key = format!("thread_{}_{}", thread_id, i);
                        engine.set(
                            Bytes::from(key),
                            RedisValue::String(Bytes::from_static(b"value")),
                            None,
                        );
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.finish();
}

fn bench_concurrent_gets_100_threads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency_gets");
    group.sample_size(50);

    group.bench_function("100_concurrent_gets_1000_ops_each", |b| {
        b.iter_batched(
            || {
                let engine = Arc::new(StorageEngine::default());
                for i in 0..100000 {
                    let key = format!("key_{}", i);
                    engine.set(
                        Bytes::from(key),
                        RedisValue::String(Bytes::from_static(b"value")),
                        None,
                    );
                }
                engine
            },
            |engine| {
                let mut handles = vec![];

                for thread_id in 0..100 {
                    let engine = Arc::clone(&engine);
                    let handle = thread::spawn(move || {
                        for i in 0..1000 {
                            let key = format!("key_{}", (thread_id * 1000 + i) % 100000);
                            let _ = engine.get(&Bytes::from(key));
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    let _ = handle.join();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_concurrent_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency_mixed");
    group.sample_size(50);

    group.bench_function("50_threads_mixed_read_write_200_ops_each", |b| {
        b.iter_batched(
            || {
                let engine = Arc::new(StorageEngine::default());
                for i in 0..10000 {
                    let key = format!("key_{}", i);
                    engine.set(
                        Bytes::from(key),
                        RedisValue::String(Bytes::from_static(b"value")),
                        None,
                    );
                }
                engine
            },
            |engine| {
                let mut handles = vec![];

                for thread_id in 0..50 {
                    let engine = Arc::clone(&engine);
                    let handle = thread::spawn(move || {
                        for i in 0..200 {
                            let key_idx = (thread_id * 200 + i) % 10000;

                            if i % 3 == 0 {
                                let _ = engine.get(&Bytes::from(format!("key_{}", key_idx)));
                            } else if i % 3 == 1 {
                                engine.set(
                                    Bytes::from(format!("key_{}", key_idx)),
                                    RedisValue::String(Bytes::from_static(b"newvalue")),
                                    None,
                                );
                            } else {
                                let _ = engine.del(&Bytes::from(format!("key_{}", key_idx)));
                            }
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    let _ = handle.join();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_concurrent_incr_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency_incr");
    group.sample_size(50);

    group.bench_function("50_threads_concurrent_incr_100_ops_each", |b| {
        b.iter_batched(
            || {
                let engine = Arc::new(StorageEngine::default());
                engine.set(Bytes::from_static(b"counter"), RedisValue::Integer(0), None);
                engine
            },
            |engine| {
                let mut handles = vec![];

                for _ in 0..50 {
                    let engine = Arc::clone(&engine);
                    let handle = thread::spawn(move || {
                        for _ in 0..100 {
                            let _ = engine.incr(&Bytes::from_static(b"counter"));
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    let _ = handle.join();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_concurrent_existence_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency_exists");
    group.sample_size(50);

    group.bench_function("100_threads_existence_checks_1000_ops_each", |b| {
        b.iter_batched(
            || {
                let engine = Arc::new(StorageEngine::default());
                for i in 0..10000 {
                    let key = format!("key_{}", i);
                    engine.set(
                        Bytes::from(key),
                        RedisValue::String(Bytes::from_static(b"value")),
                        None,
                    );
                }
                engine
            },
            |engine| {
                let mut handles = vec![];

                for thread_id in 0..100 {
                    let engine = Arc::clone(&engine);
                    let handle = thread::spawn(move || {
                        for i in 0..1000 {
                            let key = format!("key_{}", (thread_id * 1000 + i) % 10000);
                            let _ = engine.exists(&Bytes::from(key));
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    let _ = handle.join();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_concurrent_sets_100_threads,
    bench_concurrent_gets_100_threads,
    bench_concurrent_mixed_workload,
    bench_concurrent_incr_operations,
    bench_concurrent_existence_checks,
);
criterion_main!(benches);
