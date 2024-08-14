use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use criterion_cycles_per_byte::CyclesPerByte;
use dungers_bitbuf::{BitReader, BitWriter};

fn bench_write_ubit64(c: &mut Criterion<CyclesPerByte>) {
    let mut group = c.benchmark_group("write_ubit64");
    let mut buf = [0u8; 1024];

    group.bench_function("write 64 bits", |b| {
        b.iter(|| {
            let mut w = BitWriter::new(&mut buf);
            w.write_ubit64(black_box(0xdeadbeefcafebabe), 64).unwrap();
        })
    });

    group.bench_function("write 32 bits", |b| {
        b.iter(|| {
            let mut w = BitWriter::new(&mut buf);
            w.write_ubit64(black_box(0xdeadbeef), 32).unwrap();
        })
    });

    group.bench_function("write 8 bits", |b| {
        b.iter(|| {
            let mut w = BitWriter::new(&mut buf);
            w.write_ubit64(black_box(0xaa), 8).unwrap();
        })
    });

    group.bench_function("spanning blocks", |b| {
        b.iter(|| {
            let mut w = BitWriter::new(&mut buf);
            w.write_ubit64(black_box(0xfffffffffffffff), 60).unwrap();
            w.write_ubit64(black_box(0xaa), 8).unwrap();
        })
    });

    group.finish();
}

fn bench_read_ubit64(c: &mut Criterion<CyclesPerByte>) {
    let mut group = c.benchmark_group("read_ubit64");
    let mut buf = [0xFFu8; 1024];

    group.bench_function("read 64 bits", |b| {
        b.iter(|| {
            let mut r = BitReader::new(&mut buf);
            black_box(r.read_ubit64(64).unwrap());
        })
    });

    group.bench_function("read 32 bits", |b| {
        b.iter(|| {
            let mut r = BitReader::new(&mut buf);
            black_box(r.read_ubit64(32).unwrap());
        })
    });

    group.bench_function("read 8 bits", |b| {
        b.iter(|| {
            let mut r = BitReader::new(&mut buf);
            black_box(r.read_ubit64(8).unwrap());
        })
    });

    group.bench_function("spanning blocks", |b| {
        b.iter(|| {
            let mut r = BitReader::new(&mut buf);
            black_box(r.read_ubit64(60).unwrap());
            black_box(r.read_ubit64(8).unwrap());
        })
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(CyclesPerByte);
    targets = bench_write_ubit64, bench_read_ubit64
);
criterion_main!(benches);
