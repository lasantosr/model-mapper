#![no_std]
#![allow(unused, dead_code, clippy::restriction, reason = "example")]

use model_mapper::Mapper;

// The standard library is not linked, but core/alloc functionality works as expected.
// Below are simple structs and enums to map.

#[derive(Debug, PartialEq, Eq)]
struct SourceStruct {
    value: i32,
    active: Option<bool>,
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from, into, ty = SourceStruct)]
struct TargetStruct {
    value: i32,
    // Hints like `opt` work perfectly in no_std mode
    #[mapper(opt)]
    active: Option<bool>,
}

#[derive(Debug, PartialEq, Eq)]
enum SourceEnum {
    On,
    Off(i32),
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from, into, ty = SourceEnum)]
enum TargetEnum {
    On,
    Off(i32),
}

// Fallible conversions in no_std require specifying a custom error type,
// because the default error type (`anyhow::Error`) requires the standard library.
#[derive(Debug, PartialEq, Eq)]
struct SourceFallible {
    value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MyCustomError;

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(
    try_from(err = MyCustomError),
    try_into(err = MyCustomError),
    ty = SourceFallible
)]
struct TargetFallible {
    // We map any field-level failures to our custom error
    #[mapper(err = MyCustomError)]
    value: NonNegative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NonNegative(i32);

impl TryFrom<i32> for NonNegative {
    type Error = MyCustomError;

    fn try_from(val: i32) -> Result<Self, Self::Error> {
        if val >= 0 {
            Ok(NonNegative(val))
        } else {
            Err(MyCustomError)
        }
    }
}

impl TryFrom<NonNegative> for i32 {
    type Error = MyCustomError;

    fn try_from(val: NonNegative) -> Result<Self, Self::Error> {
        Ok(val.0)
    }
}

fn main() {
    // Structural mapping in no_std
    let source = SourceStruct {
        value: 123,
        active: Some(true),
    };
    let target = TargetStruct::from(source);
    assert_eq!(target.value, 123);
    assert_eq!(target.active, Some(true));

    // Enum mapping in no_std
    let source_enum = SourceEnum::Off(42);
    let target_enum = TargetEnum::from(source_enum);
    assert_eq!(target_enum, TargetEnum::Off(42));

    // Fallible conversions with a custom error in no_std
    let source_fallible = SourceFallible { value: 10 };
    let target_fallible = TargetFallible::try_from(source_fallible);
    assert!(target_fallible.is_ok());
    assert_eq!(
        target_fallible
            .expect("TargetFallible conversion should succeed for non-negative values")
            .value,
        NonNegative(10)
    );
}
