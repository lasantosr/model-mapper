# Model Mapper

This library provides a macro to implement `From` and/or `TryFrom` trait between types (both enums and structs) without boilerplate.

It also provides a `with` module containing some utilities to convert between types that does not implement `Into` trait.

```rs
#[derive(Clone, Default, Mapper)]
#[mapper(from, ty = Entity)]
pub struct Model {
    id: i64,
    name: String,
    age: Option<i64>,
}
```

## Usage

A `mapper` attribute is required at type-level and it's optional at field or variant level.

The following attributes are available.

- Type level attributes:

  - `ty = PathType` _(**mandatory**)_: The other type to derive the conversion
  - `ignore = field_1` _(optional, multiple)_: Additional fields (for structs with named fields) or variants (for
    enums) the other type has and this one doesn't \*
  - `from` _(optional)_: Wether to derive `From` the other type for self
  - `into` _(optional)_: Wether to derive `From` self for the other type
  - `try_from` _(optional)_: Wether to derive `TryFrom` the other type for self
  - `try_into` _(optional)_: Wether to derive `TryFrom` self for the other type

- Variant level attributes:

  - `ignore = field_1` _(optional, multiple)_: Additional variants that the other enum has and this one doesn't \*
  - `skip` _(optional)_: Wether to skip this variant because the other enum doesn't have it \*
  - `rename = "OtherVariant"` _(optional)_: To rename this variant on the other enum

- Field level attributes:

  - `skip` _(optional)_: Wether to skip this field because the other type doesn't have it \*
  - `rename = "other_field"` _(optional)_: To rename this field on the other type
  - `with = mod::my_function` _(optional)_: If the field type doesn't implement `Into` the other, this property allows
    you to customize the behavior by providing a conversion function
  - `try_with = mod::my_function` _(optional)_: If the field type doesn't implement `TryInto` the other, this property
    allows you to customize the behavior by providing a conversion function

**\*** When ignoring or skipping fields or variants it might be required that the enum or the field type implements
`Default` in order to properly populate it.

Attributes can be set directly if only one type is involved in the conversion:

```rs
#[mapper(from, into, ty = OtherType, ignore = field_1, ignore = field_2)]
```

Or they can be wrapped in a `derive` attribute to allow for multiple conversions:

```rs
#[mapper(derive(try_from, ty = OtherType, ignore = field_1))]
#[mapper(derive(into, ty = YetAnotherType))]
```

If multiple conversions are involved, both variant and field level attributes can also be wrapped in a `when` attribute
and must set the `ty` they refer to:

```rs
#[mapper(when(ty = OtherType, try_with = with::try_remove_option))]
#[mapper(when(ty = YetAnotherType, skip))]
```

## Example

```rs
#[derive(Default)]
pub enum Model {
    #[default]
    Empty,
    Text(String),
    Data {
        id: i64,
        text: String,
        status: Option<i32>,
        internal: bool,
    },
    Unknown,
}

#[derive(Default, Mapper)]
#[mapper(try_from, into, ty = Model, ignore = Unknown)]
pub enum Entity {
    #[default]
    Empty,
    #[mapper(rename = Text)]
    Message(String),
    #[mapper(ignore = internal)]
    Data {
        id: i64,
        #[mapper(rename = "text")]
        message: String,
        #[mapper(with = with::option)]
        #[mapper(try_with = with::try_option)]
        status: Option<i16>,
        #[mapper(skip)]
        random: bool,
    },
    #[mapper(skip)]
    Error,
}
```

There are more examples available in the integration tests.
