//! Integration tests verifying mapping behavior for asymmetric types.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use model_mapper::Mapper;

// ====================================================================================================================
// Shared "foreign" types used across multiple test modules
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Source {
    id: i64,
    name: String,
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Target {
    id: i64,
    name: String,
    value: i64,
    extra: i64,
    tag: Option<String>,
}

// ====================================================================================================================
// Struct skipped fields: `skip(default)` and `skip(default(value = expr))`
// ====================================================================================================================

mod struct_skip_default {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Outer {
        a: i64,
        b: String,
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(from, ty = Outer)]
    struct WithSkipDefault {
        a: i64,
        b: String,
        /// Skipped field populated via `Default::default()` → `0_i64`
        #[mapper(skip(default))]
        extra: i64,
        /// Skipped optional field populated via `Default::default()` → None
        #[mapper(skip(default))]
        maybe: Option<String>,
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(from, ty = Outer)]
    struct WithSkipValue {
        a: i64,
        b: String,
        /// Skipped field with explicit value expression
        #[mapper(skip(default(value = 42)))]
        extra: i64,
        /// Skipped field with explicit `Some(...)` value
        #[mapper(skip(default(value = Some("injected".to_string()))))]
        maybe: Option<String>,
    }

    #[test]
    fn skip_default_uses_default_trait() {
        let outer = Outer {
            a: 1,
            b: "hello".into(),
        };
        let mapped = WithSkipDefault::from(outer);
        assert_eq!(mapped.a, 1);
        assert_eq!(mapped.b, "hello");
        assert_eq!(mapped.extra, 0);
        assert_eq!(mapped.maybe, None);
    }

    #[test]
    fn skip_default_value_uses_expression() {
        let outer = Outer {
            a: 7,
            b: "world".into(),
        };
        let mapped = WithSkipValue::from(outer);
        assert_eq!(mapped.a, 7);
        assert_eq!(mapped.b, "world");
        assert_eq!(mapped.extra, 42);
        assert_eq!(mapped.maybe, Some("injected".to_string()));
    }
}

// ====================================================================================================================
// Cross-field skipped default expressions (referencing other source fields)
// ====================================================================================================================

mod cross_field_skip_expressions {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Numbers {
        x: i64,
        y: i64,
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(from, ty = Numbers)]
    struct Computed {
        x: i64,
        y: i64,
        /// sum is populated from source fields x + y
        #[mapper(skip(default(value = x + y)))]
        sum: i64,
        /// product is populated from source fields x * y
        #[mapper(skip(default(value = x * y)))]
        product: i64,
    }

    #[test]
    fn cross_field_expressions_compute_correctly() {
        let nums = Numbers { x: 3, y: 5 };
        let mapped = Computed::from(nums);
        assert_eq!(mapped.x, 3);
        assert_eq!(mapped.y, 5);
        assert_eq!(mapped.sum, 8);
        assert_eq!(mapped.product, 15);
    }

    #[test]
    fn cross_field_with_negative_values() {
        let nums = Numbers { x: -4, y: 10 };
        let mapped = Computed::from(nums);
        assert_eq!(mapped.sum, 6);
        assert_eq!(mapped.product, -40);
    }
}

// ====================================================================================================================
// Custom functions taking skipped fields as additional parameters (`from(custom)` / `into(custom)`)
// ====================================================================================================================

mod custom_fn_with_skipped_fields {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Flat {
        name: String,
        value: i64,
    }

    // --- from(custom): skipped fields become runtime parameters ---

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(from(custom), ty = Flat)]
    struct RichFromCustom {
        name: String,
        value: i64,
        /// Bare `skip` → no default; becomes a runtime parameter of `from_flat`
        #[mapper(skip)]
        audit_note: String,
        /// Another bare skip
        #[mapper(skip)]
        priority: i64,
    }

    // --- from(custom) with mixed skip + skip(default(value = ...)) ---

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(from(custom), ty = Flat)]
    struct RichFromMixed {
        name: String,
        value: i64,
        /// Has a default ⇒ NOT a parameter
        #[mapper(skip(default(value = value * 10)))]
        computed: i64,
        /// Bare skip ⇒ IS a parameter
        #[mapper(skip)]
        label: String,
    }

    // --- into(custom): skipped fields are simply ignored (not mapped) ---

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(into, ty = Flat)]
    struct RichInto {
        name: String,
        value: i64,
        /// Skipped on into – just ignored
        #[mapper(skip)]
        audit_note: String,
    }

    #[test]
    fn from_custom_all_skipped_as_params() {
        let flat = Flat {
            name: "item".into(),
            value: 100,
        };
        let rich = RichFromCustom::from_flat(flat, "created".into(), 5);
        assert_eq!(rich.name, "item");
        assert_eq!(rich.value, 100);
        assert_eq!(rich.audit_note, "created");
        assert_eq!(rich.priority, 5);
    }

    #[test]
    fn from_custom_mixed_default_and_param() {
        let flat = Flat {
            name: "thing".into(),
            value: 7,
        };
        let rich = RichFromMixed::from_flat(flat, "my-label".into());
        assert_eq!(rich.name, "thing");
        assert_eq!(rich.value, 7);
        assert_eq!(rich.computed, 70); // value * 10
        assert_eq!(rich.label, "my-label");
    }

    #[test]
    fn into_ignores_skipped_fields() {
        let rich = RichInto {
            name: "entry".into(),
            value: 42,
            audit_note: "should be ignored".into(),
        };
        let flat: Flat = rich.into();
        assert_eq!(flat.name, "entry");
        assert_eq!(flat.value, 42);
    }
}

