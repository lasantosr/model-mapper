mod domain;
mod expand;
mod input;
mod model_mapper;
mod parse;
mod type_path_ext;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

/// Derive mapper functions to convert between types.
///
/// A `mapper` attribute is required at type-level and it's optional at field or variant level.
///
/// The following attributes are available:
///
/// #### Type level attributes
///
/// - `ty = PathType` _(**mandatory**)_: The derived type to map to/from. Can be a string literal for complex types
///   (e.g. `ty = "Type<T>"`)
/// - `from` _(optional)_: Whether to derive `From` the derived type for the base type
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
/// - `into` _(optional)_: Whether to derive `From` the base type for the derived type
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
/// - `try_from` _(optional)_: Whether to derive `TryFrom` the derived type for the base type
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
///   - `err = TargetError` _(optional)_: Explicit error type for fallible conversions.
///   - `accumulate` _(optional)_: Collect errors into `Vec<TargetError>` instead of short-circuiting on the first
///     failure.
///   - `accumulate = CustomAccumulator` _(optional)_: Collect errors into `CustomAccumulator` instead of `Vec`.
/// - `try_into` _(optional)_: Whether to derive `TryFrom` the base type for the derived type
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
///   - `err = TargetError` _(optional)_: Explicit error type for fallible conversions.
///   - `accumulate` _(optional)_: Collect errors into `Vec<TargetError>` instead of short-circuiting on the first
///     failure.
///   - `accumulate = CustomAccumulator` _(optional)_: Collect errors into `CustomAccumulator` instead of `Vec`.
/// - `add` _(optional, multiple)_: Additional fields (for structs with named fields) or variants (for enums) the
///   derived type has and the base type doesn't **&#xb9;**
///   - `field = other_field` _(mandatory)_: The field or variant name
///   - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
///   - `default` _(optional)_: The field or variant will be populated using `Default::default()` (mandatory for enums,
///     with or without value)
///     - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
/// - `ignore_extra` _(optional)_: Opt-out for compile-time safety. Ignore all extra fields (for structs) or variants
///   (for enums) of the derived type **&#xb2;**
///
/// #### Variant level attributes
///
/// - `rename = OtherVariant` _(optional)_: To rename this variant on the derived enum
/// - `add` _(optional, multiple)_: Additional fields of the variant that the derived variant has and the base variant
///   doesn't **&#xb9;**
///   - `field = other_field` _(mandatory)_: The field name
///   - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
///   - `default` _(optional)_: The field or variant will be populated using `Default::default()`
///     - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
/// - `skip` _(optional)_: Whether to skip this variant because the derived enum doesn't have it
///   - `default` _(mandatory)_: The field or variant will be populated using `Default::default()`
///     - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression
///       instead
/// - `ignore_extra` _(optional)_: Whether to ignore all extra fields of the derived variant (only valid for _from_ and
///   _`try_from`_) **&#xb2;**
///
/// #### Field level attributes
///
/// - `rename = other_name` _(optional)_: To rename this field on the derived type
/// - `other_ty = T` _(optional)_: The type of this field in the derived type, if it differs from the base type's field.
/// - `skip` _(optional)_: Whether to skip this field because the derived type doesn't have it
///   - `default` _(optional)_: The field or variant will be populated using `Default::default()`
///     - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression
///       instead
/// - `err = ErrorVal` _(optional)_: Map conversion failures for this field to the specific error value `ErrorVal`. Used
///   for error erasure where the original error is discarded (e.g. mapping to a unit variant error like
///   `AppError::InvalidStatus`).
/// - `err_with = MapFn` _(optional)_: Map conversion failures for this field using the helper function/callable `MapFn`
///   (which can be a tuple variant constructor, a closure, or a function/method path). This preserves or maps the
///   original error.
///
/// Additional hints on how to map fields:
///
/// - `opt` _(optional)_: The field is an `Option` and the inner value shall be mapped **&#xb3;**
/// - `iter` _(optional)_: The field is an iterator and the inner value shall be mapped **&#xb3;**
/// - `map` _(optional)_: The field is a hashmap-like iterator and the inner value shall be mapped **&#xb3;**
/// - `boxed` _(optional)_: The field is a `Box` and the inner value shall be mapped **&#xb3;**
/// - `box` _(optional)_: The derived field is a `Box` while the base field is not **&#xb3;**
/// - `unbox` _(optional)_: The base field is a `Box` while the derived field is not **&#xb3;**
/// - `with = mod::my_function` _(optional)_: If the field type doesn't implement `Into` or `TryInto` the derived field,
///   this property allows you to customize the behavior by providing a conversion function
/// - `from_with = mod::my_function` _(optional)_: The same as above but only for the `from` or `try_from` derives
/// - `into_with = mod::my_function` _(optional)_: The same as above but only for the `into` or `try_into` derives
///
/// **&#xb9;** When providing additional fields without defaults, the `From` and `TryFrom` traits can't be derived and
/// a custom function will be required instead. When deriving `into` or `try_into`, the `ty` must be provided as well.
///
/// **&#xb2;** When ignoring fields or variants it might be required that the enum or the struct implements `Default`
/// in order to properly populate it.
///
/// **&#xb3;** Hints can be nested, for example: `opt(vec)`, `vec(opt(with = "my_custom_fn"))`.
///
/// ### Scope & Evaluation Order
///
/// When writing custom mapping expressions (such as `with`, `from_with`, or `into_with`) or configuring skipped/added
/// fields with default values, the following scope and evaluation order behaviors apply:
///
/// #### Variable Scope & Namespaces
/// - **Source Fields:** The local variables in scope inside custom expressions correspond to the fields of the input
///   struct of the mapping:
///   - In `from` and `try_from` mappings, fields are referenced by their names in the **remote type** (`ty`).
///   - In `into` and `try_into` mappings, fields are referenced by their names in the **local type** (the struct
///     deriving `Mapper`).
/// - **Added Fields:** Added fields injected as custom function parameters are namespaced under the local `input`
///   structure. Access them using the `input.` prefix (e.g., `input.user_id`).
///
/// #### Evaluation & Consumption Order (Borrow-Before-Move)
/// To allow custom mapping expressions and default value expressions (e.g., `skip(default(value = ...))` or `add(field
/// = ..., default(value = ...))`) to safely reference or borrow other source fields before they are moved/consumed,
/// conversions are evaluated in a strict priority order:
/// 1. **Added fields** with default expressions are evaluated first.
/// 2. **Custom-mapped and skipped fields** with default expressions are evaluated next.
/// 3. **Regular fields** (which move/consume the source variables) are evaluated last.
///
/// ## Example
///
/// ```rs
/// #[derive(Mapper)]
/// #[mapper(from, ty = Entity)]
/// pub struct Model {
///     id: i64,
///     name: String,
///     #[mapper(skip(default))]
///     surname: Option<String>,
/// }
/// ```
///
/// Other advanced use cases are available on the [examples folder](https://github.com/lasantosr/model-mapper/tree/main/model-mapper/examples/).
#[proc_macro_derive(Mapper, attributes(mapper))]
pub fn model_mapper(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as syn::DeriveInput);
    model_mapper::r#impl(input).into()
}
