# Model Mapper

This library provides a macro to implement functions to convert between types (both enums and structs) without boilerplate.

It also provides a `with` module containing some utilities to convert between types that does not implement `Into` trait.

As long as you don't use the `with` module (disable default features) and don't derive `try_into` or `try_form`, this
lib can be used in `#![no_std]` crates.

## Examples

The most common use case for this crate is to map between domain entities on services and externally-faced models or DTOs.

```rs
#[derive(Mapper)]
#[mapper(from, ty = Entity)]
pub struct Model {
    id: i64,
    name: String,
}
```

The macro expansion above would generate something like:

```rs
impl From<Entity> for Model {
    fn from(Entity { id, name }: Entity) -> Self {
        Self {
            id: Into::into(id),
            name: Into::into(name),
        }
    }
}
```

Because types doesn't always fit like a glove, you can provide additional fields on runtime, at the cost of not being
able to use the `From` trait:

```rs
pub mod service {
    pub struct UpdateUserInput {
        pub user_id: i64,
        pub name: Option<String>,
        pub surname: Option<String>,
    }
}

#[derive(Mapper)]
#[mapper(
    into(custom = "into_update_user"),
    ty = service::UpdateUserInput,
    add(field = user_id, ty = i64),
    add(field = surname, default(value = None))
)]
pub struct UpdateProfileRequest {
    pub name: String,
}
```

Would generate something like:

```rs
impl UpdateProfileRequest {
    /// Builds a new [service::UpdateUserInput] from a [UpdateProfileRequest]
    pub fn into_update_user(self, user_id: i64) -> service::UpdateUserInput {
        let UpdateProfileRequest { name } = self;
        service::UpdateUserInput {
            user_id,
            surname: None,
            name: Into::into(name),
        }
    }
}
```

Other advanced use cases are available on the [examples folder](./model-mapper/examples/).

## Usage

A `mapper` attribute is required at type-level and it's optional at field or variant level.

The following attributes are available.

- Type level attributes:

  - `ty = PathType` _(**mandatory**)_: The other type to derive the conversion
  - `from` _(optional)_: Whether to derive `From` the other type for self
    - `custom` _(optional)_: Derive a custom function instead of the trait
    - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
  - `into` _(optional)_: Whether to derive `From` self for the other type
    - `custom` _(optional)_: Derive a custom function instead of the trait
    - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
  - `try_from` _(optional)_: Whether to derive `TryFrom` the other type for self
    - `custom` _(optional)_: Derive a custom function instead of the trait
    - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
  - `try_into` _(optional)_: Whether to derive `TryFrom` self for the other type
    - `custom` _(optional)_: Derive a custom function instead of the trait
    - `custom = from_other` _(optional)_: Derive a custom function instead of the trait, with the given name
  - `add` _(optional, multiple)_: Additional fields (for structs with named fields) or variants (for enums) the
    other type has and this one doesn't **&#xb9;**
    - `field = other_field` _(mandatory)_: The field or variant name
    - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
    - `default` _(optional)_: The field or variant will be populated using `Default::default()` (mandatory for enums,
      with or without value)
      - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
  - `ignore_extra` _(optional)_: Whether to ignore all extra fields (for structs) or variants (for enums) of the other
    type **&#xb2;**

- Variant level attributes:

  - `rename = OtherVariant` _(optional)_: To rename this variant on the other enum
  - `add` _(optional, multiple)_: Additional fields of the variant that the other type variant has and this one
    doesn't **&#xb9;**
    - `field = other_field` _(mandatory)_: The field name
    - `ty = bool` _(optional)_: The field type, mandatory for `into` and `try_into` if no default value is provided
    - `default` _(optional)_: The field or variant will be populated using `Default::default()`
      - `value = true` _(optional)_: The field or variant will be populated with the given expression instead
  - `skip` _(optional)_: Whether to skip this variant because the other enum doesn't have it
    - `default` _(mandatory)_: The field or variant will be populated using `Default::default()`
      - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression instead
  - `ignore_extra` _(optional)_: Whether to ignore all extra fields of the other variant (only valid for _from_ and
    _try_from_) **&#xb2;**

- Field level attributes:

  - `rename = other_name` _(optional)_: To rename this field on the other type
  - `skip` _(optional)_: Whether to skip this field because the other type doesn't have it
    - `default` _(optional)_: The field or variant will be populated using `Default::default()`
      - `value = get_default_value()` _(optional)_: The field or variant will be populated with the given expression instead
  - `with = mod::my_function` _(optional)_: If the field type doesn't implement `Into` or `TryInto` the other, this
    property allows you to customize the behavior by providing a conversion function
  - `into_with = mod::my_function` _(optional)_: The same as above but only for the `into` or `try_into` derives
  - `from_with = mod::my_function` _(optional)_: The same as above but only for the `from` or `try_from` derives

- Additional hints on how to map fields:

  - `opt` _(optional)_: The field is an `Option` and the inner value shall be mapped **&#xb3;**
  - `iter` _(optional)_: The field is an iterator and the inner value shall be mapped **&#xb3;**
  - `map` _(optional)_: The field is a hashmap-like iterator and the inner value shall be mapped **&#xb3;**
  - `with = mod::my_function` _(optional)_: If the field type doesn't implement `Into` or `TryInto` the other, this
    property allows you to customize the behavior by providing a conversion function
  - `from_with = mod::my_function` _(optional)_: The same as above but only for the `from` or `try_from` derives
  - `from_with = mod::my_function` _(optional)_: The same as above but only for the `from` or `try_from` derives

**&#xb9;** When providing additional fields without defaults, the `From` and `TryFrom` traits can't be derived and
a custom function will be required instead. When deriving `into` or `try_into`, the `ty` must be provided as well.

**&#xb2;** When ignoring fields or variants it might be required that the enum or the struct implements `Default`
in order to properly populate it.

**&#xb3;** Hints can be nested, for example: `opt(vec)`, `vec(opt(with = "my_custom_fn"))`

### Multiple derives

When deriving conversions for a single type, attributes can be set directly:

```rs
#[mapper(from, into, ty = OtherType, add(field = field_1, default), add(field = field_2, default))]
```

But we can also derive conversions for multiple types by wrapping the properties on a `derive` attribute:

```rs
#[mapper(derive(try_into, ty = OtherType, add(field = field_1, default)))]
#[mapper(derive(from, ty = YetAnotherType))]
```

If multiple conversions are involved, both variant and field level attributes can also be wrapped in a `when` attribute
and must set the `ty` they refer to:

```rs
#[mapper(when(ty = OtherType, with = TryIntoMapper::try_map_removing_option))]
#[mapper(when(ty = YetAnotherType, skip(default)))]
```
