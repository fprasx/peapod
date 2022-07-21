use phenotype_internal::Phenotype;
use phenotype_macro::phenotype;
fn main() {
    let a = Test::A(
        1,
        Data {
            _u: 2,
            _r: 3,
            _f: 4,
        },
        5,
    );
    let (tag, data) = a.cleave();
    let reknitted = <Test as Phenotype>::reknit(tag, data);
    println!("{reknitted:?}");

    let b = Test::B { f: 1.0, u: (1, 1) };
    let (tag, data) = b.cleave();
    let reknitted = <Test as Phenotype>::reknit(tag, data);
    println!("{reknitted:?}");

    let b = Test::C;
    let (tag, data) = b.cleave();
    let reknitted = <Test as Phenotype>::reknit(tag, data);
    println!("{reknitted:?}");
}

#[derive(phenotype, Debug)]
enum Test {
    A(usize, Data, u32),
    B { f: f64, u: (u32, u32) },
    C,
}

#[derive(Debug)]
struct Data {
    _u: usize,
    _r: usize,
    _f: usize,
}
