//!
//! This crate provides convenient low-level utility functions for reinterpreting arrays of data.
//! This includes `Vec`s and `slice`s. These functions are intrinsically fragile and but are very
//! useful in performance critical code as they avoid additional copies.
//!
//! The goal of this crate is to provide explicit safe wrappers for unsafe operations that prevent
//! memory leaks and undefined behaviour. When the user misuses a function, it will panic.
//!
//! # Examples
//!
//! ```rust
//! # extern crate reinterpret;
//! # use reinterpret::*;
//! # fn main() {
//!     let points: Vec<[f64;2]> = vec![
//!         [0.1, 1.0],
//!         [1.2, 1.4],
//!         [0.5, 3.2],
//!     ];
//!     let coordinates: Vec<f64> = vec![0.1, 1.0, 1.2, 1.4, 0.5, 3.2];
//!
//!     let point_coordinates: &[f64] = reinterpret_slice(&points);
//!     assert_eq!(*point_coordinates, *coordinates.as_slice()); // Same data.
//!     assert_eq!(point_coordinates, coordinates.as_slice()); // Same location in memory.
//!
//!     let coordinate_points: &[[f64;2]] = reinterpret_slice(&coordinates);
//!     assert_eq!(*coordinate_points, *points.as_slice()); // Same data.
//!     assert_eq!(coordinate_points, points.as_slice()); // Same location in memory.
//! # }
//! ```
//!
//! # Caveats
//!
//! There are still ways to misuse these functions without causing panics that may produce
//! unexpected (but not undefined) behaviour. For instance:
//!
//! ```rust
//! # extern crate reinterpret;
//! # use reinterpret::*;
//! # fn main() {
//! let data = vec![1.0f32; 4];
//! let garbage: &[f64] = reinterpret_slice(&data);
//! # }
//! ```
//! 
//! This is valid because a `&[f64]` can snuggly fit the data stored in `data` since 2 `f64`s
//! occupy the same amount of memory as 4 `f32`s.
//!
//! It is the users responsibility to avoid these types of scenarios.
//!

use std::mem::size_of;
use std::slice;

/// Reinterpret a given slice as a slice of another type. This function checks that the resulting
/// slice is appropriately sized.
pub fn reinterpret_mut_slice<T, S>(slice: &mut [T]) -> &mut [S] {
    // We must be able to split the given slice into appropriately sized chunks.
    assert_eq!((slice.len() * size_of::<T>()) % size_of::<S>(), 0,
                "Slice cannot be safely reinterpreted due to a misaligned size");
    let nu_len = (slice.len() * size_of::<T>()) / size_of::<S>();
    unsafe { slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut S, nu_len) }
}

/// Reinterpret a given slice as a slice of another type. This function checks that the resulting
/// slice is appropriately sized.
pub fn reinterpret_slice<T, S>(slice: &[T]) -> &[S] {
    // We must be able to split the given slice into appropriately sized chunks.
    assert_eq!((slice.len() * size_of::<T>()) % size_of::<S>(), 0,
               "Slice cannot be safely reinterpreted due to a misaligned size");
    let nu_len = (slice.len() * size_of::<T>()) / size_of::<S>();
    unsafe { slice::from_raw_parts(slice.as_ptr() as *const S, nu_len) }
}

/// Reinterpret a given `Vec` as a `Vec` of another type. This function checks that the resulting
/// `Vec` is appropriately sized.
pub fn reinterpret_vec<T, S>(mut vec: Vec<T>) -> Vec<S> {
    // We must be able to split the given vec into appropriately sized chunks.
    assert_eq!((vec.len() * size_of::<T>()) % size_of::<S>(), 0,
               "Vec cannot be safely reinterpreted due to a misaligned size");
    let nu_len = (vec.len() * size_of::<T>()) / size_of::<S>();
    assert_eq!((vec.capacity() * size_of::<T>()) % size_of::<S>(), 0,
               "Vec cannot be safely reinterpreted due to a misaligned capacity");
    let nu_capacity = (vec.capacity() * size_of::<T>()) / size_of::<S>();
    let vec_ptr = vec.as_mut_ptr();
    ::std::mem::forget(vec);
    unsafe { Vec::from_raw_parts(vec_ptr as *mut S, nu_len, nu_capacity) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check that we can reinterpret a slice of `[f64;3]`s as a slice of `f64`s.
    #[test]
    fn reinterpret_slice_test() {
        let vec: Vec<[f64;3]> = vec![
            [0.1, 1.0, 2.0],
            [1.2, 1.4, 2.1],
            [0.5, 3.2, 4.0],
        ];
        let flat: Vec<f64> = vec![0.1, 1.0, 2.0, 1.2, 1.4, 2.1, 0.5, 3.2, 4.0];
        let nu_flat: &[f64] = reinterpret_slice(vec.as_slice());
        assert_eq!(*nu_flat, *flat.as_slice()); // Same data.
        assert_eq!(nu_flat, flat.as_slice()); // Same memory.

        let nu_slice: &[[f64;3]] = reinterpret_slice(flat.as_slice());
        assert_eq!(*nu_slice, *vec.as_slice()); // Same data.
        assert_eq!(nu_slice, vec.as_slice()); // Same memory.
    }

    #[test]
    fn reinterpret_slice_mut_test() {
        let vec: Vec<[f64;3]> = vec![
            [0.5, 1.0, 2.0],
            [1.2, 1.4, 2.1],
            [0.5, 3.2, 4.0],
        ];
        let flat_mut = &mut [-0.5, -1.0, -1.0, 0.2, -0.6, -0.9, -0.5, 1.2, 1.0];

        let nu_mut_slice: &mut [[f64;3]] = reinterpret_mut_slice(flat_mut);
        for v in nu_mut_slice.iter_mut() {
            v[0] += 1.0;
            v[1] += 2.0;
            v[2] += 3.0;
        }

        assert_eq!(nu_mut_slice, vec.as_slice());
    }

    #[test]
    fn reinterpret_vec_test() {
        let exp_vec: Vec<[f64;3]> = vec![
            [0.5, 1.0, 2.0],
            [1.2, 1.4, 2.1],
            [0.5, 3.2, 4.0],
        ];
        let vec = vec![-0.5, -1.0, -1.0, 0.2, -0.6, -0.9, -0.5, 1.2, 1.0];
        let mut nu_vec: Vec<[f64;3]> = reinterpret_vec(vec.clone());
        for v in nu_vec.iter_mut() {
            v[0] += 1.0;
            v[1] += 2.0;
            v[2] += 3.0;
        }

        assert_eq!(nu_vec, exp_vec);
    }
}
