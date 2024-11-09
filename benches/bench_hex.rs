use std::time::Duration;

use criterion::{
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
fn bench_to_hex(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    let mut seed = vec![0u8; 16384];
    rng.fill_bytes(&mut seed);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let mut input = [0u8; $size];
            input.copy_from_slice(&seed[..$size]);

            let mut group = c.benchmark_group("ReprHex::to_hex");
            group.throughput(Throughput::Bytes($size));

            group.bench_with_input(BenchmarkId::new("native", $size), &input, |b, i| {
                b.iter(|| hex::encode(i));
            });

            group.bench_with_input(BenchmarkId::new("simd", $size), &input, |b, i| {
                b.iter(|| i.to_hex());
            });

            group.finish();
        }};
    }

    bench_arr!(128);
    bench_arr!(256);
    bench_arr!(512);
    bench_arr!(1024);
    bench_arr!(2048);
}

#[inline(always)]
fn bench_from_hex(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    let mut seed = vec![0u8; 16384];
    rng.fill_bytes(&mut seed);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let mut input = [0u8; $size];
            input.copy_from_slice(&seed[..$size]);
            let hex_input = hex::encode(input);

            let mut group = c.benchmark_group("ReprHex::from_hex");
            group.throughput(Throughput::Bytes($size));

            group.bench_with_input(BenchmarkId::new("native", $size), &hex_input, |b, i| {
                b.iter(|| hex::decode(&i).unwrap());
            });

            group.bench_with_input(BenchmarkId::new("simd", $size), &hex_input, |b, i| {
                b.iter(|| <[u8; $size]>::from_hex(&i).unwrap());
            });

            group.finish();
        }};
    }

    bench_arr!(128);
    bench_arr!(256);
    bench_arr!(512);
    bench_arr!(1024);
    bench_arr!(2048);
}

criterion_group!(
    name = to_hex;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_to_hex
);

criterion_group!(
    name = from_hex;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_from_hex
);

criterion_main!(to_hex, from_hex);
