// TODO: tests!
#![feature(int_log)]
#![feature(test)]

use bitvec::prelude::*;
use phenotype_internal::Phenotype;

pub struct Peapod<T: Phenotype> {
    // TODO; remove pubs
    pub tags: BitVec,
    pub data: Vec<Option<T::Value>>,
}

impl<T> Peapod<T>
where
    T: Phenotype,
{
    // TODO: stick this in proc macro, use floats to compute it
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

    fn get_tag(&self, index: usize) -> usize {
        self.tags[index * Peapod::<T>::BITS..(index + 1) * Peapod::<T>::BITS].load()
    }

    fn set_tag(&mut self, index: usize, tag: usize) {
        self.tags[index * Peapod::<T>::BITS..(index + 1) * Peapod::<T>::BITS].store::<usize>(tag);
    }

    pub fn push(&mut self, t: T) {
        let (tag, data) = t.cleave();
        let pos = self.data.len();
        self.data.push(data);
        // Naively pushing seems to be faster
        // self.tags
        //     .extend_from_bitslice(&BitView::view_bits::<Lsb0>(&[tag])[0..Self::BITS]);
        for _ in 0..Self::BITS {
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

        // Subtract one as pos is the length of the vector before removing an element
        // The subtraction will not underflow as pos is guaranteed != 0
        let tag: usize = self.get_tag(len - 1);

        for _ in 0..Self::BITS {
            self.tags.pop();
        }

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
        self.tags.reserve(elements * Self::BITS);
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.tags.clear();
    }
}

impl<T> From<Peapod<T>> for Vec<T>
where
    T: Phenotype,
{
    fn from(pp: Peapod<T>) -> Self {
        pp.tags
            .chunks(Peapod::<T>::BITS)
            .zip(pp.data)
            .map(|(tag, val)| Phenotype::reknit(tag[0..Peapod::<T>::BITS].load(), val))
            .collect()
    }
}

impl<T> From<Vec<T>> for Peapod<T>
where
    T: Phenotype,
{
    fn from(pp: Vec<T>) -> Self {
        let mut tags = BitVec::<usize, Lsb0>::repeat(false, Self::BITS * pp.len());
        let mut data = Vec::with_capacity(pp.len());
        for (index, (tag, value)) in pp.into_iter().map(|p| p.cleave()).enumerate() {
            tags[index * Self::BITS..(index + 1) * Self::BITS].store::<usize>(tag);
            data.push(value)
        }
        Self { tags, data }
    }
}

// TODO: impl common traits: Debug, Iter, IntoIter, sliceindex, etc.
impl<T> Default for Peapod<T>
where
    T: Phenotype,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for Peapod<T>
where
    T: Phenotype,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

#[cfg(test)]
mod bench {
    extern crate test;
    use phenotype_macro::Phenotype;
    use test::black_box;
    use test::Bencher;

    use crate::Peapod;

    #[derive(Phenotype)]
    enum Test {
        A(usize, u32),
        B { f: f64, u: (u32, u32) },
        C,
    }

    #[bench]
    fn normal(b: &mut Bencher) {
        let mut v = black_box(vec![
            Test::A(1, 2),
            Test::B { f: 1.0, u: (1, 2) },
            Test::C,
            Test::A(1, 2),
            Test::C,
            Test::B { f: 1.0, u: (1, 2) },
            Test::A(1, 2),
            Test::C,
            Test::B { f: 1.0, u: (1, 2) },
            Test::A(1, 2),
            Test::C,
            Test::B { f: 1.0, u: (1, 2) },
            Test::A(1, 2),
            Test::B { f: 1.0, u: (1, 2) },
            Test::C,
        ]);
        v.reserve(5);
        b.iter(|| {
            v.push(Test::C);
            v.push(Test::B { f: 1.0, u: (1, 2) });
            v.push(Test::A(1, 2));
            v.push(Test::B { f: 1.0, u: (1, 2) });
            v.push(Test::C);
        });
        v.clear();
    }

    #[bench]
    fn peapod(b: &mut Bencher) {
        let mut pp = black_box({
            let mut pp = Peapod::new();
            pp.push(Test::A(1, 2));
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::C);
            pp.push(Test::A(1, 2));
            pp.push(Test::C);
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::A(1, 2));
            pp.push(Test::C);
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::A(1, 2));
            pp.push(Test::C);
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::A(1, 2));
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::C);
            pp
        });
        pp.reserve(5);
        b.iter(|| {
            pp.push(Test::C);
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::A(1, 2));
            pp.push(Test::B { f: 1.0, u: (1, 2) });
            pp.push(Test::C);
        });
        pp.clear();
    }
}
