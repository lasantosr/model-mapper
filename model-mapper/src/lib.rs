#![cfg_attr(not(feature = "std"), no_std)]

// Re-export derive macro crate
#[allow(unused_imports)]
#[macro_use]
extern crate model_mapper_macros;
#[doc(hidden)]
pub use model_mapper_macros::*;
