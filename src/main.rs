use phenotype::Peapod;
use phenotype_macro::Phenotype;

fn main() {
    let mut pp = Peapod::new();
    pp.push(Test2::A(Helper::default()));
    pp.push(Test2::B {
        helper: Helper::default(),
    });
    pp.push(Test2::A(Helper::default()));
    pp.push(Test2::A(Helper::default()));
    pp.push(Test2::B {
        helper: Helper::default(),
    });
    for i in pp {
        println!("{i:?}")
    }
}

#[derive(Phenotype, Debug)]
enum Test0 {
    A,
    B,
    C,
}
#[derive(Phenotype)]
enum Test1 {
    A(),
    B {},
    C,
}
#[derive(Phenotype, Debug)]
enum Test2 {
    A(Helper),
    B { helper: Helper },
}

#[derive(Default, Debug)]
struct Helper {
    _a: usize,
    _b: f64,
}

/*

#[derive(Phenotype)]
enum Test3 {}
#[derive(Phenotype)]
enum Test4 {}
#[derive(Phenotype)]
enum Test5 {}
#[derive(Phenotype)]
enum Test6 {}
#[derive(Phenotype)]
enum Test7 {}
#[derive(Phenotype)]
enum Test8 {}
#[derive(Phenotype)]
enum Test9 {}

*/