// ====================================================================================================================
// Target/destination added fields: type-level `add(field, default)` and `add(field, default(value = expr))`
// ====================================================================================================================

mod add_field_with_defaults {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Full {
        id: i64,
        name: String,
        score: i64,
        tag: Option<String>,
    }

    // into: bar → full, adding score and tag with defaults
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into,
        ty = Full,
        add(field = score, default(value = 100)),
        add(field = tag, default)
    )]
    struct Partial {
        id: i64,
        name: String,
    }

    // from: full → slim, adding score and tag (ignored for from direction, but must have defaults to satisfy parser
    // checks)
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        from,
        ty = Full,
        add(field = score, default),
        add(field = tag, default)
    )]
    struct Slim {
        id: i64,
        name: String,
    }

    #[test]
    fn into_adds_field_with_explicit_value() {
        let partial = Partial {
            id: 1,
            name: "alice".into(),
        };
        let full: Full = partial.into();
        assert_eq!(full.id, 1);
        assert_eq!(full.name, "alice");
        assert_eq!(full.score, 100);
    }

    #[test]
    fn into_adds_field_with_default_trait() {
        let partial = Partial {
            id: 2,
            name: "bob".into(),
        };
        let full: Full = partial.into();
        assert_eq!(full.tag, None); // Option<String>::default() == None
    }

    #[test]
    fn from_with_add_fields_are_ignored() {
        let full = Full {
            id: 3,
            name: "charlie".into(),
            score: 999,
            tag: Some("vip".into()),
        };
        let slim = Slim::from(full);
        assert_eq!(slim.id, 3);
        assert_eq!(slim.name, "charlie");
    }
}

// ====================================================================================================================
// Target/destination type-level `add(field, ty)` without default in custom mapping function
// ====================================================================================================================

mod add_field_ty_custom {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Enriched {
        name: String,
        category: i64,
        priority: String,
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into(custom = "into_enriched"),
        ty = Enriched,
        add(field = category, ty = i64),
        add(field = priority, ty = String),
    )]
    struct Plain {
        name: String,
    }

    #[test]
    fn into_custom_added_fields_as_runtime_params() {
        let plain = Plain { name: "widget".into() };
        let enriched = plain.into_enriched(42, "high".into());
        assert_eq!(enriched.name, "widget");
        assert_eq!(enriched.category, 42);
        assert_eq!(enriched.priority, "high");
    }

    #[test]
    fn into_custom_different_values() {
        let plain = Plain { name: "gadget".into() };
        let enriched = plain.into_enriched(0, "low".into());
        assert_eq!(enriched.category, 0);
        assert_eq!(enriched.priority, "low");
    }
}

// ====================================================================================================================
// Naming collision / Type override: both types have same-named fields representing conceptually different things
// ====================================================================================================================

mod naming_collision {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct External {
        name: String,
        id: i64,
    }

