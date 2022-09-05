use peapod::{Peapod, Phenotype};

fn main() {
    // The Peapod representation is a lot smaller!
    // These numbers are bytes
    assert_eq!(ILovePeas::PEAPOD_SIZE.unwrap(), 9);
    assert_eq!(std::mem::size_of::<ILovePeas>(), 16);

    let mut pp = Peapod::new();
    pp.push(ILovePeas::SnowPea);
    pp.push(ILovePeas::Edamame(0x9EA90D));
    pp.push(ILovePeas::GeneticPea {
        wrinkled: true,
        yellow: true,
    });

    for pea in pp {
        // do something with pea!
    }
}

#[derive(Phenotype)] // <- this is where the magic happens
enum ILovePeas {
    Edamame(usize),
    SnowPea,
    GeneticPea { wrinkled: bool, yellow: bool },
}
