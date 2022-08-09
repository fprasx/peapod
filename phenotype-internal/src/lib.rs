// TODO: write auto-method that tells size of new type
// tag + union

// TODO: change to only return self::value, no option

/// This trait represents the behaviour of a tagged union.
/// **Note**: it should only be implemented with the `derive` macro.
pub trait Phenotype {
    /// The number of variants of the enum.
    const NUM_VARIANTS: usize;
    
    /// The number of bits needed to represent every variant of the enum.
    const BITS: usize;

    // const PEAPOD_SIZE: usize;

    type Value;

    // TODO: remove this?
    fn discriminant(&self) -> usize;

    /// Takes an enum variant and `cleave`s it into a two parts:
    /// a tag, and an union representing the data the enum can hold.
    /// If the enum variant doesn't hold data, `None` is returned as
    /// the second tuple element.
    fn cleave(self) -> (usize, Self::Value);

    fn reknit(tag: usize, value: Self::Value) -> Self;
}

// TODO: write the derive macro for this
pub trait PhenotypeDebug {
    fn discriminant(&self) -> usize;
    fn debug(tag: usize) -> &'static str;
}
