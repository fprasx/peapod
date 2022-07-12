use phenotype_macro::phenotype;
fn main() {
    println!("this is working")
}

#[derive(phenotype)]
enum Test {
    A,
    B,
    C
}