//! # Model Mapper
//!
//! A powerful Rust macro to generate boilerplate-free declarations of `From`, `Into`, `TryFrom`, and `TryInto` traits
//! for converting between structs and enums.
//!
//! It is designed to handle common patterns like detailed DTOs to internal entities, handling optional fields, nested
//! collections, and even disparate generic types.
//!
//! ## Features
//!
//! - **Zero Boilerplate**: Automatically implements `From`, `Into`, `TryFrom`, and `TryInto`.
//! - **Flexible Mapping**: Handle renamed fields, skipped fields, and additional fields.
//! - **Custom Logic**: Inject custom conversion logic for specific fields using functions or expressions.
//! - **Generics Support**: Seamless mapping between generic types with different parameters.
//! - **Multiple Targets**: Map a single type to multiple other types with conditional configurations.
//! - **Nested Mapping**: Built-in support for mapping inner values within Option, iterators, and maps.
//! - **`no_std` compatible**: Works in `no_std` environments (with default features disabled).
//!
//! ## Quick Start
//!
//! The most common use case is mapping between domain entities and DTOs.
//!
//! ```rust
//! # use model_mapper::Mapper;
//! # struct Entity {
//! #     id: i64,
//! #     name: String,
//! # }
//! #[derive(Mapper)]
//! #[mapper(from, ty = Entity)]
//! pub struct Model {
//!     id: i64,
//!     name: String,
//! }
//! ```
//!
//! The macro expansion above would generate something like:
//!
//! ```rust
//! # struct Entity { id: i64, name: String }
//! # struct Model { id: i64, name: String }
//! impl From<Entity> for Model {
//!     fn from(Entity { id, name }: Entity) -> Self {
//!         Self {
//!             id: Into::into(id),
//!             name: Into::into(name),
//!         }
//!     }
//! }
//! ```
//!
//! Because types doesn't always fit like a glove, you can provide additional fields on runtime, at the cost of not
//! being able to use the `From` trait:
//!
//! ```rust
//! # use model_mapper::Mapper;
//! pub mod service {
//!     pub struct UpdateUserInput {
//!         pub user_id: i64,
//!         pub name: Option<String>,
//!         pub surname: Option<String>,
//!     }
//! }
//!
//! #[derive(Mapper)]
//! #[mapper(
//!     into(custom = "into_update_user"),
//!     ty = service::UpdateUserInput,
//!     add(field = user_id, ty = i64),
//!     add(field = surname, default(value = None))
//! )]
//! pub struct UpdateProfileRequest {
//!     pub name: String,
//! }
//! ```
//!
//! Would generate something like:
//!
//! ```rust
//! # pub mod service {
//! #    pub struct UpdateUserInput {
//! #        pub user_id: i64,
//! #        pub name: Option<String>,
//! #        pub surname: Option<String>,
//! #    }
//! # }
//! # struct UpdateProfileRequest { name: String }
//! impl UpdateProfileRequest {
//!     /// Builds a new [service::UpdateUserInput] from a [UpdateProfileRequest]
//!     pub fn into_update_user(self, user_id: i64) -> service::UpdateUserInput {
//!         struct Input {
//!             user_id: i64,
//!         }
//!         let input = Input { user_id };
//!         let Self { name } = self;
//!         service::UpdateUserInput {
//!             user_id: input.user_id,
//!             surname: None,
//!             name: Into::into(name),
//!         }
//!     }
//! }
//! ```
//!
//! Other advanced use cases are available on the [examples folder](https://github.com/lasantosr/model-mapper/tree/main/model-mapper/examples/).
//!
//! ## Detailed Usage
//!
//! A `mapper` attribute is required at type-level and it's optional at field or variant level.
//!
//! The following attributes are available.
//!
//! - Type level attributes:
//!
//!   - `ty = PathType` _(**mandatory**)_: The derived type to map to/from. Can be a string literal for complex types
//!     (e.g. `ty = "Type<T>"`)
//!   - `from` _(optional)_: Whether to derive `From` the derived type for the base type
//!     - `custom` _(optional)_: Derive a custom function instead of the trait
//!     - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
//!   - `into` _(optional)_: Whether to derive `From` the base type for the derived type
//!     - `custom` _(optional)_: Derive a custom function instead of the trait
//!     - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
//!   - `try_from` _(optional)_: Whether to derive `TryFrom` the derived type for the base type
//!     - `custom` _(optional)_: Derive a custom function instead of the trait
//!     - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
//!     - `err = TargetError` _(optional)_: Explicit error type for fallible conversions.
//!     - `accumulate` _(optional)_: Collect errors into `Vec<TargetError>` instead of short-circuiting on the first
//!       failure.
//!     - `accumulate = CustomAccumulator` _(optional)_: Collect errors into `CustomAccumulator` instead of `Vec`.
//!   - `try_into` _(optional)_: Whether to derive `TryFrom` the base type for the derived type
//!     - `custom` _(optional)_: Derive a custom function instead of the trait
//!     - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
//!     - `err = TargetError` _(optional)_: Explicit error type for fallible conversions.
//!     - `accumulate` _(optional)_: Collect errors into `Vec<TargetError>` instead of short-circuiting on the first
//!       failure.
//!     - `accumulate = CustomAccumulator` _(optional)_: Collect errors into `CustomAccumulator` instead of `Vec`.
//!   - `add` _(optional, multiple)_: Additional fields (for structs with named fields) or variants (for enums) the
//!     derived type has and the base type doesn't **&#xb9;**
//!     - `field = other_field` _(mandatory)_: The field or variant name
//!     - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
//!     - `default` _(optional)_: The field or variant will be populated using `Default::default()` (mandatory for
//!       enums, with or without value)
//!       - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
//!   - `ignore_extra` _(optional)_: Opt-out for compile-time safety. Ignore all extra fields (for structs) or variants
//!     (for enums) of the derived type **&#xb2;**
//!
//! - Variant level attributes:
//!
//!   - `rename = OtherVariant` _(optional)_: To rename this variant on the derived enum
//!   - `add` _(optional, multiple)_: Additional fields of the variant that the derived variant has and the base variant
//!     doesn't **&#xb9;**
//!     - `field = other_field` _(mandatory)_: The field name
//!     - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
//!     - `default` _(optional)_: The field or variant will be populated using `Default::default()`
//!       - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
//!   - `skip` _(optional)_: Whether to skip this variant because the derived enum doesn't have it
//!     - `default` _(mandatory)_: The field or variant will be populated using `Default::default()`
//!       - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression
//!         instead
//!   - `ignore_extra` _(optional)_: Whether to ignore all extra fields of the derived variant (only valid for _from_
//!     and _`try_from`_) **&#xb2;**
//!
//! - Field level attributes:
//!
//!   - `rename = other_name` _(optional)_: To rename this field on the derived type
//!   - `other_ty = T` _(optional)_: The type of this field in the derived type, if it differs from the base type's
//!     field.
//!   - `skip` _(optional)_: Whether to skip this field because the derived type doesn't have it
//!     - `default` _(optional)_: The field or variant will be populated using `Default::default()`
//!       - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression
//!         instead
//!   - `err = ErrorVal` _(optional)_: Map conversion failures for this field to the specific error value `ErrorVal`.
//!     Used for error erasure where the original error is discarded (e.g. mapping to a unit variant error like
//!     `AppError::InvalidStatus`).
//!   - `err_with = MapFn` _(optional)_: Map conversion failures for this field using the helper function/callable
//!     `MapFn` (which can be a tuple variant constructor, a closure, or a function/method path). This preserves or maps
//!     the original error.
//!
//! - Additional hints on how to map fields:
//!
//!   - `opt` _(optional)_: The field is an `Option` and the inner value shall be mapped **&#xb3;**
//!   - `iter` _(optional)_: The field is an iterator and the inner value shall be mapped **&#xb3;**
//!   - `map` _(optional)_: The field is a hashmap-like iterator and the inner value shall be mapped **&#xb3;**
//!   - `boxed` _(optional)_: The field is a `Box` and the inner value shall be mapped **&#xb3;**
//!   - `box` _(optional)_: The derived field is a `Box` while the base field is not **&#xb3;**
//!   - `unbox` _(optional)_: The base field is a `Box` while the derived field is not **&#xb3;**
//!   - `with = mod::my_function` _(optional)_: If the field type doesn't implement `Into` or `TryInto` the derived
//!     field, this property allows you to customize the behavior by providing a conversion function
//!   - `into_with = mod::my_function` _(optional)_: The same as above but only for the `into` or `try_into` derives
//!   - `from_with = mod::my_function` _(optional)_: The same as above but only for the `from` or `try_from` derives
//!
//! **&#xb9;** When providing additional fields without defaults, the `From` and `TryFrom` traits can't be derived and
//! a custom function will be required instead. When deriving `into` or `try_into`, the `ty` must be provided as well.
//!
//! **&#xb2;** When ignoring fields or variants it might be required that the enum or the struct implements `Default`
//! in order to properly populate it.
//!
//! **&#xb3;** Hints can be nested, for example: `opt(vec)`, `vec(opt(with = "my_custom_fn"))`.
//!
//! ### Scope & Evaluation Order
//!
//! When writing custom mapping expressions (such as `with`, `from_with`, or `into_with`) or configuring skipped/added
//! fields with default values, the following scope and evaluation order behaviors apply:
//!
//! #### Variable Scope & Namespaces
//! - **Source Fields:** The local variables in scope inside custom expressions correspond to the fields of the input
//!   struct of the mapping:
//!   - In `from` and `try_from` mappings, fields are referenced by their names in the **remote type** (`ty`).
//!   - In `into` and `try_into` mappings, fields are referenced by their names in the **local type** (the struct
//!     deriving `Mapper`).
//! - **Added Fields:** Added fields injected as custom function parameters are namespaced under the local `input`
//!   structure. Access them using the `input.` prefix (e.g., `input.user_id`).
//!
//! #### Evaluation & Consumption Order (Borrow-Before-Move)
//! To allow custom mapping expressions and default value expressions (e.g., `skip(default(value = ...))` or `add(field
//! = ..., default(value = ...))`) to safely reference or borrow other source fields before they are moved/consumed,
//! conversions are evaluated in a strict priority order:
//! 1. **Added fields** with default expressions are evaluated first.
//! 2. **Custom-mapped and skipped fields** with default expressions are evaluated next.
//! 3. **Regular fields** (which move/consume the source variables) are evaluated last.
//!
//! ### Multiple derives
//!
//! When deriving conversions for a single type, attributes can be set directly:
//!
//! ```rust-ignore
//! #[mapper(from, into, ty = OtherType, add(field = field_1, default), add(field = field_2, default))]
//! struct MyStruct;
//! ```
//!
//! But we can also derive conversions for multiple types by wrapping the properties on a `derive` attribute:
//!
//! ```rust-ignore
//! #[mapper(derive(try_into, ty = OtherType, add(field = field_1, default)))]
//! #[mapper(derive(from, ty = YetAnotherType))]
//! struct MyStruct;
//! ```
//!
//! If multiple conversions are involved, both variant and field level attributes can also be wrapped in a `when`
//! attribute and must set the `ty` they refer to:
//!
//! ```rust-ignore
//! #[mapper(when(ty = OtherType, with = ToString::to_string))]
//! #[mapper(when(ty = YetAnotherType, skip(default)))]
//! struct MyStruct;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Re-export derive macro crate
extern crate model_mapper_macros;
#[doc(hidden)]
pub use model_mapper_macros::*;

#[doc(hidden)]
#[expect(clippy::std_instead_of_alloc, reason = "feature gated")]
pub mod private {
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    pub use alloc::boxed::Box;
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    pub use alloc::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::boxed::Box;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;

    pub trait RefMapper<T, R> {
        fn map_value(&self, arg: T) -> R;
    }
    impl<F, T, R> RefMapper<T, R> for F
    where
        F: ?Sized + Fn(&T) -> R,
    {
        #[inline(always)]
        fn map_value(&self, arg: T) -> R {
            (self)(&arg)
        }
    }

    pub trait ValueMapper<T, R> {
        fn map_value(&self, arg: T) -> R;
    }
    impl<F, T, R> ValueMapper<T, R> for &F
    where
        F: ?Sized + Fn(T) -> R,
    {
        #[inline(always)]
        fn map_value(&self, arg: T) -> R {
            (*self)(arg)
        }
    }
}
