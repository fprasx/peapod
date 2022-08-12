extern crate criterion;
use self::criterion::*;
use peapod::Peapod;
use phenotype_macro::Phenotype;

fn vec(c: &mut Criterion) {
    // Setup (construct data, allocate memory, etc)
    let mut vec = Vec::with_capacity(1);
    for _ in 0..5 {
        vec.push(Benchy::Named { one: 1, two: 2 });
    }
    c.bench_function(
        "vec",
        |b| b.iter(|| {
            vec.pop();
            vec.pop();
            vec.pop();
            vec.pop();
            vec.pop();
        }),
    );
}

#[derive(Phenotype)]
enum Benchy {
    Named { one: usize, two: usize},
    Unnamed(usize, usize),
    None,
    C,
    D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V,
    D2, E2, F2, G2, H2, I2, J2, K2, L2, M2, N2, O2, P2, Q2, R2, S2, T2, U2, V2
}


fn peapod(c: &mut Criterion) {
    // Setup (construct data, allocate memory, etc)
    let mut pp = Peapod::new();
    for _ in 0..5 {
        pp.push(Benchy::Named { one: 1, two: 2 });
    }
    c.bench_function(
        "peapod",
        |b| b.iter(|| {
            pp.pop();
            pp.pop();
            pp.pop();
            pp.pop();
            pp.pop();
        }),
    );
}

criterion_group!(benches, vec, peapod);
criterion_main!(benches);