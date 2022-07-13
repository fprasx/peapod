#![allow(dead_code, unused_attributes, unused_variables)]
use phenotype_internal::Phenotype;
use phenotype_macro::phenotype;
fn main() {
    let disc = Test::C.discriminant();
    println!("{disc}")
}

#[derive(phenotype)]
enum Test {
    A(usize, u32),
    B { f: f64, u: u64 },
    C,
}
