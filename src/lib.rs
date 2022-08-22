// TODO: tests!
use bitvec::{field::BitField, prelude::*};
use std::{
    cmp,
    fmt::{Debug, Display},
    mem::ManuallyDrop,
    ptr,
};

pub use phenotype_internal::Phenotype;
pub use phenotype_macro::Phenotype;

#[derive(Eq)]
pub struct Peapod<T: Phenotype> {
    tags: BitVec,
    data: Vec<T::Value>,
}

impl<T> Peapod<T>
where
    T: Phenotype,
{
    pub fn new() -> Self {
        Peapod {
            tags: BitVec::new(),
            data: Vec::new(),
        }
    }

    fn get_tag(&self, index: usize) -> usize {
        self.tags[index * T::BITS..(index + 1) * T::BITS].load()
    }

    fn set_tag(&mut self, index: usize, tag: usize) {
        self.tags[index * T::BITS..(index + 1) * T::BITS].store::<usize>(tag);
    }

    pub fn push(&mut self, t: T) {
        let pos = self.data.len();

        let (tag, data) = t.cleave();

        self.data.push(data);

        // Naively pushing seems to be faster than something like
        // self.tags
        //     .extend_from_bitslice(&BitView::view_bits::<Lsb0>(&[tag])[0..T::BITS]);
        // TODO: try bitshifty stuff?
        // like push((tags >> 1) & 1), push((tags >> 2) & 1), push((tags >> 3) & 1)
        for _ in 0..T::BITS {
            self.tags.push(false)
        }

        self.set_tag(pos, tag);
    }

    pub fn pop(&mut self) -> Option<T> {
        let len = self.data.len();

        if len == 0 {
            return None;
        }

        // This is safe as we checked that the length is not 0
        let data = self.data.pop().unwrap();

        // Subtract one as len is the length of the vector before removing an element
        // The subtraction will not underflow as pos is guaranteed != 0
        let tag: usize = self.get_tag(len - 1);

        // Remove the last tag
        self.tags.truncate((len - 1) * T::BITS);

        Some(Phenotype::reknit(tag, data))
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn reserve(&mut self, elements: usize) {
        self.data.reserve(elements);
        self.tags.reserve(elements * T::BITS);
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tags: BitVec::with_capacity(capacity * T::BITS),
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.tags.clear();
    }

    pub fn truncate(&mut self, len: usize) {
        self.tags.truncate(len * T::BITS);
        self.data.truncate(len);
    }

    pub fn capacity(&self) -> usize {
        let tag_cap = self.tags.capacity() / T::BITS;
        let data_cap = self.data.capacity();
        cmp::min(tag_cap, data_cap)
    }

    fn cleave(self) -> (BitVec, Vec<T::Value>) {
        let levitating = ManuallyDrop::new(self);
        unsafe { (ptr::read(&levitating.tags), ptr::read(&levitating.data)) }
    }

    pub fn append(&mut self, other: Peapod<T>) {
        let (mut otags, mut odata) = other.cleave();
        self.tags.append(&mut otags);
        self.data.append(&mut odata);
    }
}

impl<T> Drop for Peapod<T>
where
    T: Phenotype,
{
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

impl<T> From<Peapod<T>> for Vec<T>
where
    T: Phenotype,
{
    fn from(pp: Peapod<T>) -> Self {
        pp.into_iter().collect()
    }
}

impl<T> From<Vec<T>> for Peapod<T>
where
    T: Phenotype,
{
    fn from(pp: Vec<T>) -> Self {
        let mut tags = BitVec::<usize, Lsb0>::repeat(false, T::BITS * pp.len());
        let mut data = Vec::with_capacity(pp.len());
        for (index, (tag, value)) in pp.into_iter().map(|p| p.cleave()).enumerate() {
            tags[index * T::BITS..(index + 1) * T::BITS].store::<usize>(tag);
            data.push(value)
        }
        Self { tags, data }
    }
}

impl<T> Default for Peapod<T>
where
    T: Phenotype,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for Peapod<T>
where
    T: Phenotype,
{
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let (tags, data) = self.cleave();
        IntoIter {
            tags,
            data,
            index: 0,
        }
    }
}

pub struct IntoIter<T>
where
    T: Phenotype,
{
    tags: BitVec,
    data: Vec<T::Value>,
    index: usize,
}

impl<T> Iterator for IntoIter<T>
where
    T: Phenotype,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.data.len() {
            None
        } else {
            let elem = Some(<T as Phenotype>::reknit(
                self.tags[self.index * T::BITS..(self.index + 1) * T::BITS].load(),
                // Read a value out of the vector
                // # Safety
                // We are reading from a valid ptr (as_ptr), and the offset is
                // in bounds as we stop iterating once index == len
                unsafe { std::ptr::read(self.data.as_ptr().add(self.index)) },
            ));
            self.index += 1;
            elem
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.data.len(), Some(self.data.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T>
where
    T: Phenotype,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let len = self.data.len();
        if self.index == len {
            None
        } else {
            // Unwrap is ok as we know self.index < self.data.len so iteration is not over
            let data = self.data.pop().unwrap();
            let tag = self.tags[(len - 1) * T::BITS..len * T::BITS].load();
            Some(<T as Phenotype>::reknit(tag, data))
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T>
where
    T: Phenotype,
{
    fn len(&self) -> usize {
        let (lower, upper) = self.size_hint();
        // Note: This assertion is overly defensive, but it checks the invariant
        // guaranteed by the trait. If this trait were rust-internal,
        // we could use debug_assert!; assert_eq! will check all Rust user
        // implementations too.
        assert_eq!(upper, Some(lower));
        lower
    }
}

impl<T> Debug for Peapod<T>
where
    T: Phenotype,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Peapod")
            .field(
                "tags",
                &self
                    .tags
                    .chunks(T::BITS)
                    .map(BitField::load::<usize>)
                    .collect::<Vec<_>>(),
            )
            .field("data", &[..])
            .finish()
    }
}

impl<T> Display for Peapod<T>
where
    T: Phenotype,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for i in 0..self.len() {
            f.write_str(" ")?;
            f.write_str(&format!("{{ tag: {}, data: .. }}", self.get_tag(i)))?;
            f.write_str(",")?;
        }
        f.write_str(" ")?;
        f.write_str("]")?;
        Ok(())
    }
}

