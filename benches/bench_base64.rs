use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};
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
fn bench_to_base64(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    let mut seed = vec![0u8; 16384];
    rng.fill_bytes(&mut seed);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let mut input = [0u8; $size];
            input.copy_from_slice(&seed[..$size]);

            let mut group = c.benchmark_group(format!("ReprBytes<[u8; {}]>", $size));
            group.throughput(Throughput::Bytes($size));

            group.bench_function(BenchmarkId::new("to_base64", "native"), |b| {
                b.iter(|| STANDARD.encode(black_box(input.as_bytes())));
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

#[inline(always)]
fn bench_from_base64(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    let mut seed = vec![0u8; 16384];
    rng.fill_bytes(&mut seed);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let mut input = [0u8; $size];
            input.copy_from_slice(&seed[..$size]);
            let base64_input = STANDARD.encode(input);

            let mut group = c.benchmark_group(format!("ReprBytes<[u8; {}]>", $size));
            group.throughput(Throughput::Bytes($size));

            group.bench_function(BenchmarkId::new("from_base64", "native"), |b| {
                b.iter(|| STANDARD.decode(black_box(&base64_input)).unwrap());
            });

            group.bench_function(BenchmarkId::new("from_base64", "simd"), |b| {
                b.iter(|| <[u8; $size]>::from_base64(black_box(&base64_input)).unwrap());
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
    name = to_base64;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_to_base64
);

criterion_group!(
    name = from_base64;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_from_base64
);

criterion_main!(to_base64, from_base64);
