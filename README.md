# Peapod

`Peapod` is a data structure for storing `enum`s super-compactly, like peas in a
pod. It works with any `enum` that implements the `Phenotype` trait, which
captures the behaviour of each variant.

## Contents

1. [Motivation](#motivation)
1. [Usage](#Usage)
1. [Technical](#Tecnical)
1. [How `Peapod` works](#how-does-it-do-it)
1. [When not to use `Peapod`](#when-not-to-use-peapod)

## Motivation

We only have so much memory to work with. Especially in space-constrained
systems, we want to be particularly efficient. `Peapod` provides a way of
storing `enums` that can dramatically reduce space usage. You can read more
in-depth about the motivation in [technical](#technical) section.

tl;dr: `Peapod` provides ultra-compact storage for `enum`s!

## Usage

You can basically just use `Peapod` like a normal `Vec`. Some functionality is
impossible though, like iterating over a `Peapod` without consuming it.

To make an `enum` suitable for `Peapod` storage, stick a `#[derive(Phenotype)]`
on it.

```rust
use peapod::{Phenotype, Peapod};

fn main() {
    // The Peapod representation is a lot smaller!
    // These numbers are in bytes
    assert_eq!(ILovePeas::PEAPOD_SIZE.unwrap(), 9);
    assert_eq!(std::mem::size_of::<ILovePeas>(), 16);

    let mut pp = Peapod::new();
    pp.push(ILovePeas::SnowPea);
    pp.push(ILovePeas::Edamame(0x9EA90D));
    pp.push(ILovePeas::GeneticPea {
        wrinkled: true,
        yellow: true,
    });

    for pea in pp {
        // do something with pea!
    }
}

#[derive(Phenotype)] // <- this is where the magic happens
enum ILovePeas {
    Edamame(usize),
    SnowPea,
    GeneticPea { wrinkled: bool, yellow: bool },
}
```

## Technical

`enum`s (also known as tagged unions) are represented in memory by a tag
(integer) and a `union`. The tag specifies how the bits of the `union` are
interpreted. For example, a tag of 0 might mean "read the `union` as
`Result::Ok(_)`", while a tag of 1 would mean "read the `union` as
`Result::Err(_)`".

Because of alignment reasons, the compiler has to lay out enums so that the tag
takes up a more space than need be. If there are only two variants, we only need
one _bit_ to keep track of which variant something is. Take this pretty drastic
example:

```rust
enum Two {
    First(usize),
    Second(usize)
}
// mem::size_of::<Two> == 16
```

Since the size of each variant is 8 bytes, and the size of the `enum` is 16
bytes, **8 bytes** are being used for the tag! 63 bits are being wasted! We can
do better.

`Peapod` works by "cleaving" an enum into tag and `union`. Tags are stored
together in a `bitvec` type so that no space is wasted due to alignment. All the
data from the `enum`s (in `union` form) is also stored together.

This drawing illustrates the previous example:

```
Scale: 1 - == 1 byte

Standard:
+--------+--------+
|  tag   |  data  |
+--------+--------+
        ^ Only this byte is actually needed to store the tag

Standard array:
+--------+--------+--------+--------+--------+--------+
|  tag   |  data  |  tag   |  data  |  tag   |  data  | . . .
+--------+--------+--------+--------+--------+--------+

Peapod:
+-+--------+
| |  data  |
+-+--------+
 ^ tag

Peapod array:
+-+   +--------+--------+--------+
| | + |  data  |  data  |  data  | . . .
+-+   +--------+--------+--------+
 ^ many tags can be packed into one byte, we could hold 5 more tags in this byte
```

## How does it do it?

_Preface_: compiler people I beg your forgiveness.

The magic is in the `Phenotype` trait, which has two very important methods:
`cleave` and `reknit`.

```rust
type Value;
fn cleave(self) -> (usize, Self::Value)
fn reknit(tag: usize, value: Self::Value) -> Self
```

The type `Value` is some type that can hold all the data from each `enum`
variant. It should be a union.

`cleave` tags a concrete instance of an `enum` and splits it into a tag (this
tag is internal to `Phenotype`, unrelated to the compiler's) and a
`Self::Value`. `reknit` does the opposite and takes a tag and a `Self::Value`,
and reconstitutes it into an `enum` variant.

The implementation all happens with the wizardry that is proc-macros.
`#[derive(Phenotype)]` is the workhorse of this project.

The `#[derive(Phenotype)]` takes a look at your enum and first generates some
"auxilliary" types like so:

```rust
enum ThreeTypes<T> {
    NamedFields {
        one: T,
        two: usize
    },
    Tuple(usize, usize),
    Empty
}

// Represents the `NamedFields` variant
struct __PhenotypeInternalThreeTypesNamedFieldsData<T> {
    one: T,
    two: usize,
}

// Represents the `Tuple` variant
struct __PhenotypeInternalThreeTypesTupleData(usize, usize);

#[allow(non_snake_case)]
union __PhenotypeInternalThreeTypesData<T> {
    NamedFields: ManuallyDrop<__PhenotypeInternalThreeTypesNamedFieldsData<T>>,
    Tuple: ManuallyDrop<__PhenotypeInternalThreeTypesTupleData>,
    Empty: (),
}
```

Then, it generates the `cleave` method. The generated code for this example
looks like:

```rust
fn cleave(self) -> (usize, Self::Value) {
    match &*ManuallyDrop::new(self) {
        ThreeTypes::Empty => (2usize, __PhenotypeInternalThreeTypesData { Empty: () }),
        ThreeTypes::Tuple(_0, _1) => (
            1usize,
            __PhenotypeInternalThreeTypesData {
                Tuple: ManuallyDrop::new(__PhenotypeInternalThreeTypesTupleData(
                    unsafe { ::core::ptr::read(_0) },
                    unsafe { ::core::ptr::read(_1) },
                )),
            },
        ),
        ThreeTypes::NamedFields { one, two } => (
            0usize,
            __PhenotypeInternalThreeTypesData {
                NamedFields: ManuallyDrop::new(__PhenotypeInternalThreeTypesNamedFieldsData::<
                    T,
                > {
                    one: unsafe { ::core::ptr::read(one) },
                    two: unsafe { ::core::ptr::read(two) },
                }),
            },
        ),
    }
}
```

All we're doing is `match`ing on the `enum` variant and reading out each field
into the correct auxiliary struct.

`cleave` does the opposite. Based on the tag, it reads the `union` and generates
and `enum` variant from the data contained in the auxiliary `struct`.

```rust
fn reknit(tag: usize, value: Self::Value) -> ThreeTypes<T> {
    match tag {
        2usize => ThreeTypes::Empty,
        1usize => {
            let data =
                ManuallyDrop::<__PhenotypeInternalThreeTypesTupleData>::into_inner(unsafe {
                    value.Tuple
                });
            ThreeTypes::Tuple(data.0, data.1)
        }
        0usize => {
            let data =
                ManuallyDrop::<__PhenotypeInternalThreeTypesNamedFieldsData<T>>::into_inner(
                    unsafe { value.NamedFields },
                );
            ThreeTypes::NamedFields {
                one: data.one,
                two: data.two,
            }
        }
        _ => unreachable!(),
    }
}
```

## When not to use `Peapod`

-   Sometimes `enums` are niche optimized, meaning the compiler has found a
    clever way to elide the tag. The canonical example is `Option<NonNull<T>>`:
    since the `NonNull<T>` cannot be null, the compiler can use the null pointer
    to represent the `None` variant. This is fine as the `None` variant doesn't
    actually contain a `NonNull<T>`. In summary, an valid pointer bit pattern
    represents a `Some` variant, and the null pointer represents the `None`
    variant, so there is no need to store a tag.
-   Sometimes `Peapod` won't produce a smaller representation. You can check
    this using the provided `IS_MORE_COMPACT` constant.
-   You don't have an allocator. I'm working on a fixed-size `Peapod` but it
    seems like it's going to be difficult as long as `const` generics are
    incomplete.

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](https://github.com/fprasx/peapod/blob/main/LICENSE-APACHE)
    or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](https://github.com/fprasx/peapod/blob/main/LICENSE-MIT) or
    http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
