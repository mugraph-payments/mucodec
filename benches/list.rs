#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

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

    fn bench_list<const N: usize, const BYTES: usize, L: ReprBytes<BYTES>>(
        c: &mut Criterion<WallTime>,
        _rng: &mut TestRng,
    ) where
        L: ReprPacked + Clone + 'static,
    {
        let input = L::zero();
        let (bit_width, packed_data) = input.pack();

        let mut group = c.benchmark_group(format!("Pack/Unpack List<{N}>"));
        group.throughput(Throughput::Bytes(N as u64));

        group.bench_with_input(BenchmarkId::new("pack", N), &input, |b, i| {
            b.iter(|| i.pack());
        });

        group.bench_with_input(
            BenchmarkId::new("unpack", N),
            &(bit_width, packed_data.clone()),
            |b, (width, data)| {
                b.iter(|| L::unpack(*width, data));
            },
        );

        group.finish();
    }

    // For ListU16, each element is 2 bytes + 1 byte for bit width
    bench_list::<64, { 64 * 2 + 1 }, ListU16<64>>(c, &mut rng);
    bench_list::<128, { 128 * 2 + 1 }, ListU16<128>>(c, &mut rng);
    bench_list::<256, { 256 * 2 + 1 }, ListU16<256>>(c, &mut rng);
    bench_list::<512, { 512 * 2 + 1 }, ListU16<512>>(c, &mut rng);
    bench_list::<1024, { 1024 * 2 + 1 }, ListU16<1024>>(c, &mut rng);

    // For ListU32, each element is 4 bytes + 1 byte for bit width
    bench_list::<64, { 64 * 4 + 1 }, ListU32<64>>(c, &mut rng);
    bench_list::<128, { 128 * 4 + 1 }, ListU32<128>>(c, &mut rng);
    bench_list::<256, { 256 * 4 + 1 }, ListU32<256>>(c, &mut rng);
    bench_list::<512, { 512 * 4 + 1 }, ListU32<512>>(c, &mut rng);
    bench_list::<1024, { 1024 * 4 + 1 }, ListU32<1024>>(c, &mut rng);

    // For ListU64, each element is 8 bytes + 1 byte for bit width
    bench_list::<64, { 64 * 8 + 1 }, ListU64<64>>(c, &mut rng);
    bench_list::<128, { 128 * 8 + 1 }, ListU64<128>>(c, &mut rng);
    bench_list::<256, { 256 * 8 + 1 }, ListU64<256>>(c, &mut rng);
    bench_list::<512, { 512 * 8 + 1 }, ListU64<512>>(c, &mut rng);
    bench_list::<1024, { 1024 * 8 + 1 }, ListU64<1024>>(c, &mut rng);
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
