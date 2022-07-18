#![allow(dead_code, unused_attributes, unused_variables)]
use phenotype_internal::Phenotype;
use phenotype_macro::phenotype;
fn main() {
    let x = Test::A(1, Data { u: 2, r: 3, f: 4 }, 5);
    let z = x.value();
}

#[derive(phenotype)]
enum Test {
    A(usize, Data, u32),
    B { f: f64, u: (u32, u32) },
    C,
}

struct Data {
    u: usize,
    r: usize,
    f: usize,
}