use phenotype::Peapod;
use phenotype_internal::Phenotype;
use phenotype_macro::phenotype;

fn main() {
    let mut pp = Peapod::new();
    pp.push(Test::A(1, 5));
    pp.push(Test::B { f: 1.0, u: (1, 1) });
    pp.push(Test::C);
    println!(
        "Size of normal vector: {}",
        pp.len() * std::mem::size_of::<Test>()
    );
    println!(
        "Size of peapod: {}",
        pp.len() + pp.len() * std::mem::size_of::<<Test as Phenotype>::Value>()
    );
    println!("Popped off {:?}", pp.pop());
    println!("Popped off {:?}", pp.pop());
    println!("Popped off {:?}", pp.pop());
    println!("All the edamame has been removed");
}

#[derive(phenotype, Debug)]
enum Test {
    A(usize, u32),
    B { f: f64, u: (u32, u32) },
    C,
}

#[derive(Debug)]
struct Data {
    _u: usize,
    _r: usize,
    _f: usize,
}
