//! Integration tests verifying fallible conversions and error handling/accumulation.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use std::{
    any::Any,
    convert::{TryFrom, TryInto},
};

use model_mapper::Mapper;

// ====================================================================================================================
// Struct Definitions for Mapping
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Source {
    a: i32,
    b: String,
}

// Short-circuiting
#[derive(Debug, Mapper, PartialEq, Eq)]
#[mapper(try_from(err = MyError), try_into(err = MyError), ty = Source)]
struct ShortCircuit {
    #[mapper(err = MyError::IntErr)]
    a: NonNegativeInt,
    #[mapper(err_with = MyError::NonEmptyString)]
    b: NonEmptyString,
}

// Accumulating
#[derive(Debug, Mapper, PartialEq, Eq)]
#[mapper(try_from(err = MyError, accumulate), try_into(err = MyError, accumulate), ty = Source)]
struct Accumulate {
    a: NonNegativeInt,
    #[mapper(err_with = |err: EmptyStringError| MyError::StrErr(err.to_string()))]
    b: NonEmptyString,
}

// Custom accumulator try_from
#[derive(Debug, Mapper, PartialEq, Eq)]
#[mapper(try_from(err = MyError, accumulate = MyAccumulator), ty = Source)]
struct CustomAccumulateTryFrom {
    a: NonNegativeInt,
    #[mapper(err_with = |e: EmptyStringError| MyError::StrErr(e.to_string()))]
    b: NonEmptyString,
}

// Default error fallback (anyhow::Error)
#[derive(Debug, Mapper, PartialEq, Eq)]
#[mapper(try_from, ty = Source)]
struct DefaultErrorTryFrom {
    a: NonNegativeInt,
    b: NonEmptyString,
}

// ====================================================================================================================
// Setup Custom Types & Error Types
// ====================================================================================================================

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("negative int")]
struct NegativeIntError;

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("empty string")]
struct EmptyStringError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct NonNegativeInt(i32);

impl TryFrom<i32> for NonNegativeInt {
    type Error = NegativeIntError;

    fn try_from(val: i32) -> Result<Self, Self::Error> {
        if val >= 0 {
            Ok(NonNegativeInt(val))
        } else {
            Err(NegativeIntError)
        }
    }
}

impl TryFrom<NonNegativeInt> for i32 {
    type Error = NegativeIntError;

