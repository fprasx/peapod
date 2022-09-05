use bitvec::prelude::*;
use bitvec::BitArr;
use core::mem::MaybeUninit;
use phenotype_internal::Phenotype;

pub struct FixedPod<T: Phenotype, const N: usize> {
    // TODO: calculate const in macro
    // 32 * (1 + T::BITS), supposing T has 3 variants
    // assuming N = 32
    tags_init: BitArr!(for 69),
    data: [MaybeUninit<T::Value>; N],
    len: usize,
}

impl<T, const N: usize> FixedPod<T, N>
where
    T: Phenotype,
{
    const UNINIT: MaybeUninit<T::Value> = MaybeUninit::uninit();

    pub fn new() -> Self {
        Self {
            tags_init: BitArray::ZERO,
            data: [Self::UNINIT; N],
            len: 0,
        }
    }

    fn len(&self) -> usize {
        if let Some(index) = self.tags_init[..32].first_one() {
            index + 1
        } else {
            0
        }
    }

    // return value indicates success or failure
    pub fn push(&mut self, elem: T) -> bool {
        let len = self.len();
        if len == N {
            return false;
        }
        let (tag, data) = elem.cleave();
        if self.check_init(len) {
            self.data[len].write(data);
            // We initialized the element at len'th spot, so mark it wth true
            self.set_tag(len, tag);
            // We initialized the element at len'th spot, so mark it wth true
            self.set_init(len);
            todo!()
        } else {
            todo!()
        }
    }

    // Will overwrite last element
    pub fn force_push(&mut self, elem: T) {
        todo!()
    }

    // **Note**: index must be in range
    fn check_init(&self, index: usize) -> bool {
        self.tags_init[index]
    }

    // **Note**: index must be in range
    fn set_init(&mut self, index: usize) {
        self.tags_init.set(index, true);
    }

    // **Note**: index must be in range
    fn get_tag(&self, index: usize) -> usize {
        // offset by 32 as the first 32 bits indicate init-ness
        self.tags_init[N + index * T::BITS..N + (index + 1) * T::BITS].load()
    }

    // **Note**: index must be in range
    fn set_tag(&mut self, index: usize, tag: usize) {
        // offset by 32 as the first 32 bits indicate init-ness
        self.tags_init[N + index * T::BITS..N + (index + 1) * T::BITS].store(tag);
    }
}

impl<T, const N: usize> Default for FixedPod<T, N>
where
    T: Phenotype,
{
    fn default() -> Self {
        Self::new()
    }
}

// struct FixedPod<T, const N: usize>
// where
//     [(); N / 64]:,
// {
//     tags_init: [usize; N / 64],
//     ts: [usize; N],
//     _boo: PhantomData<T>,
// }

// impl<T, const N: usize> FixedPod<T, N> {
//     fn new() -> Self {
//         Self {
//             tags_init: 0,
//             ts: [1; N],
//             _boo: PhantomData,
//         }
//     }
// }