impl<T> Extend<T> for Peapod<T>
where
    T: Phenotype,
{
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        // If we can, reserve space ahead of time
        let iter = iter.into_iter();
        if let (_, Some(len)) = iter.size_hint() {
            self.reserve(len);
        }
        for elem in iter {
            self.push(elem);
        }
    }
}

impl<T> FromIterator<T> for Peapod<T>
where
    T: Phenotype,
{
    fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
        let mut pp = Peapod::<T>::new();
        pp.extend(iter);
        pp
    }
}

impl<T> PartialEq for Peapod<T>
where
    T: Phenotype,
    T::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags && self.data == other.data
    }
}

impl<T> Clone for Peapod<T>
where
    T: Phenotype,
    T::Value: Clone,
{
    fn clone(&self) -> Self {
        Self {
            tags: self.tags.clone(),
            data: self.data.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Phenotype, PartialEq, Debug)]
    enum TestData {
        A { u: usize, f: f64 },
        B(usize, f64),
        C,
    }

    #[test]
    fn new_is_empty() {
        let pp = Peapod::<TestData>::new();
        assert_eq!(pp.len(), 0);
        assert_eq!(pp.capacity(), 0);
    }

    #[test]
    fn push_increases_len() {
        let mut pp = Peapod::<TestData>::new();
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.push(TestData::A { u: 1, f: 1.0 });
        assert_eq!(pp.len(), 4);
    }

    #[test]
    fn pop_works() {
        let mut pp = Peapod::<TestData>::new();
        assert_eq!(pp.pop(), None);
        pp.push(TestData::A { u: 1, f: 1.0 });
        assert_eq!(pp.pop(), Some(TestData::A { u: 1, f: 1.0 }));
    }

    #[test]
    fn clear_clears_empty() {
        let mut pp = Peapod::<TestData>::new();
        pp.clear();
        assert_eq!(pp.len(), 0);
        assert_eq!(pp.capacity(), 0);
    }

    #[test]
    fn clear_clears_nonempty() {
        let mut pp = Peapod::<TestData>::new();
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.push(TestData::A { u: 1, f: 1.0 });
        pp.clear();
        assert_eq!(pp.len(), 0);
    }
}
