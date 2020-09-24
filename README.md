# reinterpret

Low level utility functions to reinterpret arrays of data in Rust

[![On crates.io](https://img.shields.io/crates/v/reinterpret.svg)](https://crates.io/crates/reinterpret)
[![On docs.rs](https://docs.rs/reinterpret/badge.svg)](https://docs.rs/reinterpret/)
[![Build status](https://travis-ci.org/elrnv/reinterpret.svg?branch=master)](https://travis-ci.org/elrnv/reinterpret)


# Overview

This crate provides convenient low-level utility functions for reinterpreting data.
This includes `Vec`s and `slice`s. These functions are intrinsically unsafe but are very
useful in performance critical code since they avoid additional copies.

The goal of this crate is to provide some memory safety along the boundaries of `Vec`s and
`slice`s to reduce the boilerplate for reinterpreting data.

These functions check that the source and target arrays have the same size, however they don't
check for safety of converting the contained types.

It is possible to write safe wrappers for converting collections of concrete types using these
functions, however this is out of the scope of this crate.


# Examples

```rust
use reinterpret::*;
let points: Vec<[f64;2]> = vec![
    [0.1, 1.0],
    [1.2, 1.4],
    [0.5, 3.2],
];
let coordinates: Vec<f64> = vec![0.1, 1.0, 1.2, 1.4, 0.5, 3.2];

let point_coordinates: &[f64] = unsafe { reinterpret_slice(&points) };
assert_eq!(*point_coordinates, *coordinates.as_slice()); // Same data.
assert_eq!(point_coordinates, coordinates.as_slice()); // Same location in memory.

let coordinate_points: &[[f64;2]] = unsafe { reinterpret_slice(&coordinates) };
assert_eq!(*coordinate_points, *points.as_slice()); // Same data.
assert_eq!(coordinate_points, points.as_slice()); // Same location in memory.
```

# Panics

Panics can occur in surprising circumstances when converting `Vec`s.
The rule of thumb is to ensure that the element size of the target `Vec` is *smaller than or
equal to* the source element size.

To illustrate this consider converting between a `Vec` of `u8` and a `Vec` of `u16`. The
following code will not cause panics for any size of the source `Vec`.

```rust
use reinterpret::*;
let vec_u16 = vec![1u16,2,3,4,5];
let vec_u8: Vec<u8> = unsafe { reinterpret_vec(vec_u16) };
```

However converting to a `Vec<[u8; 3]>` can cause a panic if the size of the source capacity
cannot be divided into the number of bytes of a `[u8; 3]`. For example the following code will
panic because the capacity of `vec_u8` after adding an additional 4 bytes becomes 8, which is
not divisible by 3 and so the resulting `Vec` cannot be converted  to a `Vec<[u8; 3]>`.

```rust
vec_u8.extend(vec![1,2,3,4]);
let vec_u16_round_trip: Vec<[u8; 3]> = unsafe { reinterpret_vec(vec_u8) };
```

There is no way of ensuring that this type of conversion will succeed, however we can try to
improve the situation by calling `shrink_to_fit` on the source `Vec` and hope that this will
produce a capacity equal to the length of the `Vec` as follows: 

```rust
vec_u8.extend(vec![1,2,3,4]);
vec_u8.shrink_to_fit();
let vec_u16_round_trip: Vec<[u8; 3]> = unsafe { reinterpret_vec(vec_u8) };
```

This approach relies on the specifics of `Vec` implementations and is heavily discouraged in library
code.

# Undefined Behavior

There are ways to misuse these functions without causing panics that may produce *undefined
behavior*. For instance:

```rust
use reinterpret::*;
let a = 1;
let b = 2;
let v = vec![&a, &b];
let mut m: Vec<&mut usize> = unsafe { reinterpret_vec(v) };
*m[0] = 100; // Mutating an immutable variable a!

assert_eq!(a, 100);
```

It is the users' responsibility to avoid these types of scenarios.


# License

This repository is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