    /// Tests naming collision for the `from` direction.
    /// The target struct `InternalFrom` has a field `name: Option<String>` which is skipped.
    /// The source struct `External` has `name: String` which is ignored.
    /// The generated function `from_external` takes `name: Option<String>` as a runtime argument.
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        from(custom),
        ty = External,
        add(field = name),
    )]
    struct InternalFrom {
        id: i64,
        #[mapper(skip)]
        name: Option<String>,
    }

    /// Tests naming collision for the `into` direction.
    /// The target struct `External` has `name: String` which is added at type level.
    /// The source struct `InternalInto` has a field `name: Option<String>` which is skipped.
    /// We rename the source field to `internal_name` to avoid local variable binding collision.
    /// The generated function `to_external` takes `name: String` as a runtime argument.
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into(custom = "to_external"),
        ty = External,
        add(field = name, ty = String),
    )]
    struct InternalInto {
        id: i64,
        #[mapper(rename = internal_name, skip)]
        name: Option<String>,
    }

    /// Simulated `DateTime` type for testing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DateTime {
        epoch: i64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Record {
        title: String,
        date: DateTime,
    }

    /// The derived struct has `date` as a `String`, but `Record` has `date` as
    /// `DateTime`. We skip `date` at field-level (preventing automatic
    /// name-based mapping), rename it so it doesn't collide, then add it back at
    /// the type-level with `ty = "DateTime"`. The custom function will require
    /// `DateTime` as a runtime argument.
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into(custom),
        ty = Record,
        add(field = date, ty = "DateTime"),
    )]
    struct Draft {
        title: String,
        /// Skip this field so it doesn't automatically map to Record.date
        #[mapper(rename = _date_ignored, skip)]
        date: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Entry {
        label: String,
        date: DateTime,
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        from(custom),
        ty = Entry,
        add(field = date, default(value = "manual-date".to_string())),
    )]
    struct Summary {
        label: String,
        /// Skip this field – it won't be populated from Entry.date
        #[mapper(rename = _date_ignored, skip)]
        date: String,
    }

    #[test]
    fn collision_from_custom_with_same_name() {
        let ext = External {
            name: "original".into(),
            id: 10,
        };
        // The skipped field `name` becomes a runtime parameter of type `Option<String>`.
        let internal = InternalFrom::from_external(ext, Some("override".to_string()));
        assert_eq!(internal.id, 10);
        assert_eq!(internal.name, Some("override".to_string()));
    }

    #[test]
    fn collision_into_custom_with_same_name() {
        let internal = InternalInto {
            id: 42,
            name: Some("skipped_val".to_string()),
        };
        // The target `name` is passed as a runtime argument to `to_external`.
        let ext = internal.to_external("runtime_val".to_string());
        assert_eq!(ext.id, 42);
        assert_eq!(ext.name, "runtime_val");
    }

    #[test]
    fn skip_and_readd_field_with_different_type() {
        let draft = Draft {
            title: "My Post".into(),
            date: "2025-01-15".into(), // String – not used in mapping
        };
        let dt = DateTime { epoch: 1_736_899_200 };
        let record = draft.into_record(dt.clone());
        assert_eq!(record.title, "My Post");
        assert_eq!(record.date, dt);
    }

    #[test]
    fn skip_and_readd_preserves_title() {
        let draft = Draft {
            title: "Another".into(),
            date: "ignored".into(),
        };
        let dt = DateTime { epoch: 0 };
        let record = draft.into_record(dt);
        assert_eq!(record.title, "Another");
        assert_eq!(record.date.epoch, 0);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TargetWithToken {
        id: i64,
        combined: String,
        token: String,
    }

    #[derive(Mapper, Debug, PartialEq, Eq)]
    #[mapper(
        into(custom = "to_target"),
        ty = TargetWithToken,
        add(field = combined, default(value = format!("{} - {}", token, input.token.clone()))),
        add(field = token, ty = String),
    )]
    struct SourceWithToken {
        id: i64,
        #[mapper(skip)]
        token: String,
    }

    #[test]
    fn from_custom_skip_and_readd_same_name() {
        let entry = Entry {
            label: "Test".into(),
            date: DateTime { epoch: 42 },
        };
        // The skipped `date` has no default, so it's a parameter of type `String`.
        let summary = Summary::from_entry(entry, "manual-date".into());
        assert_eq!(summary.label, "Test");
        assert_eq!(summary.date, "manual-date");
    }

    #[test]
    fn test_same_name_skipped_and_add_param() {
        let source = SourceWithToken {
            id: 123,
            token: "struct_token".to_string(),
        };
        let target = source.to_target("param_token".to_string());
        assert_eq!(target.id, 123);
        assert_eq!(target.token, "param_token");
        assert_eq!(target.combined, "struct_token - param_token");
    }
}

// ====================================================================================================================
// Enum skipped variants: `skip(default)` and `skip(default(value = expr))` in `into` derivations
// ====================================================================================================================

