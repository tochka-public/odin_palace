use std::fs;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn parser(c: &mut Criterion) {
    let file = match fs::read("benches/bench_statement.txt") {
        Ok(file) => file,
        Err(e) => panic!("cannot read the bench statement: {e}"),
    };
    c.bench_function("parser", |b| {
        b.iter(|| odin_palace::parser::Parser::default().parse(black_box(&file)));
    });
}

criterion_group!(benches, parser);
criterion_main!(benches);
