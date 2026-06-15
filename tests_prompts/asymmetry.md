# Integration Test: `tests/asymmetry.rs`

## Crate Context

> **Read this section first.** It provides all the information you need about the crate so you do not need to scan the codebase.

### Workspace Layout

```
model-mapper/                    # Workspace root
├── model-mapper/                # Main public crate (this is where tests go)
│   ├── src/lib.rs               # Public API: re-exports derive macro + RefMapper/ValueMapper traits
│   ├── examples/                # 11 example files demonstrating all features
│   └── tests/                   # Integration tests (your target)
├── model-mapper-macros/         # Proc-macro crate (the derive implementation)
│   └── src/
│       ├── lib.rs               # Macro entry point
│       ├── parse.rs             # Darling-based attribute parsing + validation errors
│       └── domain.rs            # Semantic validation + code generation errors
├── test_plan.md                 # Full test plan with all expected error strings
└── Cargo.toml                   # Workspace manifest
```

### Public API

The only public item is the `Mapper` derive macro, re-exported from `model-mapper-macros`:

```rust
use model_mapper::Mapper;
```

The crate also exposes (in `model_mapper::private`) two dispatch traits used internally by generated code:

- `RefMapper<T, R>` — for `with` functions taking `&T`
- `ValueMapper<T, R>` — for `with` functions taking `T` by value

### Features

- `default = ["std"]`
- `std` — enables `dep:anyhow` (used as default error type for fallible conversions)
- With `default-features = false` the crate is `no_std` compatible

### Dev Dependencies (already declared)

- `thiserror` — available for use in tests

### Macro Attribute Syntax Summary

**Type-level** (`#[mapper(...)]`):

- Direction: `from`, `into`, `try_from`, `try_into` (each optionally `custom` or `custom = name`)
- `ty = TargetType` (mandatory) — can be a string literal for complex types: `ty = "Foo<T>"`
- `add(field = name, ty = Type, default)` — extra fields/variants the target has
- `ignore_extra` — ignore unmatched fields/variants in the target
- `err = ErrorType` / `err_with = MapFn` — error handling for `try_*`
- `accumulate` / `accumulate = Accumulator` — collect errors instead of short-circuit
- Multi-target: wrap in `derive(...)` blocks; use `when(ty = T, ...)` at field/variant level

**Variant-level**:

- `rename = OtherName`, `skip(default)`, `skip(default(value = expr))`
- `add(field = name, ...)`, `ignore_extra`

**Field-level**:

- `rename = other`, `skip(default)`, `skip(default(value = expr))`
- `other_ty = T` — field type in the other struct if different
- `with = fn`, `into_with = fn`, `from_with = fn` — custom mapping expressions
- `err = Val` / `err_with = MapFn`
- Hints (nestable): `opt`, `iter`, `map`, `boxed`, `box`, `unbox`

For full attribute documentation, see [README.md](../README.md).

### Example Files (in `model-mapper/examples/`)

| File | Demonstrates |
|---|---|
| `exact_match.rs` | Simple struct/enum mappings, all 4 directions |
| `renames.rs` | Field and variant renaming |
| `skipped_fields.rs` | `skip(default)`, cross-field defaults, custom functions with skipped fields |
| `additional_fields.rs` | `add(field, ty, default)`, custom functions with added fields |
| `ignore_extra_fields.rs` | `ignore_extra` at type and variant level |
| `different_types.rs` | Transform hints: `opt`, `iter`, `map`, `boxed`, `box`, `unbox`, nesting |
| `with.rs` | `with`, `into_with`, `from_with` custom mapping functions/expressions |
| `error_handling.rs` | `try_from`/`try_into`, `err`, `err_with`, `accumulate` |
| `generics.rs` | Generic structs, `other_ty`, decoupled type params |
| `multiple_derives.rs` | Multiple `derive(...)` blocks, `when(ty = T, ...)` overrides |
| `no_std.rs` | Minimal `#![no_std]` usage |

---

## Goal

Implement integration tests in [tests/asymmetry.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/asymmetry.rs) verifying mapping behavior for asymmetric types.

## Scope of Implementation

- Make edits to [tests/asymmetry.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/asymmetry.rs) ONLY.
- Do NOT perform any git commits, branches, or external operations—only perform local file edits.

## Minimum Test Coverage Requirements

- Struct skipped fields: `skip(default)` and `skip(default(value = expr))`.
- Cross-field skipped default expressions (referencing other fields in source).
- Custom functions taking skipped fields as additional parameters (`from(custom)` / `into(custom)`).
- Target/destination added fields: type-level `add(field, default)` and `add(field, default(value = expr))`.
- Target/destination type-level `add(field, ty)` without default in custom mapping function.
- Naming Collision: type-level `add(field = name)` where a field named `name` is skipped on the derived struct.
- Enum skipped variants: `skip(default)` and `skip(default(value = expr))` in `into` derivations.
- Enum added variants: type-level `add(field = VariantName, default)` and `add(field = VariantName, default(value = expr))` in `from` derivations.
- Variant-level field additions (extra fields inside variants) utilizing `default`, `default(value = expr)`, and custom functions (where the provider closures accept references to all in-scope fields).
- **Enum field provider closure arguments:** Verify that custom mapping functions for enum variant-level added or skipped fields receive references to all in-scope fields as closure arguments.
- **Skip and Add Same-Named Fields (Custom Impl):** Skip automatic mapping by marking a field (e.g. `date`) skipped on field level, then add it back at the type-level with a custom type (e.g. `add(field = date, ty = "DateTime")`) in a custom `into(custom)` derivation. Verify that the generated function maps it as a runtime argument, bypassing name-based matching.

## References & Best Practices

- Refer to [additional_fields.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/examples/additional_fields.rs) and [skipped_fields.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/examples/skipped_fields.rs) to understand asymmetries.
- Follow Rust integration testing best practices (use public API only).
- Feel free to implement any other related asymmetry mapping tests you find useful.
