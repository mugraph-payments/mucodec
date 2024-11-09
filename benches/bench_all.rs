use std::time::Duration;

use base64::Engine;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    measurement::WallTime,
    BenchmarkId,
    Criterion,
    Throughput,
};
use mucodec::*;
use proptest::test_runner::{RngAlgorithm, TestRng};
use rand::prelude::*;

#[inline(always)]
fn bench_repr(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    let mut seed = vec![0u8; 16384];
    rng.fill_bytes(&mut seed);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let mut group = c.benchmark_group(format!("ReprBytes<[u8; {}]>", $size));
            group.throughput(Throughput::Bytes($size));

            let mut input = [0u8; $size];
            input.copy_from_slice(&seed[..$size]);

            group.bench_function(BenchmarkId::new("to_hex", "native"), |b| {
                b.iter(|| hex::encode(black_box(input.as_bytes())));
            });

            group.bench_function(BenchmarkId::new("to_hex", "simd"), |b| {
                b.iter(|| input.to_hex());
            });

            let hex_input = hex::encode(input);
            group.bench_function(BenchmarkId::new("from_hex", "native"), |b| {
                b.iter(|| hex::decode(black_box(&hex_input)).unwrap());
            });

            group.bench_function(BenchmarkId::new("from_hex", "simd"), |b| {
                b.iter(|| <[u8; $size]>::from_hex(black_box(&hex_input)).unwrap());
            });

            let mut input1 = [0u8; $size];
            let mut input2 = [0u8; $size];
            input1.copy_from_slice(&seed[..$size]);
            input2.copy_from_slice(&seed[$size..($size * 2)]);

            group.bench_function(BenchmarkId::new("equals", "native"), |b| {
                b.iter(|| black_box(&input1) == black_box(&input2));
            });

            group.bench_function(BenchmarkId::new("equals", "simd"), |b| {
                b.iter(|| black_box(&input1).equals(black_box(&input2)));
            });

            let mut input = [0u8; $size];
            input.copy_from_slice(&seed[..$size]);

            group.bench_function(BenchmarkId::new("to_base64", "native"), |b| {
                b.iter(|| {
                    base64::engine::general_purpose::STANDARD.encode(black_box(input.as_bytes()))
                });
            });

            group.bench_function(BenchmarkId::new("to_base64", "simd"), |b| {
                b.iter(|| input.to_base64());
            });

            group.finish();
        }};
    }

    bench_arr!(16);
    bench_arr!(32);
    bench_arr!(64);
    bench_arr!(128);
    bench_arr!(256);
    bench_arr!(512);
    bench_arr!(1024);
    bench_arr!(2048);
}

criterion_group!(
    name = bench;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_repr
);

criterion_main!(bench);
