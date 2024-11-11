#![feature(generic_const_exprs)]

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
fn bench_pack_unpack(c: &mut Criterion<WallTime>) {
    let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[42u8; 32]);
    fn bench_list<const N: usize, const BIT_SIZE: usize, L>(
        c: &mut Criterion<WallTime>,
        _rng: &mut TestRng,
    ) where
        L: ReprBytes<{ N * (BIT_SIZE / 8) }> + Clone + 'static,
    {
        let input = L::zero();
        let packed_data = input.as_bytes();

        let mut group = c.benchmark_group(format!("Pack/Unpack {}-bit", BIT_SIZE));
        group.throughput(Throughput::Bytes(N as u64 * (BIT_SIZE / 8) as u64));

        group.bench_with_input(BenchmarkId::new("pack", N), &input, |b, i: &L| {
            b.iter(|| i.as_bytes());
        });

        group.bench_with_input(BenchmarkId::new("unpack", N), &packed_data, |b, data| {
            b.iter(|| L::from_bytes(*data));
        });

        group.finish();
    }

    // Benchmark for ListU16 with 16-bit elements
    bench_list::<128, 16, ListU16<128>>(c, &mut rng);
    bench_list::<256, 16, ListU16<256>>(c, &mut rng);
    bench_list::<512, 16, ListU16<512>>(c, &mut rng);
    bench_list::<1024, 16, ListU16<1024>>(c, &mut rng);
    bench_list::<2048, 16, ListU16<2048>>(c, &mut rng);
}

criterion_group!(
    name = pack_unpack;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(1000)
        .significance_level(0.01)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_pack_unpack
);

criterion_main!(pack_unpack);
