#![no_std]

use model_mapper::Mapper;

// 1. Struct mapping with renames, options, skipping, and custom conversion
#[derive(Debug, PartialEq, Eq)]
struct TargetStruct {
    a: i32,
    b: Option<bool>,
    renamed_field: i32,
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from, into, ty = "TargetStruct")]
struct SourceStruct {
    a: i32,

    #[mapper(opt)]
    b: Option<bool>,

    #[mapper(rename = "renamed_field")]
    field_c: i32,

    #[mapper(skip(default(value = "default_str()")))]
    only_in_source: &'static str,
}

fn default_str() -> &'static str {
    "default"
}

// 2. Enum mapping with variant rename and skip
#[derive(Debug, PartialEq, Eq)]
enum TargetEnum {
    VariantA,
    VariantB(i32),
}

impl Default for TargetEnum {
    fn default() -> Self {
        TargetEnum::VariantA
    }
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from, into, ty = "TargetEnum")]
enum SourceEnum {
    #[mapper(rename = "VariantA")]
    A,
    #[mapper(rename = "VariantB")]
    B(i32),
    #[mapper(skip(default))]
    C,
}

// 3. Fallible conversions (TryFrom / TryInto) in no_std with custom error
#[derive(Debug, PartialEq, Eq)]
struct FallibleTarget {
    value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CustomError;

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(
    try_from(err = CustomError),
    try_into(err = CustomError),
    ty = FallibleTarget
)]
struct FallibleSource {
    #[mapper(err = CustomError)]
    value: NonNegativeInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NonNegativeInt(i32);

impl TryFrom<i32> for NonNegativeInt {
    type Error = CustomError;

    fn try_from(val: i32) -> Result<Self, Self::Error> {
        if val >= 0 {
            Ok(NonNegativeInt(val))
        } else {
            Err(CustomError)
        }
    }
}

impl TryFrom<NonNegativeInt> for i32 {
    type Error = CustomError;

    fn try_from(val: NonNegativeInt) -> Result<Self, Self::Error> {
        Ok(val.0)
    }
}

#[cfg(no_std_testing)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn main() {
    // Verify Struct Mapping
    let target_struct = TargetStruct {
        a: 42,
        b: Some(true),
        renamed_field: 100,
    };
    let source_struct = SourceStruct::from(target_struct);
    assert_eq!(source_struct.a, 42);
    assert_eq!(source_struct.b, Some(true));
    assert_eq!(source_struct.field_c, 100);
    assert_eq!(source_struct.only_in_source, "default");

    // Verify Enum Mapping
    let target_enum = TargetEnum::VariantB(10);
    let source_enum = SourceEnum::from(target_enum);
    assert_eq!(source_enum, SourceEnum::B(10));

    // Verify TryFrom / TryInto
    let fallible_target = FallibleTarget { value: 5 };
    let fallible_source = FallibleSource::try_from(fallible_target).expect("try_from conversion should succeed");
    assert_eq!(fallible_source.value, NonNegativeInt(5));
}
