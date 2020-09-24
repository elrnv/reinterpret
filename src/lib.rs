//!
//! This crate provides convenient low-level utility functions for reinterpreting data.
//! This includes `Vec`s and `slice`s. These functions are intrinsically unsafe but are very
//! useful in performance critical code since they avoid additional copies.
//!
//! The goal of this crate is to provide some memory safety along the boundaries of `Vec`s and
//! `slice`s to reduce the boilerplate for reinterpreting data.
//!
//! These functions check that the source and target arrays have the same size, however they don't
//! check for safety of converting the contained types.
//!
//! It is possible to write safe wrappers for converting collections of concrete types using these
//! functions, however this is out of the scope of this crate.
//!
//! # Examples
//!
//! ```rust
//! use reinterpret::*;
//! let points: Vec<[f64;2]> = vec![
//!     [0.1, 1.0],
//!     [1.2, 1.4],
//!     [0.5, 3.2],
//! ];
//! let coordinates: Vec<f64> = vec![0.1, 1.0, 1.2, 1.4, 0.5, 3.2];
//!
//! let point_coordinates: &[f64] = unsafe { reinterpret_slice(&points) };
//! assert_eq!(*point_coordinates, *coordinates.as_slice()); // Same data.
//! assert_eq!(point_coordinates, coordinates.as_slice()); // Same location in memory.
//!
//! let coordinate_points: &[[f64;2]] = unsafe { reinterpret_slice(&coordinates) };
//! assert_eq!(*coordinate_points, *points.as_slice()); // Same data.
//! assert_eq!(coordinate_points, points.as_slice()); // Same location in memory.
//! ```
//!
//! # Panics
//!
//! Panics can occur in surprising circumstances when converting `Vec`s.
//! The rule of thumb is to ensure that the element size of the target `Vec` is *smaller than or
//! equal to* the source element size.
//!
//! To illustrate this consider converting between a `Vec` of `u8` and a `Vec` of `u16`. The
//! following code will not cause panics for any size of the source `Vec`.
//!
//! ```
//! use reinterpret::*;
//! let vec_u16 = vec![1u16,2,3,4,5];
//! let vec_u8: Vec<u8> = unsafe { reinterpret_vec(vec_u16) };
//! ```
//!
//! However converting to a `Vec<[u8; 3]>` can cause a panic if the size of the source capacity
//! cannot be divided into the number of bytes of a `[u8; 3]`. For example the following code will
//! panic because the capacity of `vec_u8` after adding an additional 4 bytes becomes 8, which is
//! not divisible by 3 and so the resulting `Vec` cannot be converted  to a `Vec<[u8; 3]>`.
//!
//! ```should_panic
//! # use reinterpret::*;
//! # let vec_u16 = vec![1u16];
//! # let mut vec_u8: Vec<u8> = unsafe { reinterpret_vec(vec_u16) };
//! vec_u8.extend(vec![1,2,3,4]);
//! let vec_u16_round_trip: Vec<[u8; 3]> = unsafe { reinterpret_vec(vec_u8) };
//! ```
//!
//! There is no way of ensuring that this type of conversion will succeed, however we can try to
//! improve the situation by calling `shrink_to_fit` on the source `Vec` and hope that this will
//! produce a capacity equal to the length of the `Vec` as follows: 
//!
//! ```
//! # use reinterpret::*;
//! # let vec_u16 = vec![1u16];
//! # let mut vec_u8: Vec<u8> = unsafe { reinterpret_vec(vec_u16) };
//! vec_u8.extend(vec![1,2,3,4]);
//! vec_u8.shrink_to_fit();
//! let vec_u16_round_trip: Vec<[u8; 3]> = unsafe { reinterpret_vec(vec_u8) };
//! ```
//!
//! This approach relies on the specifics of `Vec` implementations and is heavily discouraged in library
//! code.
//!
//! # Undefined Behavior
//!
//! There are ways to misuse these functions without causing panics that may produce *undefined
//! behavior*. For instance:
//!
//! ```rust
//! use reinterpret::*;
//! let a = 1;
//! let b = 2;
//! let v = vec![&a, &b];
//! let mut m: Vec<&mut usize> = unsafe { reinterpret_vec(v) };
//! *m[0] = 100; // Mutating an immutable variable a!
//!
//! assert_eq!(a, 100);
//! ```
//!
//! It is the users' responsibility to avoid these types of scenarios.

