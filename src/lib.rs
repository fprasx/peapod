// TODO: tests!
#![feature(test)]
use bitvec::prelude::*;
use std::{fmt::Debug, mem::ManuallyDrop, ptr};

pub use phenotype_internal::Phenotype;

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

    pub fn clear(&mut self) {
        self.data.clear();
        self.tags.clear();
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
        let pp = ManuallyDrop::new(self);
        let tags = unsafe { ptr::read(&pp.tags) };
        let data = unsafe { ptr::read(&pp.data) };
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
                unsafe { std::ptr::read(self.data.as_ptr().add(self.index)) },
            ));
            self.index += 1;
            elem
        }
    }
}

impl<T> Debug for Peapod<T>
where
    T: Phenotype,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Peapod")
            .field("tags", &[..])
            .field("data", &[..])
            .finish()
    }
}
