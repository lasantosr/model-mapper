# Integration Test Plan for `model-mapper` Macro

This document details the test plan to verify the macro logic of the `model-mapper` crate. The goal is to establish a comprehensive baseline of integration tests in a new `tests/` directory under the [model-mapper](file:///home/ubuntu/projects/model-mapper/model-mapper) crate, covering all happy paths, edge cases, and invalid macro configurations (negative tests) to achieve 100% coverage.

> [!NOTE]
> All integration tests must interact strictly with the public API surface of the crate, without using any crate-internal implementation details.

---

## 1. Test Architecture

The integration tests will be organized as follows. We will also add [trybuild](https://crates.io/crates/trybuild) to `[dev-dependencies]` in [Cargo.toml](file:///home/ubuntu/projects/model-mapper/model-mapper/Cargo.toml) to support compile-fail UI tests.

| Test File Name | Target Feature Category |
| :--- | :--- |
| [struct_mappings.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/struct_mappings.rs) | Structs (Named, Tuple, Newtype) mappings, directions, widening, and round-trips. |
| [enum_mappings.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/enum_mappings.rs) | Enums (Unit, Unnamed, Named fields) mappings, directions, and round-trips. |
| [renames.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/renames.rs) | Renaming at type, variant, and field levels, and conflict cases. |
| [asymmetry.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/asymmetry.rs) | Asymmetric structs/enums (skipped and added fields/variants, including variant fields). |
| [ignore_extra.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/ignore_extra.rs) | `ignore_extra` behavior at struct, enum, and variant levels. |
| [hints.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/hints.rs) | Transform hints (`opt`, `iter`, `map`, `boxed`, `box`, `unbox`) and nesting. |
| [custom_functions.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/custom_functions.rs) | Custom mapping functions/expressions (`with`, `from_with`, `into_with`), binding scopes, ref vs value dispatch, and custom name generation. |
| [error_handling.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/error_handling.rs) | Fallible conversions (`try_from` / `try_into`), error erasure, custom mapping, anyhow default, and accumulation. |
| [generics.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/generics.rs) | Generics mapping, decoupling, bounds propagation, `where` clauses, and complex type paths. |
| [multiple_targets.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/multiple_targets.rs) | Mapping a single source type to multiple target types (`derive` + `when` blocks). |
| [no_std.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/no_std.rs) | `#![no_std]` compilation and execution verification. |
| [compile_tests.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/compile_tests.rs) | Negative UI compiler tests and positive complex tests using `trybuild`. |

---

## 2. Test Specifications

### 2.1 Struct Mappings

Verify basic mapping functionality across different struct shapes and conversion directions.

- **Shapes to Test:**
  - Named-field structs (traditional `struct Foo { x: i32 }`).
  - Tuple / unnamed-field structs (`struct Foo(i32, String)`).
  - Newtype structs (`struct Foo(String)`).
  - *Note:* Unit / zero-field structs (`struct Foo;`) are unsupported by the Darling config (`struct_unit` is absent) and must be verified in compile-fail tests.
- **Conversion Directions:**
  - `from`, `into`, `try_from`, `try_into` combinations.
  - **New Coverage Territory:** Explicitly test `try_from` and `try_into` on both tuple and newtype structs to ensure macro generates correct field destructuring indices.
- **Edge Cases & Integration Behavior:**
  - **Round-trip testing:** Convert `StructA` to `StructB` via `into` / `try_into` and convert back via `from` / `try_from` to assert structural equality.
  - **Same-type mapping:** Verify mapping a type to itself (Self -> Self) works correctly.
  - **Implicit `Into` Widening:** Verify that fields convert automatically without hints if they implement `Into` (e.g. `i32` -> `i64`, `&str` -> `String`, `String` -> `PathBuf`).

### 2.2 Enum Mappings

Verify mapping functionality between various enum configurations.

- **Shapes to Test:**
  - Unit variant enums (simple C-style enums).
  - Complex enums containing tuple/unnamed variants.
  - Complex enums containing named-field variants.
  - Mixed enums containing all variant types.
- **Conversion Directions:**
  - All four combinations of `from`, `into`, `try_from`, and `try_into`.
- **Edge Cases & Integration Behavior:**
  - **Round-trip testing:** Round-trip conversions between complex enums to guarantee structural equivalence.
  - **Infallible Fallibility:** Verify that converting unit-variant-only enums via `try_from`/`try_into` is always `Ok`, returning the expected target variant.

### 2.3 Field & Variant Renaming

Verify that fields and variants can be mapped even if their names differ.

- **Scenarios to Test:**
  - Struct field renaming using `#[mapper(rename = target_name)]`.
  - Enum variant renaming using `#[mapper(rename = TargetVariant)]`.
  - Field renaming inside enum named-field variants.
  - Multiple field/variant renames on the same type.
- **Edge Cases:**
  - **Rename + Skip Collision:** Verify what happens when `#[mapper(rename = other, skip(default))]` is defined. The skip default value should take precedence, and the rename must not cause compilation or bind issues.

### 2.4 Asymmetric Structs & Enums

Verify mappings when structs or enums do not share the exact same fields or variants.

- **Struct Asymmetry (Skipped & Added Fields):**
  - **Destination side mapping extra fields (`from` / `try_from` direction):**
    - Fields skipped using `#[mapper(skip(default))]` (uses `Default::default()`).
    - Fields skipped using `#[mapper(skip(default(value = expr)))]` (uses custom expression).
    - **Cross-Field Skipped Defaults:** Test when `expr` references other mapped fields (e.g., `#[mapper(skip(default(value = field2 / 2)))]`).
    - Fields skipped without defaults in custom functions (`from(custom)` or `from(custom = "name")`), verified as additional function arguments.
  - **Source side mapping extra fields (`into` / `try_into` direction):**
    - Adding fields to target via type-level `add(field = ..., default)`.
    - Adding fields to target via type-level `add(field = ..., default(value = expr))`.
    - Adding fields to target without defaults via `into(custom)` and type-level `add(field = ..., ty = "Type")`, verified as additional arguments.
  - **Name Collisions:** Verify naming collision avoidance when adding a type-level `add(field = collides)` where a field named `collides` is skipped on the derived struct.
  - **Skip and Add Same-Named Fields (Custom Impl):** Verify that when a source struct and target struct have a field with the same name (e.g., `date`) but representing different business concepts (e.g., creation date vs last updated date), we can skip the automatic name-based mapping by marking `date` as skipped on the field level, and add it back at the type-level with a custom type (e.g. `add(field = date, ty = "DateTime")`) using a custom `into(custom)` derivation. The generated custom function must take `date` as a runtime parameter to populate the target struct, bypassing automatic conversion.
- **Enum Asymmetry (Skipped & Added Variants/Fields):**
  - **Source side has extra variants (`into` / `try_into` direction):**
    - Skipping variant via `#[mapper(skip(default))]` (defaults the whole target enum).
    - Skipping variant via `#[mapper(skip(default(value = expr)))]` (evaluates variant expression).
  - **Destination side has extra variants (`from` / `try_from` direction):**
    - Adding variants to destination via type-level `add(field = VariantName, default)`.
    - Adding variants to destination via type-level `add(field = VariantName, default(value = expr))`.
  - **Variant-Level Field Additions:**
    - Test adding fields *inside* a variant (e.g. derived variant has an extra field).
    - Named-field variant with extra fields utilizing `default` and `default(value = expr)`.
    - Test in both `from`/`try_from` and `into`/`try_into` directions, as well as in custom functions (supplying `ty` to variant field `add`).

### 2.5 Transform Hints

Verify mapping customization when fields cannot be directly converted via `Into`/`TryInto`.

- **Basic Hints:**
  - `opt`: Handles mapping inner values of `Option<T>` to `Option<U>`.
  - `iter`: Handles collection conversions (e.g., `Vec<T>` to `HashSet<U>`) mapping elements.
  - `map`: Handles mapping keys/values of hashmap-like collections.
  - `boxed`: Maps `Box<T>` to `Box<U>`.
  - `box`: Maps a raw `T` to a heap-allocated `Box<U>`.
  - `unbox`: Maps a heap-allocated `Box<T>` to a raw `U`.
- **Nested Hint Combinations:**
  - `opt(iter)`: e.g., `Option<Vec<T>>` to `Option<Vec<U>>`.
  - `iter(opt)`: e.g., `Vec<Option<T>>` to `HashSet<Option<U>>`.
  - `opt(map(opt(boxed)))`: High-complexity nested mappings.

### 2.6 Custom Mapping Functions

Verify custom mapping functions, bindings, and trait dispatch.

- **Custom Mapping Logic:**
  - `with = expr`: Custom expression or function path for both directions.
  - `from_with = expr`: Custom expression/function path used only for `from`/`try_from` direction.
  - `into_with = expr`: Custom expression/function path used only for `into`/`try_into` direction.
  - Mixed custom logic within nested hints (e.g., `opt(iter(with = "my_func"))`).
- **Ref vs Value Dispatch:**
  - Verify function paths that take references (e.g., `String::len` which takes `&String`) vs ownership (e.g., `i32::abs` which takes `i32`). This exercises the library's `RefMapper` and `ValueMapper` traits.
- **Cross-Field Expressions:**
  - Test mapping expressions that reference multiple source fields by name (e.g. `into_with = format!("{},{}", field1, field2)`). Verify that all source fields are properly destructured and in-scope.
- **Custom Name Generation:**
  - Verify that when `custom` is used without an explicit name (e.g. `from(custom)` or `into(custom)`), the naming conventions compile and output correct name functions:
    - `from(custom)` -> `from_{snake_case_type}`
    - `into(custom)` -> `into_{snake_case_type}`
    - `try_from(custom)` -> `try_from_{snake_case_type}`
    - `try_into(custom)` -> `try_into_{snake_case_type}`
  - Ensure type paths with namespaces (e.g., `ty = "service::UpdateUserInput"`) resolve to clean snake-case function names (e.g. `into_update_user_input`).

### 2.7 ignore_extra Features

Verify `ignore_extra` behaviors at the struct, enum, and variant levels.

- **Struct Behavior:**
  - `ignore_extra` on `into` struct: Target struct must implement `Default`; missing target fields must be filled with defaults while matched fields map normally.
  - `ignore_extra` on `from` struct: Source struct contains extra fields not on self; they are ignored without requiring `Default` on self.
- **Enum Behavior:**
  - `ignore_extra` on `into` enum: Extra target variants are valid and not matched.
  - `ignore_extra` on `from` enum: Source enum has extra variants; destination enum must implement `Default` (to map unhandled source variants to `Default::default()`).
- **Mixed & Variant Levels:**
  - `ignore_extra` combined with explicit type-level `add` fields.
  - **Variant-Level `ignore_extra`:** Verify that `ignore_extra` can be applied at variant level (only valid on `from` / `try_from`) to ignore extra fields inside a specific variant in the source.

### 2.8 Error Handling & Accumulation

Verify fallible mappings, custom error types, and error accumulation.

- **Short-circuiting vs. Accumulation:**
  - Short-circuiting `TryFrom`/`TryInto`: Returns `Result<T, Err>` and stops at the first field error.
  - Error accumulation: Returns `Result<T, Vec<Err>>` and collects all field-level failures.
  - Custom accumulator: Returns `Result<T, CustomAccumulator>` using `accumulate = CustomAccumulator`.
- **Error Mapping Options:**
  - `err = TargetError`: Maps a conversion failure to a specific unit variant or value (error erasure).
  - `err_with = MapFn`: Maps a conversion failure through a constructor, closure, or helper function path.
- **Default Error Type:**
  - Test `try_from` and `try_into` conversions where no explicit `err` type is defined. Verify that they default to returning `::anyhow::Error` as the error type.
- **Directions:**
  - Parallel testing of both `try_from` and `try_into` directions with field-level `err` and error accumulation to assert symmetrical code generation.

### 2.9 Generics Support

Verify that the macro correctly parses and handles generics, including bounds propagation.

- **Generics Mapping Scenarios:**
  - Mapping structs with identical generic parameter names (e.g., `Foo<T>` -> `Bar<T>`).
  - Decoupled generic mapping: `#[mapper(..., ty = "Target<A, B>")]` on `Source<X, Y>`.
  - Explicit generic type overrides via `#[mapper(other_ty = SourceGeneric)]`.
  - **Existing `where` clauses:** Verify that any `where` bounds on the source type are properly propagated in the generated implementation's `where` clause.
  - **Quoted/Complex Type Paths:** Verify mapping using quoted complex type paths (e.g. `ty = "Foo<T>"`) and namespaced paths (e.g. `ty = "service::UpdateUserInput"`).

### 2.10 Multiple Targets & Derives

Verify the ability to map a single source type to multiple distinct target types.

- **Scenarios to Test:**
  - Multiple `#[mapper(derive(from, ty = TargetA))]` and `#[mapper(derive(into, ty = TargetB))]` on the same struct/enum.
  - Multiple separate `#[mapper(...)]` blocks on the same item.
  - Type-specific overrides using `#[mapper(when(ty = TargetA, skip))]`.
  - default settings (e.g. `#[mapper(skip(default))]`) combined with `when`-specific overrides.

### 2.11 `no_std` Compatibility

Verify compilation and runtime correctness in a non-standard library context.

- **Testing approach:**
  - Implement a compilation integration test file containing `#![no_std]` and import the crate with `default-features = false`.
  - Map structs and enums containing only core types (integers, booleans, `core::option::Option`).

---

## 3. Negative / Compile-Fail Test Specifications

Verify that the macro correctly triggers clean compile-time errors with appropriate span mapping for invalid code. We will implement these tests using the `trybuild` crate in `tests/compile_tests.rs`.

The compiler error messages must match the rules in [domain.rs](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs) and [parse.rs](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/parse.rs) exactly.

| Test Code Scenario | Expected Compiler Diagnostic Error | Source Reference |
| :--- | :--- | :--- |
| Mix single-type attributes at struct level with multi-type `derive` items. | `"Cannot mix single-type attributes directly on the struct/enum with multi-type `derive` items"` | [parse.rs:300](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/parse.rs#L300) |
| Mix single-type attributes at variant level with `when` items. | `"Cannot mix single-type attributes directly on the variant with multi-type `when` items"` | [parse.rs:404](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/parse.rs#L404) |
| Mix single-type attributes at field level with `when` items. | `"Cannot mix single-type attributes directly on the field with multi-type `when` items"` | [parse.rs:516](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/parse.rs#L516) |
| Provide both `err` and `err_with` on the same field. | `"Only one of 'err' or 'err_with' can be set"` | [domain.rs:138](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L138) |
| Provide `err` on an infallible (`from` or `into`) conversion. | `"'err' is only valid when 'try_from' or 'try_into' is set"` | [domain.rs:142](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L142) |
| Provide `err_with` on an infallible (`from` or `into`) conversion. | `"'err_with' is only valid when 'try_from' or 'try_into' is set"` | [domain.rs:155](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L155) |
| Provide a closure expression directly to the `err` attribute. | `"Use 'err_with' instead of 'err' for closure-based error mapping"` | [domain.rs:148](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L148) |
| Provide multiple mapping hints (e.g. `opt` and `iter`) on the same level of a field. | `"Only one of 'with', 'into_with'/'from_with', 'opt', 'iter', 'map', 'boxed', 'box' or 'unbox' can be set"` | [domain.rs:201](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L201) |
| Use `into_with` on a `from` or `try_from` direction (struct). | `"'into_with' is not allowed on a FROM/TRY_FROM mapping direction"` | [domain.rs:495](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L495) |
| Use `into_with` on a `from` or `try_from` direction (enum). | `"'into_with' is not allowed on a FROM/TRY_FROM mapping direction"` | [domain.rs:993](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L993) |
| Use `from_with` on an `into` or `try_into` direction (struct). | `"'from_with' is not allowed on an INTO/TRY_INTO mapping direction"` | [domain.rs:678](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L678) |
| Use `from_with` on an `into` or `try_into` direction (enum). | `"'from_with' is not allowed on an INTO/TRY_INTO mapping direction"` | [domain.rs:1206](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1206) |
| Skip a field on a non-custom `from`/`try_from` mapping without a default value. | `"Enable \`default\` here or include \`custom\` on \`from\` and \`try_from\` derives"` | [domain.rs:501](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L501) |
| Add a field on a non-custom `into`/`try_into` mapping without a default value. | `"Enable \`default\` here or include \`custom\` on \`into\` and \`try_into\` derives"` | [domain.rs:684](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L684) |
| Add a field to custom `into`/`try_into` without a default value and without a type. | `"Provide a field type with \`ty\` if the field is not \`default\` to derive \`into\` and \`try_into\`"` | [domain.rs:689](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L689) |
| Skip an enum variant on a non-custom `into`/`try_into` mapping without a default. | `"Enable \`default\` here required for \`into\` and \`try_into\` derives"` | [domain.rs:1212](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1212) |
| Add a variant to `from`/`try_from` mapping without a default value. | `"Missing mandatory \`default\` for enums when deriving \`from\` or \`try_from\`"` | [domain.rs:1004](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1004) |
| Add a variant with a type override on an enum mapping. | `"Illegal attribute for enums"` | [domain.rs:1001](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1001) |
| Use `accumulate` on infallible struct `from` conversion. | `"accumulate is only valid for try_from / try_into"` | [domain.rs:476](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L476) |
| Use `accumulate` on infallible struct `into` conversion. | `"accumulate is only valid for try_from / try_into"` | [domain.rs:659](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L659) |
| Use `accumulate` on infallible enum `from` conversion. | `"accumulate is only valid for try_from / try_into"` | [domain.rs:974](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L974) |
| Use `accumulate` on infallible enum `into` conversion. | `"accumulate is only valid for try_from / try_into"` | [domain.rs:1187](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1187) |
| Variant-level add field without default on non-custom `into` enum. | `"Enable \`default\` here or include \`custom\` on \`into\` and \`try_into\` derives"` | [domain.rs:1217](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1217) |
| Variant-level add field custom `into` enum without default and without type. | `"Provide a field type with \`ty\` if the field is not \`default\` to derive \`into\` and \`try_into\`"` | [domain.rs:1222](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L1222) |
| Skip field on non-custom `from`/`try_from` struct add field without default. | `"Enable \`default\` here or include \`custom\` on \`from\` and \`try_from\` derives"` | [domain.rs:512](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain.rs#L512) |
| Duplicate derive target type. | `"This type is duplicated: 'Target'"` | [parsing.rs:377](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain_new/parsing.rs#L377) |
| `when(ty = X)` targeting type with no derive defined. | `"There is no derive defined for type: 'Target'"` | [parsing.rs:383](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/domain_new/parsing.rs#L383) |
| Derive `Mapper` on a unit struct (`struct Foo;`). | Darling compilation error (unsupported shape) | [parse.rs:19](file:///home/ubuntu/projects/model-mapper/model-mapper-macros/src/parse.rs#L19) |

---

## 4. Positive trybuild Regression Tests

To guard against parsing and code-generation regressions, we will also include positive compile tests targeting complex valid macro configurations:
- **Comprehensive Derive**: A single struct deriving `from`, `into`, `try_from`, and `try_into` to the same target type.
- **Hint Collection**: A struct utilizing all hint types (`opt`, `iter`, `map`, `boxed`, `box`, `unbox`) in different fields.
- **Deep nesting**: Verify successful compile of complex nested hints like `opt(iter(map(boxed)))`.
