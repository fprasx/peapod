// TODO: add examples

/// This trait represents the behaviour of an `enum`/tagged union.
/// **Note**: it should only be implemented with `#[derive(Phenotype)]`
pub trait Phenotype {
    /// The number of variants of the enum.
    const NUM_VARIANTS: usize;

    /// The number of bits needed to represent every variant of the enum.
    /// For example, if the enum has 4 variants, then two bits are needed.
    const BITS: usize;

    /// The number of bits `Phenotype` uses to represent and instance of a type.
    /// If the type `Phenotype` is being implemented for is generic,
    /// this will be `None`, as sizes may vary accross different
    /// generic parameters. For example, `Type<usize>` could be differently
    /// sized than `Type<[usize; 4]>`
    const PEAPOD_SIZE: Option<usize>;

    /// Whether using `Phenotype` produces a more compact representation.
    /// Will be `Some(true)` if implementations are the same size.
    /// If the type `Phenotype` is being implemented for is generic,
    /// this will be `None`, as sizes may vary accross different
    /// generic parameters. For example, `Type<usize>` could be differently
    /// sized than `Type<[usize; 4]>`
    const IS_MORE_COMPACT: Option<bool>;

    /// A type that represents all the data an enum can contain.
    /// This should be a union whose fields each represent a particular
    /// enum variant.
    type Value;

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

/// Some helpful methods for using `Phenotype`
pub trait PhenotypeDebug: Phenotype {
    /// Returns the tag that Phenotype uses internally
    /// to identify the enum variant.
    /// **Note**: this is different from the tag the compiler
    /// uses or a discriminant that was manually specified. It
    /// only has meaning in the context of `Phenotype`.
    fn discriminant(&self) -> usize;

    /// Takes a tag and returns a string that represents
    /// the variant. For example, it might return something
    /// like `Result::Ok` for 0 and `Result::Err` for 1 if `Phenotype`
    /// was derived on the `Result` type.
    fn debug_tag(tag: usize) -> &'static str;
}
