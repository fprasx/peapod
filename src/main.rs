use peapod::{peapod, Phenotype};

#[derive(Phenotype, Debug)]
enum Holder {
    Variant,
}

fn main() {
    println!("{}", Holder::BITS);
    let mut pp = peapod![Holder::Variant];
    pp.push(Holder::Variant);
    pp.pop();
    pp.push(Holder::Variant);
    pp.push(Holder::Variant);
    pp.push(Holder::Variant);
    for p in pp {
        println!("{p:?}")
    }
    let mut pp = peapod![Holder::Variant];
    pp.push(Holder::Variant);
    pp.pop();
    pp.push(Holder::Variant);
    pp.push(Holder::Variant);
    pp.push(Holder::Variant);
    println!("Debug: {pp:?}");
    println!("Display: {pp}");
    pp.into_iter().next_back();
}
