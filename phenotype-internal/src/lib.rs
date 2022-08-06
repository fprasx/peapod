// TODO: write auto-method that tells size of new type
// tag + union

/// This trait represents the behaviour of a tagged union.
/// **Note**: it should only be implemented with the `derive` macro.
pub trait Phenotype {
    const NUM_VARIANTS: usize;
    // const PEAPOD_SIZE: usize;
    type Value;
    fn discriminant(&self) -> usize;
    
    /// Takes an enum variant and `cleave`s it into a two parts:
    /// a tag, and an union representing the data the enum can hold.
    /// If the enum variant doesn't hold data, `None` is returned as
    /// the second tuple element.
    fn cleave(self) -> (usize, Option<Self::Value>);
    fn reknit(tag: usize, value: Option<Self::Value>) -> Self;
}

// TODO: write the derive macro for this
pub trait PhenotypeDebug {
    fn discriminant(&self) -> usize;
    fn debug(tag: usize) -> &'static str;
    
}
