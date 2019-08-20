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


## Examples

```rust
# extern crate reinterpret;
# use reinterpret::*;
# fn main() {
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
# }
```


## Undefined Behavior

There are ways to misuse these functions without causing panics that may produce *undefined
behavior*. For instance:

```rust
# extern crate reinterpret;
# use reinterpret::*;
# fn main() {
    let a = 1;
    let b = 2;
    let v = vec![&a, &b];
    let mut m: Vec<&mut usize> = unsafe { reinterpret_vec(v) };


# License

This repository is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
