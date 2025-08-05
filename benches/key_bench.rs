use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, SamplingMode};
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use rudis::storage::memory::{RedisValue, StorageEngine};
use std::time::{Duration, Instant};

lazy_static! {
    static ref PATTERN: Regex = Regex::new("user_\\d{4}").unwrap();
}

fn setup_storage_engine(size: usize, expiration_ratio: f64) -> StorageEngine {
    let mut rng = rand::thread_rng();
    let storage = StorageEngine::default();

    for i in 0..size {
        let key = format!("key_{:08x}", i).into_bytes().into();
        let value = RedisValue::String(format!("value_{}", i).into_bytes().into());

        let expiry = if rng.gen_bool(expiration_ratio) {
            Some(Instant::now() - Duration::from_secs(3600)) // Expired
        } else {
            None
        };

        storage.set(key, value, expiry);
    }

    // Add special pattern keys
    for i in 0..(size / 10) {
        let key = format!("user_{:04}", i).into_bytes().into();
        let value = RedisValue::String("user_data".into());
        storage.set(key, value, None);
    }

    storage
}

fn bench_matching_keys(c: &mut Criterion) {
    let sizes = [1_000, 10_000, 100_000];
    let expiration_ratios = [0.0, 0.3, 0.7];

    let mut group = c.benchmark_group("Key Matching");
    group.sampling_mode(SamplingMode::Flat); // For more consistent results

    for size in sizes {
        for ratio in expiration_ratios {
            let storage = setup_storage_engine(size, ratio);

            group.bench_with_input(
                format!("serial_{}_expired-{}", size, ratio),
                &storage,
                |b, s| {
                    b.iter_batched(
                        || PATTERN.as_str(),
                        |pattern| {
                            let result = s.get_matching_keys(pattern).unwrap();
                            black_box(result);
                        },
                        BatchSize::LargeInput,
                    )
                },
            );

            group.bench_with_input(
                format!("parallel_{}_expired-{}", size, ratio),
                &storage,
                |b, s| {
                    b.iter_batched(
                        || PATTERN.as_str(),
                        |pattern| {
                            let result = s.get_matching_keys_par(pattern).unwrap();
                            black_box(result);
                        },
                        BatchSize::LargeInput,
                    )
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_matching_keys);
criterion_main!(benches);
