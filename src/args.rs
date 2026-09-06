//! Owned argument lists for TJS dispatch calls.
//!
//! TJS APIs accept a contiguous `tTJSVariant **`, but the variants behind
//! those pointers must remain alive for the duration of the call.  These
//! types keep that ownership together with the pointer list.

use crate::tp_stub::tTJSVariant;
use std::ops::{Deref, DerefMut};

/// An owned, fixed-size list of TJS call arguments.
///
/// The variants are boxed so moving this wrapper cannot invalidate the
/// pointers exposed through [`DerefMut`].
pub struct Tjs2ArgsArray<const N: usize> {
    values: Box<[tTJSVariant; N]>,
    params: [*mut tTJSVariant; N],
}

impl<const N: usize> Tjs2ArgsArray<N> {
    /// Creates an argument list from owned variants.
    pub fn new(values: [tTJSVariant; N]) -> Self {
        let mut values = Box::new(values);
        let params = std::array::from_fn(|index| &mut values[index] as *mut tTJSVariant);
        Self { values, params }
    }

    /// Returns the number of arguments.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether this argument list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<const N: usize> Deref for Tjs2ArgsArray<N> {
    type Target = [*mut tTJSVariant; N];

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

impl<const N: usize> DerefMut for Tjs2ArgsArray<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.params
    }
}

/// An owned, dynamically sized list of TJS call arguments.
pub struct Tjs2ArgsVec {
    values: Vec<tTJSVariant>,
    params: Vec<*mut tTJSVariant>,
}

impl Tjs2ArgsVec {
    /// Creates an argument list from owned variants.
    pub fn new(mut values: Vec<tTJSVariant>) -> Self {
        let params = values
            .iter_mut()
            .map(|value| value as *mut tTJSVariant)
            .collect();
        Self { values, params }
    }

    /// Returns the number of arguments.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether this argument list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Deref for Tjs2ArgsVec {
    type Target = [*mut tTJSVariant];

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

impl DerefMut for Tjs2ArgsVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.params
    }
}
