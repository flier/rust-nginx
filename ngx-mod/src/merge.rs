/// The `Merge` trait provides a method for merging configuration down through each level.
pub trait Merge {
    type Error;

    /// Module merge function.
    fn merge(&mut self, prev: &Self) -> Result<(), Self::Error>;
}

impl Merge for () {
    type Error = ();

    fn merge(&mut self, _prev: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}
