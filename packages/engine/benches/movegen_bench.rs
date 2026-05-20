use criterion::{criterion_group, criterion_main, Criterion};
use specter::board::position::Position;
use specter::movegen::attacks::init_all;
use specter::movegen::legal::legal_moves;

fn bench_movegen(c: &mut Criterion) {
    init_all();
    let mut pos = Position::startpos();
    c.bench_function("legal_moves_startpos", |b| {
        b.iter(|| legal_moves(&mut pos))
    });
}

criterion_group!(benches, bench_movegen);
criterion_main!(benches);
