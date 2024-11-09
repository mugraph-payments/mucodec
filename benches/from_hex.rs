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
fn bench_from_hex(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    fn bench_arr<const N: usize>(c: &mut Criterion<WallTime>, rng: &mut TestRng) {
        let input = Bytes::<N>::random(rng).to_hex();

        let mut group = c.benchmark_group("ReprHex");
        group.throughput(Throughput::Bytes(N as u64));

        group.bench_with_input(BenchmarkId::new("from_hex (native)", N), &input, |b, i| {
            b.iter(|| hex::decode(&i).unwrap());
        });

        group.bench_with_input(BenchmarkId::new("from_hex (simd)", N), &input, |b, i| {
            b.iter(|| Bytes::<N>::from_hex(i).unwrap());
        });

        group.finish();
    }

    bench_arr::<128>(c, &mut rng);
    bench_arr::<256>(c, &mut rng);
    bench_arr::<512>(c, &mut rng);
    bench_arr::<1024>(c, &mut rng);
    bench_arr::<2048>(c, &mut rng);
}

criterion_group!(
    name = from_hex;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_from_hex
);

criterion_main!(from_hex);
