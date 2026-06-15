//! Integration tests verifying renaming logic.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use model_mapper::Mapper;

// ====================================================================================================================
// 1. Struct field renaming using `#[mapper(rename = ...)]`
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleStructTarget {
    target_field_a: String,
    target_field_b: i32,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, into, ty = SimpleStructTarget)]
struct SimpleStructSource {
    #[mapper(rename = target_field_a)]
    source_field_a: String,
    #[mapper(rename = target_field_b)]
    source_field_b: i32,
}

#[test]
fn test_struct_field_renaming() {
    let target = SimpleStructTarget {
        target_field_a: "hello".to_string(),
        target_field_b: 42,
    };
    let source = SimpleStructSource::from(target.clone());
    assert_eq!(source.source_field_a, "hello");
    assert_eq!(source.source_field_b, 42);

    let mapped_target: SimpleStructTarget = source.into();
    assert_eq!(mapped_target, target);
}

// ====================================================================================================================
// 2. Enum variant renaming using `#[mapper(rename = ...)]`
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnumVariantTarget {
    VariantTargetA,
    VariantTargetB(String),
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, into, ty = EnumVariantTarget)]
enum EnumVariantSource {
    #[mapper(rename = VariantTargetA)]
    VariantSourceA,
    #[mapper(rename = VariantTargetB)]
    VariantSourceB(String),
}

#[test]
fn test_enum_variant_renaming() {
    // Target -> Source
    let target_a = EnumVariantTarget::VariantTargetA;
    let source_a = EnumVariantSource::from(target_a);
    assert!(matches!(source_a, EnumVariantSource::VariantSourceA));

    let target_b = EnumVariantTarget::VariantTargetB("test".to_string());
    let source_b = EnumVariantSource::from(target_b);
    if let EnumVariantSource::VariantSourceB(val) = source_b {
        assert_eq!(val, "test");
    } else {
        panic!("Expected VariantSourceB");
    }

    // Source -> Target
    let source_a = EnumVariantSource::VariantSourceA;
    let target_a: EnumVariantTarget = source_a.into();
    assert!(matches!(target_a, EnumVariantTarget::VariantTargetA));

    let source_b = EnumVariantSource::VariantSourceB("hello".to_string());
    let target_b: EnumVariantTarget = source_b.into();
    if let EnumVariantTarget::VariantTargetB(val) = target_b {
        assert_eq!(val, "hello");
    } else {
        panic!("Expected VariantTargetB");
    }
}

// ====================================================================================================================
// 3. Field renaming inside enum named-field variants
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnumNamedFieldTarget {
    Variant { target_name: String, target_val: i32 },
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, into, ty = EnumNamedFieldTarget)]
enum EnumNamedFieldSource {
    Variant {
        #[mapper(rename = target_name)]
        source_name: String,
        #[mapper(rename = target_val)]
        source_val: i32,
    },
}

#[test]
fn test_enum_named_field_renaming() {
    let target = EnumNamedFieldTarget::Variant {
        target_name: "name".to_string(),
        target_val: 100,
    };
    let source = EnumNamedFieldSource::from(target.clone());
    let EnumNamedFieldSource::Variant {
        source_name,
        source_val,
    } = &source;
    assert_eq!(source_name, "name");
    assert_eq!(*source_val, 100);

    let mapped_target: EnumNamedFieldTarget = source.into();
    assert_eq!(mapped_target, target);
}

// ====================================================================================================================
// 4. Multiple renames on the same type
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct MultipleTargetsA {
    field_a: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MultipleTargetsB {
    field_b: String,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(derive(from, into, ty = MultipleTargetsA))]
#[mapper(derive(from, into, ty = MultipleTargetsB))]
struct MultiRenameSource {
    #[mapper(when(ty = MultipleTargetsA, rename = field_a))]
    #[mapper(when(ty = MultipleTargetsB, rename = field_b))]
    value: String,
}

#[test]
fn test_multiple_renames_on_same_type() {
    let src = MultiRenameSource {
        value: "multi".to_string(),
    };

    // Into Target A
    let a: MultipleTargetsA = src.clone().into();
    assert_eq!(a.field_a, "multi");

    // Into Target B
    let b: MultipleTargetsB = src.clone().into();
    assert_eq!(b.field_b, "multi");

    // From Target A
    let from_a = MultiRenameSource::from(MultipleTargetsA {
        field_a: "from_a".to_string(),
    });
    assert_eq!(from_a.value, "from_a");

    // From Target B
    let from_b = MultiRenameSource::from(MultipleTargetsB {
        field_b: "from_b".to_string(),
    });
    assert_eq!(from_b.value, "from_b");
}

// ====================================================================================================================
// 5. Edge Case: Conflicting rename + skip on the same field
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetConflict {
    renamed_field: String,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(from, ty = TargetConflict, add(field = renamed_field, default))]
struct SourceConflict {
    #[mapper(rename = renamed_field, skip(default(value = "skipped".to_string())))]
    some_field: String,
}

#[test]
fn test_conflicting_rename_and_skip() {
    let target = TargetConflict {
        renamed_field: "original".to_string(),
    };
    let source = SourceConflict::from(target);
    // Since skip takes precedence, some_field should be "skipped", not "original"
    assert_eq!(source.some_field, "skipped");
}
