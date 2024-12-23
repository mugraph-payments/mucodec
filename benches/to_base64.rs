use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};
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

#[inline(always)]
fn bench_to_base64(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let input = Bytes::<$size>::random(&mut rng);

            let mut group = c.benchmark_group("ReprBase64::to_base64");
            group.throughput(Throughput::Bytes($size));

            group.bench_with_input(BenchmarkId::new("native", $size), &input, |b, i| {
                b.iter(|| STANDARD.encode(i));
            });

            group.bench_with_input(BenchmarkId::new("simd", $size), &input, |b, i| {
                b.iter(|| i.to_base64());
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
    name = to_base64;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_to_base64
);

criterion_main!(to_base64);
