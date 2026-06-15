//! Integration tests verifying struct mappings.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use std::{
    convert::{TryFrom, TryInto},
    path::PathBuf,
};

use model_mapper::Mapper;

// ====================================================================================================================
// Custom Error Setup
// ====================================================================================================================

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("conversion error")]
struct CustomError;

impl From<std::convert::Infallible> for CustomError {
    fn from(_: std::convert::Infallible) -> Self {
        CustomError
    }
}

impl From<std::num::TryFromIntError> for CustomError {
    fn from(_: std::num::TryFromIntError) -> Self {
        CustomError
    }
}

// ====================================================================================================================
// 1. Named-field structs mapping directions (from, into, try_from, try_into)
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamedTarget {
    a: i32,
    b: String,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, into, ty = NamedTarget)]
struct NamedSource {
    a: i32,
    b: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamedTargetFallible {
    a: i32,
    b: String,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(try_from(err = CustomError), try_into(err = CustomError), ty = NamedTargetFallible)]
struct NamedSourceFallible {
    a: i32,
    b: String,
}

// ====================================================================================================================
// 2. Tuple / unnamed-field structs mapping directions
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct TupleTarget(i32, String);

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, into, ty = TupleTarget)]
struct TupleSource(i32, String);

#[derive(Debug, Clone, PartialEq, Eq)]
struct TupleTargetFallible(i32, String);

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(try_from(err = CustomError), try_into(err = CustomError), ty = TupleTargetFallible)]
struct TupleSourceFallible(i32, String);

// ====================================================================================================================
// 3. Newtype structs mapping directions
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct NewtypeTarget(String);

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, into, ty = NewtypeTarget)]
struct NewtypeSource(String);

#[derive(Debug, Clone, PartialEq, Eq)]
struct NewtypeTargetFallible(String);

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(try_from(err = CustomError), try_into(err = CustomError), ty = NewtypeTargetFallible)]
struct NewtypeSourceFallible(String);

// ====================================================================================================================
// 4. Implicit Into widening (i32 -> i64, &str -> String, String -> PathBuf)
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct WideningTarget {
    value_i64: i64,
    name: String,
    path: PathBuf,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(into, ty = WideningTarget)]
struct WideningSource<'a> {
    value_i64: i32,
    name: &'a str,
    path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NarrowingTarget {
    value_i32: i32,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(try_into(err = CustomError), ty = NarrowingTarget)]
struct NarrowingSource {
    value_i32: i64,
}

// ====================================================================================================================
// 5. Same-type mapping (Self -> Self)
// ====================================================================================================================

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from(custom), into(custom), ty = SameType)]
struct SameType {
    a: i32,
    b: String,
}

// ====================================================================================================================
// Tests
// ====================================================================================================================

#[test]
fn test_named_fields() {
    let target = NamedTarget {
        a: 42,
        b: "hello".to_string(),
    };
    let source = NamedSource::from(target.clone());
    assert_eq!(source.a, 42);
    assert_eq!(source.b, "hello");

    let target2: NamedTarget = source.into();
    assert_eq!(target2, target);

    let target_fallible = NamedTargetFallible {
        a: 100,
        b: "world".to_string(),
    };
    let source_fallible =
        NamedSourceFallible::try_from(target_fallible.clone()).expect("try_from conversion should succeed");
    assert_eq!(source_fallible.a, 100);
    assert_eq!(source_fallible.b, "world");

    let target_fallible2: NamedTargetFallible = source_fallible.try_into().expect("try_into conversion should succeed");
    assert_eq!(target_fallible2, target_fallible);
}

#[test]
fn test_tuple_fields() {
    let target = TupleTarget(42, "hello".to_string());
    let source = TupleSource::from(target.clone());
    assert_eq!(source.0, 42);
    assert_eq!(source.1, "hello");

    let target2: TupleTarget = source.into();
    assert_eq!(target2, target);

    let target_fallible = TupleTargetFallible(100, "world".to_string());
    let source_fallible =
        TupleSourceFallible::try_from(target_fallible.clone()).expect("try_from conversion should succeed");
    assert_eq!(source_fallible.0, 100);
    assert_eq!(source_fallible.1, "world");

    let target_fallible2: TupleTargetFallible = source_fallible.try_into().expect("try_into conversion should succeed");
    assert_eq!(target_fallible2, target_fallible);
}

#[test]
fn test_newtype_structs() {
    let target = NewtypeTarget("hello".to_string());
    let source = NewtypeSource::from(target.clone());
    assert_eq!(source.0, "hello");

    let target2: NewtypeTarget = source.into();
    assert_eq!(target2, target);

    let target_fallible = NewtypeTargetFallible("world".to_string());
    let source_fallible =
        NewtypeSourceFallible::try_from(target_fallible.clone()).expect("try_from conversion should succeed");
    assert_eq!(source_fallible.0, "world");

    let target_fallible2: NewtypeTargetFallible =
        source_fallible.try_into().expect("try_into conversion should succeed");
    assert_eq!(target_fallible2, target_fallible);
}

#[test]
fn test_implicit_into_widening() {
    let source = WideningSource {
        value_i64: 42,
        name: "test",
        path: "/tmp/test".to_string(),
    };
    let target: WideningTarget = source.into();
    assert_eq!(target.value_i64, 42i64);
    assert_eq!(target.name, "test".to_string());
    assert_eq!(target.path, PathBuf::from("/tmp/test"));
}

#[test]
fn test_implicit_try_into_narrowing() {
    let source_ok = NarrowingSource { value_i32: 42i64 };
    let target: NarrowingTarget = source_ok.try_into().expect("try_into conversion should succeed");
    assert_eq!(target.value_i32, 42i32);

    let source_fail = NarrowingSource { value_i32: i64::MAX };
    let err = NarrowingTarget::try_from(source_fail)
        .expect_err("should fail to convert NarrowingSource when i64 value is out of range for i32");
    assert_eq!(err, CustomError);
}

#[test]
fn test_same_type_mapping() {
    let original = SameType {
        a: 123,
        b: "same".to_string(),
    };
    let copy = SameType::from_same_type(original.clone());
    assert_eq!(copy, original);

    let copy2 = copy.into_same_type();
    assert_eq!(copy2, original);
}

#[test]
fn test_round_trip_validation() {
    let source = NamedSource {
        a: 555,
        b: "round-trip".to_string(),
    };
    // Source -> Target -> Source
    let target: NamedTarget = source.clone().into();
    let round_tripped = NamedSource::from(target);
    assert_eq!(round_tripped, source);
}
