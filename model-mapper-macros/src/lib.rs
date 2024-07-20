mod input;
mod model_mapper;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// Derive mapper functions to convert between types.
///
/// A `mapper` attribute is required at type-level and it's optional at field or variant level.
///
/// The following attributes are available:
///
/// #### Type level attributes
///
/// - `ty = PathType` _(**mandatory**)_: The other type to derive the conversion
/// - `from` _(optional)_: Whether to derive `From` the other type for self
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
/// - `into` _(optional)_: Whether to derive `From` self for the other type
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
/// - `try_from` _(optional)_: Whether to derive `TryFrom` the other type for self
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
/// - `try_into` _(optional)_: Whether to derive `TryFrom` self for the other type
///   - `custom` _(optional)_: Derive a custom function instead of the trait
///   - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
/// - `add` _(optional, multiple)_: Additional fields (for structs with named fields) or variants (for enums) the other
///   type has and this one doesn't **&#x00b9;**
///   - `field = other_field` _(mandatory)_: The field or variant name
///   - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
///   - `default` _(optional)_: The field or variant will be populated using `Default::default()` (mandatory for enums,
///     with or without value)
///     - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
/// - `ignore_extra` _(optional)_: Whether to ignore all extra fields (for structs) or variants (for enums) of the other
///   type **&#x00b2;**
///
/// #### Variant level attributes
///
/// - `rename = OtherVariant` _(optional)_: To rename this variant on the other enum
/// - `add` _(optional, multiple)_: Additional fields of the variant that the other type variant has and this one
///   doesn't **&#x00b9;**
///   - `field = other_field` _(mandatory)_: The field name
///   - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
///   - `default` _(optional)_: The field or variant will be populated using `Default::default()`
///     - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
/// - `skip` _(optional)_: Whether to skip this variant because the other enum doesn't have it
///   - `default` _(mandatory)_: The field or variant will be populated using `Default::default()`
///     - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression
///       instead
/// - `ignore_extra` _(optional)_: Whether to ignore all extra fields of the other variant (only valid for _from_ and
///   _try_from_) **&#x00b2;**
///
/// #### Field level attributes
///
/// - `rename = other_name` _(optional)_: To rename this field on the other type
/// - `skip` _(optional)_: Whether to skip this field because the other type doesn't have it
///   - `default` _(optional)_: The field or variant will be populated using `Default::default()`
///     - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression
///       instead
/// - `with = mod::my_function` _(optional)_: If the field type doesn't implement `Into` or `TryInto` the other, this
///   property allows you to customize the behavior by providing a conversion function
/// - `into_with = mod::my_function` _(optional)_: The same as above but only for the `into` or `try_into` derives
/// - `from_with = mod::my_function` _(optional)_: The same as above but only for the `from` or `try_from` derives
///
/// **&#x00b9;** When providing additional fields without defaults, the `From` and `TryFrom` traits can't be derived and
/// a custom function will be required instead. When deriving `into` or `try_into`, the `ty` must be provided as well.
///
/// **&#x00b2;** When ignoring fields or variants it might be required that the enum or the struct implements `Default`
/// in order to properly populate it.
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
#[proc_macro_error]
#[proc_macro_derive(Mapper, attributes(mapper))]
pub fn model_mapper(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    model_mapper::r#impl(input).into()
}
