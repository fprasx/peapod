use peapod::Peapod;
use phenotype_macro::Phenotype;

fn main() {
    let x = Test2::A(Helper { _a: 1, _b: 1.0 });
    let mut pp = Peapod::new();
    pp.push(x);
}

enum Tuples {
    A(usize, usize),
    B(isize, isize),
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

impl Drop for Test2 {
    fn drop(&mut self) {
        println!("Dropping test2!")
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        println!("Dropping helper!")
    }
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
