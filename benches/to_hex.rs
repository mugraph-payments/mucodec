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

#[inline(always)]
fn bench_to_hex(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);

    macro_rules! bench_arr {
        ($size:expr) => {{
            let input = Bytes::<$size>::random(&mut rng);

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

criterion_group!(
    name = to_hex;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_to_hex
);

criterion_main!(to_hex);
