use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rudis::RespValue;

fn bench_parse_simple_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("resp_parse_simple_string");

    group.throughput(Throughput::Bytes(10));
    group.bench_function("short_string", |b| {
        b.iter(|| {
            let input = black_box(Bytes::from_static(b"+OK\r\n"));
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.throughput(Throughput::Bytes(100));
    group.bench_function("medium_string", |b| {
        let input = black_box(Bytes::from_static(
            b"+This is a medium length string for testing purposes\r\n",
        ));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.finish();
}

fn bench_parse_bulk_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("resp_parse_bulk_string");

    group.throughput(Throughput::Bytes(20));
    group.bench_function("small_bulk_string", |b| {
        let input = black_box(Bytes::from_static(b"$5\r\nHello\r\n"));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.throughput(Throughput::Bytes(1024));
    group.bench_function("large_bulk_string_1kb", |b| {
        let data = vec![b'x'; 1024];
        let input = format!("${}\r\n", data.len());
        let mut full_input = input.into_bytes();
        full_input.extend_from_slice(&data);
        full_input.extend_from_slice(b"\r\n");
        let full_input_bytes = Bytes::from_iter(full_input);

        b.iter(|| {
            let _ = RespValue::parse_checked(black_box(&full_input_bytes));
        });
    });

    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("large_bulk_string_1mb", |b| {
        let data = vec![b'x'; 1024 * 1024];
        let input = format!("${}\r\n", data.len());
        let mut full_input = input.into_bytes();
        full_input.extend_from_slice(&data);
        full_input.extend_from_slice(b"\r\n");
        let full_input_bytes = Bytes::from_iter(full_input);

        b.iter(|| {
            let _ = RespValue::parse_checked(black_box(&full_input_bytes));
        });
    });

    group.finish();
}

fn bench_parse_integer(c: &mut Criterion) {
    let mut group = c.benchmark_group("resp_parse_integer");

    group.bench_function("small_integer", |b| {
        let input = black_box(Bytes::from_static(b":42\r\n"));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.bench_function("large_integer", |b| {
        let input = black_box(Bytes::from_static(b":9223372036854775807\r\n"));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.finish();
}

fn bench_parse_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("resp_parse_array");

    group.bench_function("small_array_3_elements", |b| {
        let input = black_box(Bytes::from_static(
            b"*3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n",
        ));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.bench_function("medium_array_10_elements", |b| {
        let input = black_box(Bytes::from_static(b"*10\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n$4\r\nitem\r\n"));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.finish();
}

fn bench_parse_error(c: &mut Criterion) {
    let mut group = c.benchmark_group("resp_parse_error");

    group.bench_function("simple_error", |b| {
        let input = black_box(Bytes::from_static(b"-ERR unknown command\r\n"));
        b.iter(|| {
            let _ = RespValue::parse_checked(&input);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple_string,
    bench_parse_bulk_string,
    bench_parse_integer,
    bench_parse_array,
    bench_parse_error,
);
criterion_main!(benches);