use std::mem::size_of;
use std::slice;

/// Reinterpret a given slice as a slice of another type. This function checks that the resulting
/// slice is appropriately sized.
pub unsafe fn reinterpret_mut_slice<T, S>(slice: &mut [T]) -> &mut [S] {
    let size_t = size_of::<T>();
    let size_s = size_of::<S>();
    let nu_len = if size_t > 0 {
        assert_ne!(
            size_s, 0,
            "Cannot reinterpret a slice of non-zero sized types as a slice of zero sized types."
        );
        // We must be able to split the given slice into appropriately sized chunks.
        assert_eq!(
            (slice.len() * size_t) % size_s,
            0,
            "Slice cannot be safely reinterpreted due to a misaligned size"
        );
        (slice.len() * size_t) / size_s
    } else {
        assert_eq!(
            size_s, 0,
            "Cannot reinterpret a slice of zero sized types as a slice of non-zero sized types."
        );
        slice.len()
    };
    slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut S, nu_len)
}

/// Reinterpret a given slice as a slice of another type. This function checks that the resulting
/// slice is appropriately sized.
pub unsafe fn reinterpret_slice<T, S>(slice: &[T]) -> &[S] {
    let size_t = size_of::<T>();
    let size_s = size_of::<S>();
    let nu_len = if size_t > 0 {
        assert_ne!(
            size_s, 0,
            "Cannot reinterpret a slice of non-zero sized types as a slice of zero sized types."
        );
        // We must be able to split the given slice into appropriately sized chunks.
        assert_eq!(
            (slice.len() * size_t) % size_s,
            0,
            "Slice cannot be safely reinterpreted due to a misaligned size"
        );
        (slice.len() * size_t) / size_s
    } else {
        assert_eq!(
            size_s, 0,
            "Cannot reinterpret a slice of zero sized types as a slice of non-zero sized types."
        );
        slice.len()
    };
    slice::from_raw_parts(slice.as_ptr() as *const S, nu_len)
}

