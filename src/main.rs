use phenotype::Peapod;
use phenotype_internal::Phenotype;
use phenotype_macro::Phenotype;
use bitvec::prelude::*;
use bitvec::view::BitView;
fn main() {
    // UB!! hehe
    let mut pp = Peapod::<Test>::new();
    let (_, data) = Test::U(3).cleave();
    pp.tags.extend_from_bitslice(BitView::view_bits::<Lsb0>(&[1usize]));
    pp.data.push(data);
    println!("{:?}", pp.pop());
}

#[derive(Phenotype, Debug)]
enum Test {
    U(usize),
    B(bool)
}