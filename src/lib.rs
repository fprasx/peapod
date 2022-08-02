#![feature(int_log)]
use bitvec::prelude::*;
use phenotype_internal::Phenotype;

pub struct Peapod<T: Phenotype> {
    pub tags: BitVec,
    data: Vec<Option<T::Value>>,
}

#[allow(dead_code)]
impl<T> Peapod<T>
where
    T: Phenotype,
{
    // TODO: some kind of check to make sure the enum has vairants
    const BITS: usize = {
        let log = usize::log2(T::NUM_VARIANTS);
        let pow = 2usize.pow(log);
        (if pow < T::NUM_VARIANTS { log + 1 } else { log }) as usize
    };

    pub fn new() -> Self {
        Peapod {
            tags: BitVec::new(),
            data: Vec::new(),
        }
    }

    pub fn get(&self) -> &T {
        todo!()
    }

    pub fn get_mut(&mut self) -> &mut T {
        todo!()
    }

    // TODO: figure out how to less allocations
    pub fn push(&mut self, t: T) {
        let (tag, data) = t.cleave();
        let pos = self.data.len();
        self.data.push(data);
        // maybe use `repeat`
        self.tags
            .extend_from_bitslice(&bitvec![1; Peapod::<T>::BITS]);
        self.tags[pos * Peapod::<T>::BITS..(pos + 1) * Peapod::<T>::BITS].store::<usize>(tag);
    }

    // TODO: make sure endianness is ok??
    pub fn pop(&mut self) -> Option<T> {
        let pos = self.data.len();
        if pos == 0 {
            return None;
        }
        let data = self.data.pop().unwrap();
        let tag: usize = self.tags[(pos - 1) * Peapod::<T>::BITS..pos * Peapod::<T>::BITS].load();
        // TODO: truncate tags
        Some(Phenotype::reknit(tag, data))
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn reserve(&mut self) {}

    pub fn into_vec(self) -> Vec<T> {
        todo!()
    }
}

// TODO: impl common traits: Debug, Iter, IntoIter, etc.
impl<T> Default for Peapod<T>
where
    T: Phenotype,
{
    fn default() -> Self {
        Self::new()
    }
}
