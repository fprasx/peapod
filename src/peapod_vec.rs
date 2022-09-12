extern crate alloc;
use alloc::{format, vec::Vec};
use bitvec::{field::BitField, prelude::*};
use core::{
    cmp,
    fmt::{self, Debug, Display},
    mem::ManuallyDrop,
    ptr,
};
use phenotype_internal::Phenotype;

// credit: https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#bit-twiddling
#[doc(hidden)]
#[macro_export]
macro_rules! count_tts {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { ($crate::count_tts!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { $crate::count_tts!($($a)*) << 1 };
}

#[macro_export]
/// A nice way to generate a `Peapod` from a list of elements. If you're familiar
/// with the `vec![]` macro, this is `Peapod`'s equivalent.
/// ```rust
/// # use peapod::{Peapod, Phenotype, peapod};
/// #[derive(Phenotype)]
/// enum Test {
///     A,
///     B
/// }
///
/// let mut fast = peapod![Test::A, Test::B];
///
/// // is the same as
///
/// let mut slow = Peapod::with_capacity(2);
/// slow.push(Test::A);
/// slow.push(Test::B);
/// ```
macro_rules! peapod {
    () => {
        $crate::Peapod::new();
    };
    ($($elem:expr),+ $(,)?) => {
        {
            let count = $crate::count_tts!($($elem:expr),*);
            let mut pp = $crate::Peapod::with_capacity(count);
            $(pp.push($elem);)*
            pp
        }

    };
}

/// A `vec`-like data structure for compactly storing `enum`s that implement [`Phenotype`].
#[derive(Eq)]
pub struct Peapod<T: Phenotype> {
    tags: BitVec,
    data: Vec<T::Value>,
}

impl<T> Peapod<T>
where
    T: Phenotype,
{
    /// Create a new `Peapod` with 0 capacity and 0 length. This does not allocate.
    pub fn new() -> Self {
        Peapod {
            tags: BitVec::new(),
            data: Vec::new(),
        }
    }

    // **Note**: index must be in range
    fn get_tag(&self, index: usize) -> usize {
        self.tags[index * T::BITS..(index + 1) * T::BITS].load()
    }

    // **Note**: index must be in range
    fn set_tag(&mut self, index: usize, tag: usize) {
        self.tags[index * T::BITS..(index + 1) * T::BITS].store::<usize>(tag);
    }

    /// Append a new element to the end of the collection.
    /// 
    /// ## Panics
    /// Panics if the underlying `bitvec` or `Vec` panics.
    /// The underlying [`Vec`](https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#panics-7) 
    /// will panic if its allocation exceeds
    /// `isize::MAX` bytes. The underlying `bitvec` will panic
    /// if the maximum tag capacity is exceeded.
    /// On 32-bit systems, maximum tag capacity is `0x1fff_ffff/T::BITS` tags.
    /// On 64-bit systems, maximum tag capacity is `0x1fff_ffff_ffff_ffff/T::BITS` tags.
    pub fn push(&mut self, t: T) {
        let pos = self.data.len();

        let (tag, data) = t.cleave();

        // Naively pushing seems to be faster than something like
        // self.tags
        //     .extend_from_bitslice(&BitView::view_bits::<Lsb0>(&[tag])[0..T::BITS]);
        for _ in 0..T::BITS {
            self.tags.push(false)
        }

        // https://github.com/fprasx/peapod/issues/2
        // We have to push the data second because pushing to
        // self.tags will panic if capacity is exceeded.
        // If this panic is caught and we already pushed a 
        // value to self.data, but not self.tags, there
        // will be an untagged value on the end of self.data.
        //
        // This is still not good as we will have some cruft
        // on the end of self.tags, but because we always use
        // self.data to get self's length, and we always use
        // get/set instead of push/pop to modify self.tags,
        // the correct tag and data should still always match
        // up.
        self.data.push(data);

        self.set_tag(pos, tag);
    }

    /// Remove an element from the end of the collection.
    /// Returns `None` if the collection is empty.
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

        // # Safety
        // The tag matches the data
        unsafe { Some(Phenotype::reknit(tag, data)) }
    }

    /// Returns the number of elements in the collection.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of elements in the collection.
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    /// Returns whether the collection is empty (it contains no elements).
    pub fn reserve(&mut self, elements: usize) {
        self.data.reserve(elements);
        self.tags.reserve(elements * T::BITS);
    }

    /// Creates a new peapod with enough space to add `capacity` elements
    /// without reallocating.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tags: BitVec::with_capacity(capacity * T::BITS),
            data: Vec::with_capacity(capacity),
        }
    }

    /// Removes all elements from the collection.
    /// **Note**: this does not affect its allocated capacity.
    pub fn clear(&mut self) {
        self.data.clear();
        self.tags.clear();
    }

    /// Shortens the collection so it only contains the first `len` elements.
    /// **Note**: this does not affect its allocated capacity.
    pub fn truncate(&mut self, len: usize) {
        // https://github.com/fprasx/peapod/issues/2
        // len  * T::BITS can overflow so we saturate at the top,
        // If it overflows this means len > max-capacity of the bitvec, 
        // so it would be impossible to reach a state with that many elements.
        // Therefore saturating at the top won't remove anything - which is correct
        self.tags.truncate(usize::saturating_mul(len, T::BITS));
        self.data.truncate(len);
    }

    /// Returns the number of elements the collection can hold
    /// without reallocating.
    pub fn capacity(&self) -> usize {
        let tag_cap = self.tags.capacity() / T::BITS;
        let data_cap = self.data.capacity();
        cmp::min(tag_cap, data_cap)
    }

    /// Adds `other` to the end of `self`, so the new collection
    /// now contains all the elements of `self` followed by the elements
    /// of `other`.
    pub fn append(&mut self, other: Peapod<T>) {
        self.extend(other.into_iter());
    }

    fn cleave(self) -> (BitVec, Vec<T::Value>) {
        let levitating = ManuallyDrop::new(self);
        unsafe {
            (
                // # Safety
                // We are reading from a reference,
                // we have wrapped self in ManuallyDrop to prevent a double-free
                ptr::read(&levitating.tags),
                ptr::read(&levitating.data),
            )
        }
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
    fn from(v: Vec<T>) -> Self {
        let mut pp = Peapod::with_capacity(v.len());
        pp.extend(v.into_iter());
        pp
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
        // Are we done iterating?
        if self.index == self.data.len() {
            None
        } else {
            // # Safety
            // We are reading the tag that matches the data
            let elem = unsafe {
                Some(<T as Phenotype>::reknit(
                    self.tags[self.index * T::BITS..(self.index + 1) * T::BITS].load(),
                    // Read a value out of the vector
                    // # Safety
                    // We are reading from a valid ptr (as_ptr), and the offset is
                    // in bounds as we stop iterating once index == len
                    ptr::read(self.data.as_ptr().add(self.index)),
                ))
            };
            self.index += 1;
            elem
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.data.len() - self.index,
            Some(self.data.len() - self.index),
        )
    }
}

