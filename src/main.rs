use phenotype_macro::Phenotype;

fn main() {
}

#[derive(Phenotype)]
enum Test0
{
    A,
    B,
    C
}
#[derive(Phenotype)]
enum Test1
{
    A(),
    B {},
    C
}
#[derive(Phenotype)]
enum Test2 {
    A(Helper),
    B { helper: Helper}
}

struct Helper {
    _a: usize,
    _b: f64
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