    fn try_from(val: NonNegativeInt) -> Result<Self, Self::Error> {
        if val.0 >= 0 { Ok(val.0) } else { Err(NegativeIntError) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NonEmptyString(String);

impl TryFrom<String> for NonEmptyString {
    type Error = EmptyStringError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        if val.is_empty() {
            Err(EmptyStringError)
        } else {
            Ok(NonEmptyString(val))
        }
    }
}

impl TryFrom<NonEmptyString> for String {
    type Error = EmptyStringError;

    fn try_from(val: NonEmptyString) -> Result<Self, Self::Error> {
        if val.0.is_empty() {
            Err(EmptyStringError)
        } else {
            Ok(val.0)
        }
    }
}

// Error Enum for mappings
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
enum MyError {
    #[error("int error")]
    IntErr,
    #[error("string error: {0}")]
    StrErr(String),
    #[error(transparent)]
    NonNegativeInt(#[from] NegativeIntError),
    #[error("{0}")]
    NonEmptyString(#[source] EmptyStringError),
}

// Custom error accumulator struct
#[derive(Debug, PartialEq, Eq)]
struct MyAccumulator(Vec<MyError>);

impl FromIterator<MyError> for MyAccumulator {
    fn from_iter<T: IntoIterator<Item = MyError>>(iter: T) -> Self {
        MyAccumulator(iter.into_iter().collect())
    }
}

// ====================================================================================================================
// Tests
// ====================================================================================================================

#[test]
fn test_short_circuit_try_from() {
    // Happy path
    let src = Source {
        a: 42,
        b: "hello".to_string(),
    };
    let target = ShortCircuit::try_from(src).expect("try_from conversion should succeed");
    assert_eq!(target.a, NonNegativeInt(42));
    assert_eq!(target.b, NonEmptyString("hello".to_string()));

    // First field fails: returns error immediately
    let src_fail1 = Source {
        a: -5,
        b: String::new(),
    };
    let err = ShortCircuit::try_from(src_fail1).expect_err("should fail to convert from invalid Source (IntErr)");
    assert_eq!(err, MyError::IntErr);

    // Second field fails (first is okay)
    let src_fail2 = Source {
        a: 10,
        b: String::new(),
    };
    let err =
        ShortCircuit::try_from(src_fail2).expect_err("should fail to convert from invalid Source (NonEmptyString)");
    assert!(matches!(err, MyError::NonEmptyString(_)));
}

#[test]
fn test_short_circuit_try_into() {
    // Happy path
    let target = ShortCircuit {
        a: NonNegativeInt(42),
        b: NonEmptyString("hello".to_string()),
    };
    let src: Source = target.try_into().expect("try_into conversion should succeed");
    assert_eq!(src.a, 42);
    assert_eq!(src.b, "hello");

    // First field fails: returns error immediately
    let target_fail1 = ShortCircuit {
        a: NonNegativeInt(-5),
        b: NonEmptyString(String::new()),
    };
    let err = Source::try_from(target_fail1).expect_err("should fail to convert from invalid Target (IntErr)");
    assert_eq!(err, MyError::IntErr);

    // Second field fails
    let target_fail2 = ShortCircuit {
        a: NonNegativeInt(10),
        b: NonEmptyString(String::new()),
    };
    let err = Source::try_from(target_fail2).expect_err("should fail to convert from invalid Target (NonEmptyString)");
    assert!(matches!(err, MyError::NonEmptyString(_)));
}

#[test]
fn test_accumulate_try_from() {
    let src_all_fail = Source {
        a: -5,
        b: String::new(),
    };
    let errs = Accumulate::try_from(src_all_fail).expect_err("should accumulate conversion errors from invalid Source");
    assert_eq!(errs.len(), 2);
    assert!(matches!(errs[0], MyError::NonNegativeInt(_)));
    assert_eq!(errs[1], MyError::StrErr("empty string".to_string()));
}

#[test]
fn test_accumulate_try_into() {
    let target_all_fail = Accumulate {
        a: NonNegativeInt(-5),
        b: NonEmptyString(String::new()),
    };
    let errs = Source::try_from(target_all_fail).expect_err("should accumulate conversion errors from invalid Target");
    assert_eq!(errs.len(), 2);
    assert!(matches!(errs[0], MyError::NonNegativeInt(_)));
    assert_eq!(errs[1], MyError::StrErr("empty string".to_string()));
}

#[test]
fn test_custom_accumulate_try_from() {
    let src_all_fail = Source {
        a: -5,
        b: String::new(),
    };
    let MyAccumulator(errs) = CustomAccumulateTryFrom::try_from(src_all_fail)
        .expect_err("should accumulate conversion errors using CustomAccumulator");
    assert_eq!(errs.len(), 2);
    assert!(matches!(errs[0], MyError::NonNegativeInt(_)));
    assert_eq!(errs[1], MyError::StrErr("empty string".to_string()));
}

#[test]
fn test_default_error_fallback() {
    let src_fail = Source {
        a: -5,
        b: "ok".to_string(),
    };
    let err = DefaultErrorTryFrom::try_from(src_fail)
        .expect_err("should fail to convert from invalid Source using default error fallback");
    // Verify it is indeed an anyhow::Error
    assert_eq!(err.to_string(), "negative int");
    assert_eq!(err.type_id(), std::any::TypeId::of::<anyhow::Error>());
    assert!(err.downcast_ref::<NegativeIntError>().is_some());
}
