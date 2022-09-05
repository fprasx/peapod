#![doc = include_str!("../README.md")]
// TODO: tests!
#![no_std]
 
// Exports :)
pub use crate::peapod::Peapod;
pub use phenotype_internal::{Phenotype, PhenotypeDebug};
pub use phenotype_macro::{Phenotype, PhenotypeDebug};

mod peapod;
 
// in the works
mod array;