/// Reinterpret a given `Vec` as a `Vec` of another type. This function checks that the resulting
/// `Vec` is appropriately sized.
pub unsafe fn reinterpret_vec<T, S>(mut vec: Vec<T>) -> Vec<S> {
    let size_t = size_of::<T>();
    let size_s = size_of::<S>();
    let nu_vec = if size_t > 0 {
        assert_ne!(
            size_s, 0,
            "Cannot reinterpret a Vec of non-zero sized types as a Vec of zero sized types."
        );
        // We must be able to split the given vec into appropriately sized chunks.
        assert_eq!(
            (vec.len() * size_t) % size_s,
            0,
            "Vec cannot be safely reinterpreted due to a misaligned size"
        );
        let nu_len = (vec.len() * size_t) / size_s;
        assert_eq!(
            (vec.capacity() * size_t) % size_s,
            0,
            "Vec cannot be safely reinterpreted due to a misaligned capacity"
        );
        let nu_capacity = (vec.capacity() * size_t) / size_s;
        let vec_ptr = vec.as_mut_ptr();
        Vec::from_raw_parts(vec_ptr as *mut S, nu_len, nu_capacity)
    } else {
        assert_eq!(
            size_s, 0,
            "Cannot reinterpret a Vec of zero sized types as a Vec of non-zero sized types."
        );
        let nu_len = vec.len();
        let nu_capacity = vec.capacity();
        debug_assert_eq!(
            nu_capacity,
            (-1isize) as usize,
            "Capacity should be -1 for 0 sized types. (bug)"
        );
        let vec_ptr = vec.as_mut_ptr();
        Vec::from_raw_parts(vec_ptr as *mut S, nu_len, nu_capacity)
    };
    ::std::mem::forget(vec);
    nu_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check that we can reinterpret a slice of `[f64;3]`s as a slice of `f64`s.
    #[test]
    fn reinterpret_slice_test() {
        let vec: Vec<[f64; 3]> = vec![[0.1, 1.0, 2.0], [1.2, 1.4, 2.1], [0.5, 3.2, 4.0]];
        let flat: Vec<f64> = vec![0.1, 1.0, 2.0, 1.2, 1.4, 2.1, 0.5, 3.2, 4.0];
        let nu_flat: &[f64] = unsafe { reinterpret_slice(vec.as_slice()) };
        assert_eq!(*nu_flat, *flat.as_slice()); // Same data.
        assert_eq!(nu_flat, flat.as_slice()); // Same memory.

        let nu_slice: &[[f64; 3]] = unsafe { reinterpret_slice(flat.as_slice()) };
        assert_eq!(*nu_slice, *vec.as_slice()); // Same data.
        assert_eq!(nu_slice, vec.as_slice()); // Same memory.
    }

    #[test]
    fn reinterpret_mut_slice_test() {
        let vec: Vec<[f64; 3]> = vec![[0.5, 1.0, 2.0], [1.2, 1.4, 2.1], [0.5, 3.2, 4.0]];
        let flat_mut = &mut [-0.5, -1.0, -1.0, 0.2, -0.6, -0.9, -0.5, 1.2, 1.0];

        let nu_mut_slice: &mut [[f64; 3]] = unsafe { reinterpret_mut_slice(flat_mut) };
        for v in nu_mut_slice.iter_mut() {
            v[0] += 1.0;
            v[1] += 2.0;
            v[2] += 3.0;
        }

        assert_eq!(nu_mut_slice, vec.as_slice());
    }

    #[test]
    fn reinterpret_vec_test() {
        let exp_vec: Vec<[f64; 3]> = vec![[0.5, 1.0, 2.0], [1.2, 1.4, 2.1], [0.5, 3.2, 4.0]];
        let vec = vec![-0.5, -1.0, -1.0, 0.2, -0.6, -0.9, -0.5, 1.2, 1.0];
        let mut nu_vec: Vec<[f64; 3]> = unsafe { reinterpret_vec(vec.clone()) };
        for v in nu_vec.iter_mut() {
            v[0] += 1.0;
            v[1] += 2.0;
            v[2] += 3.0;
        }

        assert_eq!(nu_vec, exp_vec);
    }

    /// Test reinterpreting collections of zero size structs.
    #[test]
    fn zero_size_test() {
        #[derive(Debug, Clone, PartialEq)]
        struct Foo {
            a: (),
            b: (),
        }

        let exp_vec: Vec<Foo> = vec![Foo { a: (), b: () }, Foo { a: (), b: () }];
        let mut mut_vec = vec![(), ()];
        let vec = mut_vec.clone();
        let mut_slice = mut_vec.as_mut_slice();
        let slice = vec.as_slice();
        let exp_slice = exp_vec.as_slice();

        // Convert to a collection of Foo.
        let nu_vec: Vec<Foo> = unsafe { reinterpret_vec(vec.clone()) };
        assert_eq!(nu_vec, exp_vec);

        let nu_slice: &[Foo] = unsafe { reinterpret_slice(slice) };
        assert_eq!(nu_slice, exp_slice);

        let nu_mut_slice: &mut [Foo] = unsafe { reinterpret_mut_slice(mut_slice) };
        assert_eq!(nu_mut_slice, exp_slice);

        // Convert back to a collection of ().
        let old_vec: Vec<()> = unsafe { reinterpret_vec(nu_vec.clone()) };
        assert_eq!(vec, old_vec);

        let old_slice: &[()] = unsafe { reinterpret_mut_slice(nu_mut_slice) };
        assert_eq!(slice, old_slice);
    }
}
