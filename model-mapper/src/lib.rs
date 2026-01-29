#![cfg_attr(not(feature = "std"), no_std)]

// Re-export derive macro crate
#[allow(unused_imports)]
#[macro_use]
extern crate model_mapper_macros;
#[doc(hidden)]
pub use model_mapper_macros::*;

#[doc(hidden)]
pub mod private {
    pub trait RefMapper<T, R> {
        fn map_value(&self, arg: T) -> R;
    }
    impl<F, T, R> RefMapper<T, R> for F
    where
        F: ?Sized + Fn(&T) -> R,
    {
        #[inline(always)]
        fn map_value(&self, arg: T) -> R {
            (self)(&arg)
        }
    }

    pub trait ValueMapper<T, R> {
        fn map_value(&self, arg: T) -> R;
    }
    impl<F, T, R> ValueMapper<T, R> for &F
    where
        F: ?Sized + Fn(T) -> R,
    {
        #[inline(always)]
        fn map_value(&self, arg: T) -> R {
            (*self)(arg)
        }
    }
}
