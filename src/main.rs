use peapod::{Peapod, Phenotype};

#[derive(Phenotype, PartialEq, Eq, Debug)]
enum Enum {
    Variant0,
    Variant1,
    Variant2,
}

fn main() {
    let mut pp = Peapod::new();
    pp.push(Enum::Variant0);
    pp.truncate(usize::MAX / 2 + 1);
    assert_eq!(pp.pop(), Some(Enum::Variant0));
}