//! Integration tests verifying `ignore_extra` features.

#![allow(dead_code, unused, clippy::restriction, clippy::infallible_try_from, reason = "test")]

use model_mapper::Mapper;

// ====================================================================================================================
// Struct-level `ignore_extra` on `into`
// ====================================================================================================================

#[derive(Debug, PartialEq, Default)]
struct TargetStructInto {
    field1: i32,
    field2: String,
    extra_field: Option<String>,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(into, ty = TargetStructInto, ignore_extra)]
struct SourceStructInto {
    field1: i32,
    field2: String,
}

#[test]
fn test_struct_ignore_extra_into() {
    let src = SourceStructInto {
        field1: 42,
        field2: "hello".to_string(),
    };
    let target = TargetStructInto::from(src);
    assert_eq!(target.field1, 42);
    assert_eq!(target.field2, "hello".to_string());
    assert_eq!(target.extra_field, None);
}

// ====================================================================================================================
// Struct-level `ignore_extra` on `from`
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct SourceStructFrom {
    field1: i32,
    field2: String,
    extra_field: bool,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, ty = SourceStructFrom, ignore_extra)]
struct TargetStructFrom {
    field1: i32,
    field2: String,
}

#[test]
fn test_struct_ignore_extra_from() {
    let src = SourceStructFrom {
        field1: 100,
        field2: "world".to_string(),
        extra_field: true,
    };
    let target = TargetStructFrom::from(src);
    assert_eq!(target.field1, 100);
    assert_eq!(target.field2, "world".to_string());
}

// ====================================================================================================================
// Enum-level `ignore_extra` on `into`
// ====================================================================================================================

#[derive(Debug, PartialEq)]
enum TargetEnumInto {
    One,
    Two,
    ExtraThree,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(into, ty = TargetEnumInto, ignore_extra)]
enum SourceEnumInto {
    One,
    Two,
}

#[test]
fn test_enum_ignore_extra_into() {
    let src = SourceEnumInto::One;
    let target = TargetEnumInto::from(src);
    assert_eq!(target, TargetEnumInto::One);

    let src2 = SourceEnumInto::Two;
    let target2 = TargetEnumInto::from(src2);
    assert_eq!(target2, TargetEnumInto::Two);
}

// ====================================================================================================================
// Enum-level `ignore_extra` on `from`
// ====================================================================================================================

#[derive(Debug, PartialEq)]
enum SourceEnumFrom {
    One,
    Two,
    ExtraThree,
}

#[derive(Mapper, Debug, PartialEq, Default)]
#[mapper(from, ty = SourceEnumFrom, ignore_extra)]
enum TargetEnumFrom {
    #[default]
    One,
    Two,
}

#[test]
fn test_enum_ignore_extra_from() {
    let src = SourceEnumFrom::One;
    let target = TargetEnumFrom::from(src);
    assert_eq!(target, TargetEnumFrom::One);

    let src2 = SourceEnumFrom::Two;
    let target2 = TargetEnumFrom::from(src2);
    assert_eq!(target2, TargetEnumFrom::Two);

    let src3 = SourceEnumFrom::ExtraThree;
    let target3 = TargetEnumFrom::from(src3);
    assert_eq!(target3, TargetEnumFrom::One);
}

// ====================================================================================================================
// Mixed behavior: `ignore_extra` combined with explicit type-level `add` fields/variants
// ====================================================================================================================

#[derive(Debug, PartialEq, Default)]
struct TargetMixed {
    field1: i32,
    field2: String,
    added_field: i64,
    extra_field: Option<String>,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(
    into,
    ty = TargetMixed,
    ignore_extra,
    add(field = added_field, default(value = 999))
)]
struct SourceMixed {
    field1: i32,
    field2: String,
}

#[test]
fn test_struct_mixed_ignore_extra_and_add() {
    let src = SourceMixed {
        field1: 42,
        field2: "hello".to_string(),
    };
    let target = TargetMixed::from(src);
    assert_eq!(target.field1, 42);
    assert_eq!(target.field2, "hello".to_string());
    assert_eq!(target.added_field, 999);
    assert_eq!(target.extra_field, None);
}

#[derive(Debug, PartialEq)]
enum SourceEnumMixed {
    One,
    Two,
    Three,
    Four,
}

#[derive(Mapper, Debug, PartialEq, Default)]
#[mapper(
    from,
    ty = SourceEnumMixed,
    ignore_extra,
    add(field = Three, default(value = TargetEnumMixed::Two))
)]
enum TargetEnumMixed {
    One,
    Two,
    #[mapper(skip(default))]
    #[default]
    DefaultVariant,
}

#[test]
fn test_enum_mixed_ignore_extra_and_add() {
    let target = TargetEnumMixed::from(SourceEnumMixed::One);
    assert_eq!(target, TargetEnumMixed::One);

    let target2 = TargetEnumMixed::from(SourceEnumMixed::Two);
    assert_eq!(target2, TargetEnumMixed::Two);

    let target3 = TargetEnumMixed::from(SourceEnumMixed::Three);
    assert_eq!(target3, TargetEnumMixed::Two);

    let target4 = TargetEnumMixed::from(SourceEnumMixed::Four);
    assert_eq!(target4, TargetEnumMixed::DefaultVariant);
}

// ====================================================================================================================
// Variant-level `ignore_extra` on `from`/`try_from` enum variant fields
// ====================================================================================================================

#[derive(Debug, PartialEq)]
enum SourceEnumVariantFields {
    Variant { a: i32, b: String, extra_field: bool },
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, ty = SourceEnumVariantFields)]
enum TargetEnumVariantFields {
    #[mapper(ignore_extra)]
    Variant { a: i32, b: String },
}

#[test]
fn test_variant_level_ignore_extra() {
    let src = SourceEnumVariantFields::Variant {
        a: 123,
        b: "hello".to_string(),
        extra_field: true,
    };
    let target = TargetEnumVariantFields::from(src);
    match target {
        TargetEnumVariantFields::Variant { a, b } => {
            assert_eq!(a, 123);
            assert_eq!(b, "hello".to_string());
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
enum TestAppError {
    #[error("Infallible error: {0}")]
    Infallible(#[from] std::convert::Infallible),
}

#[derive(Debug, PartialEq)]
enum SourceEnumVariantFieldsTry {
    Variant { a: i32, b: String, extra_field: bool },
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(try_from(err = TestAppError), ty = SourceEnumVariantFieldsTry)]
enum TargetEnumVariantFieldsTry {
    #[mapper(ignore_extra)]
    Variant { a: i32, b: String },
}

#[test]
fn test_variant_level_ignore_extra_try_from() {
    let src = SourceEnumVariantFieldsTry::Variant {
        a: 456,
        b: "try".to_string(),
        extra_field: false,
    };
    let target = TargetEnumVariantFieldsTry::try_from(src).expect("try_from conversion should succeed");
    match target {
        TargetEnumVariantFieldsTry::Variant { a, b } => {
            assert_eq!(a, 456);
            assert_eq!(b, "try".to_string());
        }
    }
}
