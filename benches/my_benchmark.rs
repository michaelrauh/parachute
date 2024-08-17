use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parachute::{
    folder::{merge_process, single_process},
    registry::Registry,
};

fn merge_benchmark(c: &mut Criterion) {
    // a b  e f
    // c d  g h

    let left_registry = single_process(&Registry::from_text(
        "a b. a c. b d. a e. b f. c g. d h. a b. y u. a y. b u. y u. d x. y d. u x. o p. a o. a i. p o.",
        "first.txt",
        1,
    ));
    let right_registry = single_process(&Registry::from_text(
        "c d. e f. g h. e g. f h.",
        "second.txt",
        2,
    ));

    c.bench_function("sift merge", |b| {
        b.iter(|| merge_process(black_box(&left_registry), black_box(&right_registry)))
    });
}

criterion_group!(benches, merge_benchmark);
criterion_main!(benches);
