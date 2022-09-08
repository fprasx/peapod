use peapod::Phenotype;

fn main() {}


#[derive(Phenotype)]
enum Test {
    One(usize, usize),
    Two {
        one: usize,
        two: usize
    },
    Three
}