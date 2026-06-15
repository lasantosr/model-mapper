//! Integration tests verifying mapping a single source type to multiple target types.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use model_mapper::Mapper;

// ====================================================================================================================
// Scenario 1: Struct mapping to multiple targets using separate #[mapper(...)] blocks
// ====================================================================================================================

#[derive(Debug, PartialEq, Eq, Clone)]
struct TargetA {
    x: i32,
    y: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TargetB {
    x: i32,
    extra_b: i32,
}

#[derive(Mapper, Debug, Clone)]
#[mapper(derive(into, ty = TargetA))]
#[mapper(derive(into, ty = TargetB, add(field = extra_b, default(value = 100))))]
struct SourceSeparateBlocks {
    x: i32,
    // only target TargetA has y, so we skip it for TargetB or only map when TargetA.
    // Actually, target-specific field options using when:
    #[mapper(when(ty = TargetB, skip))]
    y: String,
}

#[test]
fn test_separate_blocks_and_when_skip() {
    let src = SourceSeparateBlocks {
        x: 42,
        y: "hello".to_string(),
    };

    let target_a: TargetA = src.clone().into();
    assert_eq!(
        target_a,
        TargetA {
            x: 42,
            y: "hello".to_string()
        }
    );

    let target_b: TargetB = src.into();
    assert_eq!(target_b, TargetB { x: 42, extra_b: 100 });
}

// ====================================================================================================================
// Scenario 2: Struct mapping using multiple `derive(...)` properties inside a single #[mapper(...)] attribute
// ====================================================================================================================

#[derive(Debug, PartialEq, Eq, Clone)]
struct TargetC {
    id: i64,
    name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TargetD {
    id: i64,
    title: String,
}

#[derive(Mapper, Debug, Clone)]
#[mapper(
    derive(from, into, ty = TargetC),
    derive(from, into, ty = TargetD)
)]
struct SourceSingleBlock {
    id: i64,
    // rename to title only when mapping to/from TargetD
    #[mapper(when(ty = TargetD, rename = title))]
    name: String,
}

#[test]
fn test_single_block_multi_derive_and_rename() {
    let src = SourceSingleBlock {
        id: 1,
        name: "test".to_string(),
    };

    // Test Into
    let c: TargetC = src.clone().into();
    assert_eq!(
        c,
        TargetC {
            id: 1,
            name: "test".to_string()
        }
    );

    let d: TargetD = src.clone().into();
    assert_eq!(
        d,
        TargetD {
            id: 1,
            title: "test".to_string()
        }
    );

    // Test From
    let from_c = SourceSingleBlock::from(TargetC {
        id: 2,
        name: "from_c".to_string(),
    });
    assert_eq!(from_c.id, 2);
    assert_eq!(from_c.name, "from_c");

    let from_d = SourceSingleBlock::from(TargetD {
        id: 3,
        title: "from_d".to_string(),
    });
    assert_eq!(from_d.id, 3);
    assert_eq!(from_d.name, "from_d");
}

// ====================================================================================================================
// Scenario 3: Target-specific overrides at variant/field levels in Enums
// ====================================================================================================================

#[derive(Debug, PartialEq, Eq, Clone)]
enum EnumTargetA {
    Variant1,
    Variant2 { val: i32 },
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum EnumTargetB {
    Other1,
    Other2 { value: i32 },
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(derive(from, into, ty = EnumTargetA))]
#[mapper(derive(from, into, ty = EnumTargetB))]
enum SourceEnum {
    #[mapper(when(ty = EnumTargetB, rename = Other1))]
    Variant1,

    #[mapper(when(ty = EnumTargetB, rename = Other2))]
    Variant2 {
        #[mapper(when(ty = EnumTargetB, rename = value))]
        val: i32,
    },
}

#[test]
fn test_enum_multi_target_overrides() {
    // Source -> TargetA
    let src_v1 = SourceEnum::Variant1;
    let target_a_v1: EnumTargetA = src_v1.clone().into();
    assert_eq!(target_a_v1, EnumTargetA::Variant1);

    let src_v2 = SourceEnum::Variant2 { val: 42 };
    let target_a_v2: EnumTargetA = src_v2.clone().into();
    assert_eq!(target_a_v2, EnumTargetA::Variant2 { val: 42 });

    // Source -> TargetB
    let target_b_v1: EnumTargetB = src_v1.into();
    assert_eq!(target_b_v1, EnumTargetB::Other1);

    let target_b_v2: EnumTargetB = src_v2.into();
    assert_eq!(target_b_v2, EnumTargetB::Other2 { value: 42 });

    // TargetA -> Source
    let from_a_v1 = SourceEnum::from(EnumTargetA::Variant1);
    assert_eq!(from_a_v1, SourceEnum::Variant1);

    let from_a_v2 = SourceEnum::from(EnumTargetA::Variant2 { val: 10 });
    assert_eq!(from_a_v2, SourceEnum::Variant2 { val: 10 });

    // TargetB -> Source
    let from_b_v1 = SourceEnum::from(EnumTargetB::Other1);
    assert_eq!(from_b_v1, SourceEnum::Variant1);

    let from_b_v2 = SourceEnum::from(EnumTargetB::Other2 { value: 20 });
    assert_eq!(from_b_v2, SourceEnum::Variant2 { val: 20 });
}

// ====================================================================================================================
// Scenario 4: Mixed attributes: default settings combined with `when`-specific overrides
// ====================================================================================================================

#[derive(Debug, PartialEq, Eq, Clone)]
struct SourceA {
    id: i64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct SourceB {
    id: i64,
}

#[derive(Mapper, Debug, Clone, PartialEq, Eq)]
#[mapper(derive(from, ty = SourceA))]
#[mapper(derive(from, ty = SourceB))]
struct TargetWithDefaults {
    id: i64,

    // Use target-specific configurations via when for both SourceA and SourceB.
    #[mapper(when(ty = SourceA, skip(default(value = "default_val".to_string()))))]
    #[mapper(when(ty = SourceB, skip(default(value = "override_val".to_string()))))]
    val: String,
}

#[test]
fn test_mixed_defaults_and_overrides() {
    let src_a = SourceA { id: 10 };
    let target_from_a = TargetWithDefaults::from(src_a);
    assert_eq!(
        target_from_a,
        TargetWithDefaults {
            id: 10,
            val: "default_val".to_string(),
        }
    );

    let src_b = SourceB { id: 20 };
    let target_from_b = TargetWithDefaults::from(src_b);
    assert_eq!(
        target_from_b,
        TargetWithDefaults {
            id: 20,
            val: "override_val".to_string(),
        }
    );
}