impl<T> DoubleEndedIterator for IntoIter<T>
where
    T: Phenotype,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let len = self.data.len();

        // Are we done iterating?
        if self.index == len {
            None
        } else {
            // Reduce self.data's length by one so the last element won't be accessible,
            // preventing a double-free it were to be read again
            unsafe {
                self.data.set_len(len - 1);
            }

            // # Safety
            // The tag matches the data
            unsafe {
                Some(<T as Phenotype>::reknit(
                    self.tags[(len - 1) * T::BITS..len * T::BITS].load(),
                    // Read a value out of the vector
                    // # Safety
                    // We are reading from a valid ptr (as_ptr), and the offset is
                    // in bounds as we stop iterating once index == len
                    ptr::read(self.data.as_ptr().add(len - 1)),
                ))
            }
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

impl<T> Drop for IntoIter<T>
where
    T: Phenotype,
{
    fn drop(&mut self) {
        for _ in self {}
        // When the drop glue for self drops self.data,
        // nothing get's dropped as the union fields are wrapped
        // in ManuallyDrop, so all the dropping happens when we
        // iterate over self, and we avoid a double-free
    }
}

impl<T> Debug for Peapod<T>
where
    T: Phenotype,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        } else if let (len, None) = iter.size_hint() {
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
    use crate::peapod;
    use core::iter::{DoubleEndedIterator, Iterator};
    use phenotype_macro::Phenotype;

    #[derive(Phenotype, PartialEq, Debug)]
    enum TestData {
        A { u: usize, f: f64 },
        B(usize, f64),
        C,
    }

    #[test]
    fn exact_size_iterator() {
        let pp = peapod![
            TestData::A { u: 1, f: 1.0 },
            TestData::B(1, 1.0),
            TestData::A { u: 1, f: 1.0 },
            TestData::B(1, 1.0),
            TestData::A { u: 1, f: 1.0 },
            TestData::B(1, 1.0),
            TestData::A { u: 1, f: 1.0 }
        ];
        let mut pp = pp.into_iter();
        assert_eq!(pp.len(), 7);
        assert_eq!(pp.size_hint(), (7, Some(7)));
        pp.next();
        assert_eq!(pp.len(), 6);
        assert_eq!(pp.size_hint(), (6, Some(6)));
        for _ in 0..10 {
            pp.next();
        }
        assert_eq!(pp.len(), 0);
        assert_eq!(pp.size_hint(), (0, Some(0)));
    }

    #[test]
    fn double_ended_iterator() {
        let pp = peapod![
            TestData::A { u: 1, f: 1.0 },
            TestData::B(1, 1.0),
            TestData::A { u: 1, f: 1.0 },
            TestData::B(1, 1.0),
            TestData::A { u: 1, f: 1.0 },
            TestData::B(1, 1.0),
            TestData::A { u: 1, f: 1.0 }
        ];
        let mut pp = pp.into_iter();
        assert_eq!(pp.next_back(), Some(TestData::A { u: 1, f: 1.0 }));
        assert_eq!(pp.next(), Some(TestData::A { u: 1, f: 1.0 }));
        assert_eq!(pp.next_back(), Some(TestData::B(1, 1.0)));
        assert_eq!(pp.next(), Some(TestData::B(1, 1.0)));
        assert_eq!(pp.next_back(), Some(TestData::A { u: 1, f: 1.0 }));
        assert_eq!(pp.next(), Some(TestData::A { u: 1, f: 1.0 }));
        assert_eq!(pp.next_back(), Some(TestData::B(1, 1.0)));
        assert_eq!(pp.next(), None);
        assert_eq!(pp.next_back(), None);
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
