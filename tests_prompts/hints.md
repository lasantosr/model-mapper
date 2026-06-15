# Integration Test: `tests/hints.rs`

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

Implement integration tests in [tests/hints.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/hints.rs) verifying type mapping transform hints.

## Scope of Implementation

- Make edits to [tests/hints.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/tests/hints.rs) ONLY.
- Do NOT perform any git commits, branches, or external operations—only perform local file edits.

## Minimum Test Coverage Requirements

- Simple transform hints: `opt`, `iter`, `map`, `boxed`, `box`, `unbox`.
- Multi-level nested hint combinations (e.g. `opt(iter)`, `iter(opt)`, `opt(map(opt(boxed)))`).

## References & Best Practices

- Refer to [different_types.rs](file:///home/ubuntu/projects/model-mapper/model-mapper/examples/different_types.rs) to see how transform hints are used.
- Follow Rust integration testing best practices (use public API only).
- Feel free to implement any other related transform hint tests you find useful.
