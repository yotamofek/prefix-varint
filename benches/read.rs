use std::io;

use criterion::{criterion_group, criterion_main, Criterion};
use prefix_varint::{read_varint, write_varint};
use rand::{prelude::StdRng, Rng, SeedableRng};

pub fn benchmark_read(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(101);

    c.bench_function("read_varint", |b| {
        b.iter_batched(
            || {
                let mut varint = vec![];
                write_varint(rng.gen(), &mut varint).unwrap();
                varint
            },
            |varint| read_varint(&mut io::Cursor::new(varint)),
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, benchmark_read);
criterion_main!(benches);
