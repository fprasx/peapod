// TODO: add examples

/// This trait represents the behaviour of a tagged union.
/// **Note**: it should only be implemented with the `derive` macro.
pub trait Phenotype {
    /// The number of variants of the enum.
    const NUM_VARIANTS: usize;

    /// The number of bits needed to represent every variant of the enum.
    const BITS: usize;

    /// The number of bits `Phenotype` uses to represent and instance of a type
    const PEAPOD_SIZE: usize;

    /// Whether using `Phenotype` produces a more compact representation.
    /// Will return `true` if implementations are the same size.
    const IS_MORE_COMPACT: bool;

    /// A type that represents all the data an enum can contain.
    /// This should be a union whose fields each represent a particular
    /// enum variant.
    type Value;

    // TODO: remove this?
    fn discriminant(&self) -> usize;

    /// Takes an enum variant and `cleave`s it into a two parts:
    /// a tag, and an union representing the data the enum can hold.
    /// If the enum variant doesn't hold data, `None` is returned as
    /// the second tuple element.
    fn cleave(self) -> (usize, Self::Value);

    /// Takes a tag and a value and recombines them into a proper
    /// instance of an enum variant. Calling this function with incorrect
    /// inputs can result in undefined behavior. The tag must always match
    /// the state that the union is in.
    ///
    /// For example, consider the following example
    /// ```
    /// #[derive(Phenotype)]
    /// enum UB {
    ///     U(usize), // -> tag = 0
    ///     B(bool)   // -> tag = 1
    /// }
    ///
    /// // This is the type <UB as Phenotype>::Value
    /// union Value {
    ///     U: usize,
    ///     B: bool
    /// }
    ///
    /// use peapod::Phenotype;
    /// fn main {
    ///     let ub = UB::U(3);
    ///     let (_, data) = ub.cleave();
    ///     // ** DANGER **
    ///     // We are interpreting 3 as a bool! That's undefined behavior.
    ///     let BAD = <UB as Phenotype>::reknit(1, data);
    /// }
    /// ```
    fn reknit(tag: usize, value: Self::Value) -> Self;
}

// TODO: write the derive macro for this
pub trait PhenotypeDebug {
    fn discriminant(&self) -> usize;
    fn debug(tag: usize) -> &'static str;
}
