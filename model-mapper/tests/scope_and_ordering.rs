//! Integration tests verifying field evaluation, consumption ordering, and variable namespaces/scopes.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use std::convert::{TryFrom, TryInto};

use model_mapper::Mapper;

// ====================================================================================================================
// Variable Scope & Namespaces
// ====================================================================================================================

mod scope_namespaces {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct TargetUser {
        id: i64,
        username: String,
        role: String,
    }

    #[derive(Mapper, Debug)]
    #[mapper(into(custom), ty = TargetUser, add(field = role, ty = String))]
    struct SourceUser {
        #[mapper(rename = id)]
        user_id: i64,
        username: String,
    }

    #[test]
    fn test_added_fields_namespacing() {
        let user = SourceUser {
            user_id: 42,
            username: "bob".to_string(),
        };
        // The added field `role` is wrapped in Input struct.
        let target = user.into_target_user("Admin".to_string());
        assert_eq!(target.id, 42);
        assert_eq!(target.username, "bob");
        assert_eq!(target.role, "Admin");
    }

    #[derive(Debug, PartialEq, Eq)]
    struct CollidingTarget {
        username: String,
        role: String,
    }

    #[derive(Mapper, Debug)]
    #[mapper(into(custom), ty = CollidingTarget, add(field = role, ty = String))]
    struct CollidingSource {
        username: String,
        #[mapper(skip)]
        role: String,
    }

    #[test]
    fn test_skipped_and_added_collision() {
        let src = CollidingSource {
            username: "alice".to_string(),
            role: "skipped_source_role".to_string(),
        };
        // The custom parameter `role` (accessed as `input.role`) is mapped to the target.
        // The skipped field `role` does not collide with the custom parameter.
        let target = src.into_colliding_target("parameter_role".to_string());
        assert_eq!(target.username, "alice");
        assert_eq!(target.role, "parameter_role");
    }
}

// ====================================================================================================================
// Custom expressions (with, from_with, into_with) referencing other fields
// ====================================================================================================================

mod custom_expression_ordering {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Source {
        a: String,
        b: String,
    }

    // --- 1. Infallible From & Fallible TryInto ---
    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(
        from,
        try_into(err = TryError),
        ty = Source,
        add(field = a, default(value = url.split('_').next().ok_or(TryError)?.to_string()))
    )]
    struct Target {
        #[mapper(
            rename = b,
            from_with = format!("{a}_{b}"),
            into_with = url.strip_prefix(&format!("{a}_")).ok_or(TryError)?.to_string(),
            err = TryError
        )]
        url: String,
    }

    // --- 2. Fallible TryFrom & Infallible Into ---
    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_from(err = TryError), into, ty = Source)]
    struct TryTarget {
        a: String,
        #[mapper(
            from_with = {
                if !b.contains('_') {
                    return Err(TryError);
                }
                format!("{a}_{b}")
            },
            into_with = b.strip_prefix(&format!("{a}_")).unwrap_or(&b).to_string(),
            err = TryError
        )]
        b: String,
    }

    // --- 3. Accumulating TryFrom & TryInto ---
    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_from(err = TryError, accumulate), try_into(err = TryError, accumulate), ty = Source)]
    struct TryAccumulateTarget {
        a: String,
        #[mapper(
            from_with = {
                if !b.contains('_') {
                    return Err(vec![TryError]);
                }
                format!("{a}_{b}")
            },
            into_with = b.strip_prefix(&format!("{a}_")).ok_or(vec![TryError])?.to_string(),
            err = TryError
        )]
        b: String,
    }

    #[test]
    fn test_custom_expression_ordering() {
        let src = Source {
            a: "prefix".to_string(),
            b: "suffix".to_string(),
        };

        // Infallible From
        let target = Target::from(src.clone());
        assert_eq!(target.url, "prefix_suffix");

        // Fallible TryInto
        let target_into: Source = target.try_into().expect("try_into conversion should succeed");
        assert_eq!(target_into.a, "prefix");
        assert_eq!(target_into.b, "suffix");
    }
}

// ====================================================================================================================
// Type-level added fields with default expressions referencing other fields
// ====================================================================================================================

mod type_level_add_ordering {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Target {
        a: String,
        b: String,
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(into, ty = Target, add(field = b, default(value = format!("{}_default", a))))]
    struct Source {
        a: String,
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_into(err = TryError), ty = Target, add(field = b, default(value = format!("{}_default", a))))]
    struct TrySource {
        a: String,
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_into(err = TryError, accumulate), ty = Target, add(field = b, default(value = format!("{}_default", a))))]
    struct TryAccumulateSource {
        a: String,
    }

    #[test]
    fn test_type_level_add_defaults() {
        let src = Source { a: "hello".to_string() };
        let target: Target = src.into();
        assert_eq!(target.a, "hello");
        assert_eq!(target.b, "hello_default");
    }
}

// ====================================================================================================================
// Field-level skipped fields with default expressions referencing other fields
// ====================================================================================================================

mod field_level_skip_ordering {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Source {
        a: String,
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(from, ty = Source)]
    struct Target {
        a: String,
        #[mapper(skip(default(value = format!("{a}_default"))))]
        b: String,
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_from(err = TryError), ty = Source)]
    struct TryTarget {
        a: String,
        #[mapper(skip(default(value = format!("{a}_default"))))]
        b: String,
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_from(err = TryError, accumulate), ty = Source)]
    struct TryAccumulateTarget {
        a: String,
        #[mapper(skip(default(value = format!("{a}_default"))))]
        b: String,
    }

    #[test]
    fn test_field_level_skip_defaults() {
        let src = Source { a: "hello".to_string() };
        let target = Target::from(src);
        assert_eq!(target.a, "hello");
        assert_eq!(target.b, "hello_default");
    }
}

// ====================================================================================================================
// Enum variant added fields with provider closures referencing other variant fields
// ====================================================================================================================

mod enum_variant_provider_ordering {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Target {
        Variant { a: String, b: String },
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(into(custom = "to_target"), ty = Target)]
    enum Source {
        #[mapper(add(field = b, ty = String))]
        Variant { a: String },
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_into(custom = "try_to_target", err = TryError), ty = Target)]
    enum TrySource {
        #[mapper(add(field = b, ty = String))]
        Variant { a: String },
    }

    #[derive(Debug, Mapper, PartialEq, Eq)]
    #[mapper(try_into(custom = "try_accumulate_to_target", err = TryError, accumulate), ty = Target)]
    enum TryAccumulateSource {
        #[mapper(add(field = b, ty = String))]
        Variant { a: String },
    }

    #[test]
    fn test_enum_variant_provider_closures() {
        let src = Source::Variant { a: "hello".to_string() };
        let target = src.to_target(|a| format!("{a}_variant"));
        assert_eq!(
            target,
            Target::Variant {
                a: "hello".to_string(),
                b: "hello_variant".to_string(),
            }
        );
    }
}

// A generic try error for all Try mappings
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("try error")]
struct TryError;

impl From<std::convert::Infallible> for TryError {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}
