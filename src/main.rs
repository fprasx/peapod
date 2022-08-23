use core::fmt::Debug;
use std::{collections::HashMap, marker::PhantomData};
use peapod::Peapod;
use phenotype_macro::{Phenotype, PhenotypeDebug};

fn main() {
    let mut pp = Peapod::<Generic<usize, u8>>::new(); 
    pp.push(Generic::One(&4));
    pp.push(Generic::Two(12, 34));
    println!("{:?}", pp.pop());
    println!("{:?}", pp.pop());
}

// Yay it works on this megageneric struct!
#[derive(Phenotype, PhenotypeDebug, Debug)]
enum Generic<'a, T, U: Debug> {
    One(&'a T),
    Two(U, U),
    Three(*mut *const T),
    Vec(Vec<T>),
    HashMap(HashMap<T, U>, [Vec<(T, U)>; 7]),
    Boo(PhantomData<&'a mut U>),
}