mod enum_skip_variants {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    enum SmallEnum {
        #[default]
        Alpha,
        Beta,
    }

    /// `into` derivation: `FullEnum` → `SmallEnum`.
    /// Extra variants in `FullEnum` are mapped to `SmallEnum` defaults.
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(into, ty = SmallEnum)]
    enum FullEnum {
        Alpha,
        Beta,
        /// Skipped variant mapped via `Default::default()` → `SmallEnum::Alpha`
        #[mapper(skip(default))]
        Gamma,
        /// Skipped variant mapped via explicit value
        #[mapper(skip(default(value = SmallEnum::Beta)))]
        Delta,
    }

    #[test]
    fn enum_skip_default_uses_default_trait() {
        let full = FullEnum::Gamma;
        let small: SmallEnum = full.into();
        assert_eq!(small, SmallEnum::Alpha); // Default
    }

    #[test]
    fn enum_skip_default_value_uses_expression() {
        let full = FullEnum::Delta;
        let small: SmallEnum = full.into();
        assert_eq!(small, SmallEnum::Beta);
    }

    #[test]
    fn enum_matched_variants_pass_through() {
        let alpha: SmallEnum = FullEnum::Alpha.into();
        let beta: SmallEnum = FullEnum::Beta.into();
        assert_eq!(alpha, SmallEnum::Alpha);
        assert_eq!(beta, SmallEnum::Beta);
    }
}

// ====================================================================================================================
// Enum added variants: type-level `add(field = VariantName, default)` and `add(field = VariantName, default(value =
//    expr))` in `from` derivations
// ====================================================================================================================

mod enum_add_variants {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum BigEnum {
        Red,
        Green,
        Blue,
        Yellow,
    }

    #[derive(Debug, Default, PartialEq, Eq, Mapper)]
    #[mapper(
        from,
        ty = BigEnum,
        add(field = Blue, default(value = TinyEnum::Red)),
        add(field = Yellow, default),
    )]
    enum TinyEnum {
        #[default]
        Red,
        Green,
    }

    #[test]
    fn added_variant_with_explicit_value() {
        let big = BigEnum::Blue;
        let tiny = TinyEnum::from(big);
        assert_eq!(tiny, TinyEnum::Red);
    }

    #[test]
    fn added_variant_with_default_trait() {
        let big = BigEnum::Yellow;
        let tiny = TinyEnum::from(big);
        assert_eq!(tiny, TinyEnum::Red); // Default → TinyEnum::Red
    }

    #[test]
    fn matched_variants_pass_through() {
        assert_eq!(TinyEnum::from(BigEnum::Red), TinyEnum::Red);
        assert_eq!(TinyEnum::from(BigEnum::Green), TinyEnum::Green);
    }
}

// ====================================================================================================================
// Variant-level field additions (extra fields inside variants) utilizing `default`, `default(value = expr)`, and
//    custom functions
// ====================================================================================================================

mod variant_field_additions {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Action {
        Move {
            x: i64,
            y: i64,
            speed: i64,
        },
        Attack {
            target: String,
            damage: i64,
            critical: bool,
        },
    }

    // from: Action → SimpleAction
    // The extra fields in Action variants are declared via variant-level add
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(from, ty = Action)]
    enum SimpleAction {
        #[mapper(add(field = speed, default(value = 1)))]
        Move { x: i64, y: i64 },
        #[mapper(add(field = damage, default), add(field = critical, default))]
        Attack { target: String },
    }

    // into: SimpleAction2 → Action
    // Added variant-level fields with default values for the into direction
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(into, ty = Action)]
    enum SimpleAction2 {
        #[mapper(add(field = speed, default(value = 5)))]
        Move { x: i64, y: i64 },
        #[mapper(add(field = damage, default(value = 10)), add(field = critical, default))]
        Attack { target: String },
    }

    #[test]
    fn from_variant_add_fields_with_explicit_value() {
        let action = Action::Move {
            x: 10,
            y: 20,
            speed: 99,
        };
        let simple = SimpleAction::from(action);
        assert_eq!(simple, SimpleAction::Move { x: 10, y: 20 });
    }

    #[test]
    fn from_variant_add_fields_with_default() {
        let action = Action::Attack {
            target: "enemy".into(),
            damage: 50,
            critical: true,
        };
        let simple = SimpleAction::from(action);
        assert_eq!(simple, SimpleAction::Attack { target: "enemy".into() });
    }

    #[test]
    fn into_variant_add_fields_with_explicit_value() {
        let simple = SimpleAction2::Move { x: 1, y: 2 };
        let action: Action = simple.into();
        assert_eq!(action, Action::Move { x: 1, y: 2, speed: 5 });
    }

    #[test]
    fn into_variant_add_fields_with_default() {
        let simple = SimpleAction2::Attack { target: "boss".into() };
        let action: Action = simple.into();
        assert_eq!(
            action,
            Action::Attack {
                target: "boss".into(),
                damage: 10,
                critical: false, // bool::default()
            }
        );
    }
}

