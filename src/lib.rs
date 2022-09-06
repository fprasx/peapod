#![doc = include_str!("../README.md")]
// TODO: tests!
#![no_std]

// Exports :)
pub use crate::peapod::Peapod;
pub use phenotype_internal as traits;
pub use phenotype_macro as macros;
pub use phenotype_macro::Phenotype;

mod peapod;

// in the works
mod array;
