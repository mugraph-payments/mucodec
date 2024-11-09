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
fn bench_bytes(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);

    fn bench_arr<const N: usize>(c: &mut Criterion<WallTime>, rng: &mut TestRng) {
        let a = Bytes::<N>::random(rng);
        let b = Bytes::<N>::random(rng);

        let mut group = c.benchmark_group("Bytes");
        group.throughput(Throughput::Bytes(N as u64));

        group.bench_with_input(
            BenchmarkId::new("== (native)", N),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| *a == *b);
            },
        );

        group.bench_with_input(
            BenchmarkId::new("== (simd)", N),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| a == b);
            },
        );

        group.finish();
    }

    bench_arr::<128>(c, &mut rng);
    bench_arr::<256>(c, &mut rng);
    bench_arr::<512>(c, &mut rng);
    bench_arr::<1024>(c, &mut rng);
    bench_arr::<2048>(c, &mut rng);
}

criterion_group!(
    name = bench;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(1))
        .sample_size(10000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(1));
    targets = bench_bytes
);

criterion_main!(bench);
