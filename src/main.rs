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
    pp.into_iter().next_back();
}

macro_rules! tests {
    ($type:ty, $($variant:ident),*) => {
        $(
            #[allow(non_snake_case)]
            mod $variant {
                use super::*;
                #[test]
                fn test() {
                        let mut pp = peapod::Peapod::new();
                            pp.push(<$type>::$variant);
                        pp.pop();
                        for _ in pp {}
                }
            }
        )*
    };
}
tests! {Holder, Variant}
