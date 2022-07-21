// TODO: write macro that convert enum to union
pub trait Phenotype {
    const NUM_VARIANTS: usize;
    type Value;
    fn discriminant(&self) -> usize;
    // Return autogenerated enum
    fn cleave(self) -> (usize, Option<Self::Value>);
    fn reknit(tag: usize, value: Option<Self::Value>) -> Self;
}