// ====================================================================================================================
// Variant-level field additions with custom functions
// ====================================================================================================================

mod variant_field_additions_custom {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Event {
        Click { x: i64, y: i64, timestamp: i64 },
        Scroll { delta: i64, metadata: String },
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into(custom),
        ty = Event,
    )]
    enum SimpleEvent {
        // One variant adds timestamp customly without default (gets a provider param)
        #[mapper(add(field = timestamp, ty = i64))]
        Click { x: i64, y: i64 },
        // Other variant adds metadata with a default (no duplicate provider param conflict)
        #[mapper(add(field = metadata, default(value = "none".to_string())))]
        Scroll { delta: i64 },
    }

    #[test]
    fn into_custom_variant_field_as_runtime_param() {
        let simple = SimpleEvent::Click { x: 100, y: 200 };
        // Since timestamp is a variant-level added field without default in Click,
        // it requires a provider closure as a parameter: `timestamp_provider: impl FnOnce(&i64, &i64) -> i64`
        let event = simple.into_event(|x, y| {
            // Assert the field ordering and values are passed correctly to the provider closure
            assert_eq!(*x, 100);
            assert_eq!(*y, 200);
            999
        });
        assert_eq!(
            event,
            Event::Click {
                x: 100,
                y: 200,
                timestamp: 999,
            }
        );
    }

    #[test]
    fn into_custom_variant_field_scroll() {
        let simple = SimpleEvent::Scroll { delta: -3 };
        let event = simple.into_event(|_, _| 0);
        assert_eq!(
            event,
            Event::Scroll {
                delta: -3,
                metadata: "none".to_string(),
            }
        );
    }
}

// ====================================================================================================================
// Mixed into/from with add and skip on the same derive
// ====================================================================================================================

mod mixed_add_skip_bidirectional {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DbRow {
        id: i64,
        name: String,
        created_at: i64,
    }

    /// `into` derive: maps `ApiModel` → `DbRow`, adding `created_at` with a default.
    /// `from` derive: maps `DbRow` → `ApiModel`, the `created_at` additional field is just ignored.
    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into, from,
        ty = DbRow,
        add(field = created_at, default(value = 0))
    )]
    struct ApiModel {
        id: i64,
        name: String,
        /// version only exists on `ApiModel`, skipped for both directions
        #[mapper(skip(default(value = 1)))]
        version: i64,
    }

    #[test]
    fn into_adds_created_at_with_default_value() {
        let api = ApiModel {
            id: 1,
            name: "test".into(),
            version: 5,
        };
        let row: DbRow = api.into();
        assert_eq!(row.id, 1);
        assert_eq!(row.name, "test");
        assert_eq!(row.created_at, 0);
    }

    #[test]
    fn from_ignores_created_at_and_defaults_version() {
        let row = DbRow {
            id: 2,
            name: "entry".into(),
            created_at: 1_700_000_000,
        };
        let api = ApiModel::from(row);
        assert_eq!(api.id, 2);
        assert_eq!(api.name, "entry");
        assert_eq!(api.version, 1); // skip(default(value = 1))
    }
}

// ====================================================================================================================
// Custom-named into function with add(field, ty) and add(field, default(value)) mixed
// ====================================================================================================================

mod custom_fn_mixed_add {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Output {
        data: String,
        code: i64,
        message: String,
    }

    #[derive(Debug, PartialEq, Eq, Mapper)]
    #[mapper(
        into(custom = "to_output"),
        ty = Output,
        add(field = code, default(value = 200)),
        add(field = message, ty = String),
    )]
    struct Input {
        data: String,
    }

    #[test]
    fn custom_fn_mixes_default_and_runtime_params() {
        let input = Input { data: "payload".into() };
        // `code` has a default so it's not a parameter.
        // `message` has a `ty` but no default so it IS a parameter.
        let output = input.to_output("ok".into());
        assert_eq!(output.data, "payload");
        assert_eq!(output.code, 200);
        assert_eq!(output.message, "ok");
    }
}
