use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rudis::storage::memory::RedisValue;
use std::collections::VecDeque;

fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_operations");

    group.bench_function("create_list_with_10_elements", |b| {
        b.iter(|| {
            let mut list = VecDeque::new();
            for i in 0..10 {
                list.push_back(Bytes::from(format!("elem_{}", i)));
            }
            let _ = RedisValue::List(black_box(list));
        });
    });

    group.bench_function("push_to_existing_list", |b| {
        b.iter_batched(
            || {
                let mut list = VecDeque::new();
                for i in 0..100 {
                    list.push_back(Bytes::from(format!("elem_{}", i)));
                }
                list
            },
            |mut list| {
                list.push_back(black_box(Bytes::from_static(b"new_elem")));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("pop_from_list_100_elements", |b| {
        b.iter_batched(
            || {
                let mut list = VecDeque::new();
                for i in 0..100 {
                    list.push_back(Bytes::from(format!("elem_{}", i)));
                }
                list
            },
            |mut list| {
                let _ = list.pop_front();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("list_length_check", |b| {
        b.iter_batched(
            || {
                let mut list = VecDeque::new();
                for i in 0..1000 {
                    list.push_back(Bytes::from(format!("elem_{}", i)));
                }
                list
            },
            |list| {
                let _ = black_box(&list).len();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_hash_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_operations");

    group.bench_function("create_hash_with_10_fields", |b| {
        b.iter(|| {
            use dashmap::DashMap;
            let hash: DashMap<Bytes, Bytes> = DashMap::new();
            for i in 0..10 {
                hash.insert(
                    Bytes::from(format!("field_{}", i)),
                    Bytes::from(format!("value_{}", i)),
                );
            }
            let _ = black_box(hash);
        });
    });

    group.bench_function("hash_insert_single_field", |b| {
        b.iter_batched(
            || {
                use dashmap::DashMap;
                DashMap::new()
            },
            |hash| {
                hash.insert(
                    black_box(Bytes::from_static(b"field")),
                    black_box(Bytes::from_static(b"value")),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("hash_get_field", |b| {
        b.iter_batched(
            || {
                use dashmap::DashMap;
                let hash: DashMap<Bytes, Bytes> = DashMap::new();
                hash.insert(Bytes::from_static(b"field"), Bytes::from_static(b"value"));
                hash
            },
            |hash| {
                let _ = hash.get(black_box(&Bytes::from_static(b"field")));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("hash_insert_100_fields", |b| {
        b.iter_batched(
            || {
                use dashmap::DashMap;
                DashMap::new()
            },
            |hash| {
                for i in 0..100 {
                    hash.insert(
                        black_box(Bytes::from(format!("field_{}", i))),
                        Bytes::from(format!("value_{}", i)),
                    );
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_set_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_operations");

    group.bench_function("create_set_with_10_members", |b| {
        b.iter(|| {
            use dashmap::DashSet;
            let set: DashSet<Bytes> = DashSet::new();
            for i in 0..10 {
                set.insert(Bytes::from(format!("member_{}", i)));
            }
            let _ = black_box(set);
        });
    });

    group.bench_function("set_insert_single_member", |b| {
        b.iter_batched(
            || {
                use dashmap::DashSet;
                DashSet::new()
            },
            |set| {
                set.insert(black_box(Bytes::from_static(b"member")));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("set_contains_check", |b| {
        b.iter_batched(
            || {
                use dashmap::DashSet;
                let set: DashSet<Bytes> = DashSet::new();
                set.insert(Bytes::from_static(b"member"));
                set
            },
            |set| {
                let _ = set.contains(black_box(&Bytes::from_static(b"member")));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("set_insert_100_members", |b| {
        b.iter_batched(
            || {
                use dashmap::DashSet;
                DashSet::new()
            },
            |set| {
                for i in 0..100 {
                    set.insert(black_box(Bytes::from(format!("member_{}", i))));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_list_operations,
    bench_hash_operations,
    bench_set_operations,
);
criterion_main!(benches);
