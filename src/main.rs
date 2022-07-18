#![allow(dead_code, unused_attributes, unused_variables)]
use phenotype_internal::Phenotype;
use phenotype_macro::phenotype;
fn main() {
    let e = Test::A(1, 2, 3);
    let des = e.value();
    println!("{}", des.0);
}

#[derive(phenotype)]
enum Test {
    A(usize, usize, u32),
    B { f: f64, u: u64 },
    C,
}
