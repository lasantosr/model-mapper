mod input;
mod model_mapper;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// Derives [From] and/or [TryFrom] another type to this one, or vice versa.
///
/// A `mapper` attribute is required at type-level and it's optional at field or variant level.
///
/// Attributes can be set directly if only one type is involved in the conversion:
/// ``` ignore
/// #[mapper(from, into, ty = OtherType, ignore(field = field_1), ignore(field = field_2))]
/// ```
/// Or they can be wrapped in a `derive` attribute to allow for multiple types:
/// ``` ignore
/// #[mapper(derive(try_from, ty = OtherType, ignore(field = field_1)))]
/// #[mapper(derive(into, ty = YetAnotherType))]
/// ```
///
/// If multiple types are involved, both variant and field level attributes can also be wrapped in a `when` attribute
/// and must set the `ty` they refer to:
/// ``` ignore
/// #[mapper(when(ty = OtherType, try_with = with::try_remove_option))]
/// #[mapper(when(ty = YetAnotherType, skip))]
/// ```
///
/// The following attributes are available:
///
/// #### Type level attributes
/// - `ty = String` _(**mandatory**)_: The other type to derive the conversion
/// - `ignore` _(optional, multiple)_: Additional fields (for structs with named fields) or variants (for enums) the
///   other type has and this one doesn't \*
///   - `field = String` _(mandatory)_: The field or variant to ignore
///   - `default = Expr` _(optional)_: The default value (defaults to `Default::default()`)
/// - `from` _(optional)_: Wether to derive [From] the other type for self
/// - `into` _(optional)_: Wether to derive [From] self for the other type
/// - `try_from` _(optional)_: Wether to derive [TryFrom] the other type for self
/// - `try_into` _(optional)_: Wether to derive [TryFrom] self for the other type
///
/// #### Variant level attributes
/// - `ignore` _(optional, multiple)_: Additional fields of the variant that the other type variant has and this one
///   doesn't \*
///   - `field = String` _(mandatory)_: The field or variant to ignore
///   - `default = Expr` _(optional)_: The default value (defaults to `Default::default()`)
/// - `skip` _(optional)_: Wether to skip this variant because the other enum doesn't have it \*
/// - `default = Expr` _(optional)_: If skipped, the default value to populate this variant  (defaults to
///   `Default::default()`)
/// - `rename = "OtherVariant"` _(optional)_: To rename this variant on the other enum
///
/// #### Field level attributes
/// - `skip` _(optional)_: Wether to skip this field because the other type doesn't have it \*
/// - `default = Expr` _(optional)_: If skipped, the default value to populate this field  (defaults to
///   `Default::default()`)
/// - `rename = "other_field"` _(optional)_: To rename this field on the other type
/// - `with = mod::my_function` _(optional)_: If the field type doesn't implement [Into] the other, this property allows
///   you to customize the behavior by providing a conversion function
/// - `try_with = mod::my_function` _(optional)_: If the field type doesn't implement [TryInto] the other, this property
///   allows you to customize the behavior by providing a conversion function
///
/// **\*** When ignoring or skipping fields or variants it might be required that the enum or the field type implements
/// [Default] in order to properly populate it if no default is provided.
///
/// ## Examples
///
/// ```
/// # use model_mapper_macros::Mapper;
/// # #[derive(Default)]
/// # pub enum ResponseModel {
/// #     #[default]
/// #     Empty,
/// #     Text(String),
/// #     Data {
/// #         id: i64,
/// #         text: String,
/// #         status: Option<i32>,
/// #         internal: bool,
/// #     },
/// #     Unknown,
/// # }
/// # mod with {
/// #     pub fn option<F,I>(opt: Option<F>) -> Option<I> { unreachable!() }
/// #     pub fn try_option<F,I>(opt: Option<F>) -> Result<Option<I>, std::io::Error> { unreachable!() }
/// # }
/// #[derive(Mapper)]
/// #[mapper(try_from, into, ty = ResponseModel, ignore(field = Unknown, default = CustomResponse::Empty))]
/// pub enum CustomResponse {
///     Empty,
///     #[mapper(rename = Text)]
///     Message(String),
///     #[mapper(ignore(field = internal))]
///     Data {
///         id: i64,
///         #[mapper(rename = "text")]
///         message: String,
///         #[mapper(with = with::option)]
///         #[mapper(try_with = with::try_option)]
///         status: Option<i16>,
///         #[mapper(skip)]
///         random: bool,
///     },
///     #[mapper(skip)]
///     Error,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(Mapper, attributes(mapper))]
pub fn model_mapper(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    model_mapper::r#impl(input).into()
}
