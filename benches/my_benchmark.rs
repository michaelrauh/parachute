use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parachute::{
    folder::{merge_process, single_process},
    registry::Registry,
};

fn merge_benchmark(c: &mut Criterion) {
    // a b  e f
    // c d  g h

    let mut left_registry = Registry::from_text(
        "a b. a c. b d. a e. b f. c g. d h. a b. y u. a y. b u. y u. d x. y d. u x. o p. a o. a i. p o.",
        "first.txt",
        1,
    );

    single_process(&mut left_registry);
    let mut right_registry = Registry::from_text("c d. e f. g h. e g. f h.", "second.txt", 2);

    single_process(&mut right_registry);

    c.bench_function("sift merge", |b| {
        b.iter(|| merge_process(black_box(&mut left_registry), black_box(right_registry.clone())))
    });
}

criterion_group!(benches, merge_benchmark);
criterion_main!(benches);
