//! Concrete, shape-isolated target-centric domain model.
//!
//! This module represents mapping configurations using a shape-isolated
//! [`ItemMapping`] enum. This separates Struct mapping tasks from Enum mapping tasks
//! at the top level, while representing individual mapping flows as flat direction enums.
//! All struct fields are private to enforce invariants.

mod components;
mod config;
mod parsing;
mod typestate;

use core::fmt;
use std::collections::{HashMap, HashSet};

pub(crate) use components::*;
pub(crate) use config::*;
pub(crate) use typestate::*;

use crate::input::*;

/// Parses all mapping configurations from a [`MappingSpec`], verifying and returning
/// them as a mapping keyed by derived type key.
pub(crate) fn parse_mappings(spec: &MappingSpec) -> Result<HashMap<DerivedTypeKey, ItemMapping>, syn::Error> {
    let spec_derives: HashSet<DerivedTypeKey> = spec
        .derives
        .iter()
        .map(|derive| DerivedTypeKey::new(&derive.path))
        .collect();

    parsing::validate_spec_overrides(spec, &spec_derives)?;

    let mut mappings = HashMap::new();
    for derive in &spec.derives {
        let key = DerivedTypeKey::new(&derive.path);
        if mappings.contains_key(&key) {
            return Err(syn::Error::new(
                derive.path.span(),
                format!("This type is duplicated: '{key}'"),
            ));
        }
        let item_mapping = ItemMapping::try_new(spec, derive)?;
        mappings.insert(key, item_mapping);
    }
    Ok(mappings)
}

/// A normalized, comparable, and hashable key representing a target derived type path.
/// Generic arguments are stripped (e.g. `Type<T>` becomes `Type`) to permit precise lookup mapping.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct DerivedTypeKey(String);

impl DerivedTypeKey {
    /// Normalizes a type path (stripping generics) and constructs a new [`DerivedTypeKey`].
    pub(crate) fn new(path: &syn::TypePath) -> Self {
        let mut stripped = path.clone();
        for segment in &mut stripped.path.segments {
            segment.arguments = syn::PathArguments::None;
        }
        let key = quote::quote!(#stripped).to_string().replace(' ', "");
        Self(key)
    }
}

impl fmt::Display for DerivedTypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The domain model representing all mapping flows for a single target type.
pub(crate) enum ItemMapping {
    /// Mapping specifications for a struct base item.
    Struct {
        /// Shared mapping context.
        spec: MappingContext,
        /// List of active structural conversions.
        derives: Vec<MappingFlow<StructShape>>,
    },
    /// Mapping specifications for an enum base item.
    Enum {
        /// Shared mapping context.
        spec: MappingContext,
        /// List of active enum conversions.
        derives: Vec<MappingFlow<EnumShape>>,
    },
}

impl ItemMapping {
    /// Builds an [`ItemMapping`] representation for the given target specification and config,
    /// running all validation checks at the boundary.
    fn try_new(spec: &MappingSpec, derive_config: &DeriveConfig) -> Result<Self, syn::Error> {
        match &spec.data {
            BaseData::Struct(fields) => Self::try_new_struct(spec, derive_config, fields),
            BaseData::Enum(variants) => Self::try_new_enum(spec, derive_config, variants),
        }
    }

    fn try_new_struct(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        fields: &[BaseField],
    ) -> Result<Self, syn::Error> {
        let spec_inner = MappingContext {
            base_ident: spec.ident.clone(),
            base_generics: spec.generics.clone(),
            derived_path: (*derive_config.path).clone(),
        };

        for bf in fields {
            parsing::validate_field_against_derive(bf, derive_config)?;
        }
        let mut derives = Vec::new();
        if let Some(opts) = &derive_config.from {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Infallible(FromCustom::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Infallible(FromTrait::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::From(derive));
        }
        if let Some(opts) = &derive_config.try_from {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Fallible(TryFromCustom::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Fallible(TryFromTrait::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::From(derive));
        }
        if let Some(opts) = &derive_config.into {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Infallible(IntoCustom::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Infallible(IntoTrait::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::Into(derive));
        }
        if let Some(opts) = &derive_config.try_into {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Fallible(TryIntoCustom::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Fallible(TryIntoTrait::<StructShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::Into(derive));
        }

        if derives.is_empty() {
            return Err(syn::Error::new(
                derive_config.path.span(),
                "One of 'from', 'into', 'try_from' or 'try_into' must be set",
            ));
        }

        Ok(Self::Struct {
            spec: spec_inner,
            derives,
        })
    }

    fn try_new_enum(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        variants: &[BaseVariant],
    ) -> Result<Self, syn::Error> {
        let spec_inner = MappingContext {
            base_ident: spec.ident.clone(),
            base_generics: spec.generics.clone(),
            derived_path: (*derive_config.path).clone(),
        };

        for bv in variants {
            for bf in &bv.fields {
                parsing::validate_field_against_derive(bf, derive_config)?;
            }
        }
        let mut derives = Vec::new();
        if let Some(opts) = &derive_config.from {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Infallible(FromCustom::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Infallible(FromTrait::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::From(derive));
        }
        if let Some(opts) = &derive_config.try_from {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Fallible(TryFromCustom::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Fallible(TryFromTrait::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::From(derive));
        }
        if let Some(opts) = &derive_config.into {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Infallible(IntoCustom::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Infallible(IntoTrait::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::Into(derive));
        }
        if let Some(opts) = &derive_config.try_into {
            let derive = if opts.custom.is_some() {
                ImplStrategy::Custom(FallibilityStrategy::Fallible(TryIntoCustom::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            } else {
                ImplStrategy::Trait(FallibilityStrategy::Fallible(TryIntoTrait::<EnumShape>::try_new(
                    spec,
                    derive_config,
                    opts,
                )?))
            };
            derives.push(MappingFlow::Into(derive));
        }

        if derives.is_empty() {
            return Err(syn::Error::new(
                derive_config.path.span(),
                "One of 'from', 'into', 'try_from' or 'try_into' must be set",
            ));
        }

        Ok(Self::Enum {
            spec: spec_inner,
            derives,
        })
    }
}

/// Shared mapping context across all target mapping paths.
pub(crate) struct MappingContext {
    /// The identifier of the base struct/enum.
    base_ident: syn::Ident,
    /// The generics of the base struct/enum.
    base_generics: syn::Generics,
    /// The derived path to convert to or from.
    derived_path: syn::TypePath,
}

impl MappingContext {
    /// Gets the identifier of the base type.
    pub(crate) fn base_ident(&self) -> &syn::Ident {
        &self.base_ident
    }

    /// Gets the generics of the base type.
    pub(crate) fn base_generics(&self) -> &syn::Generics {
        &self.base_generics
    }

    /// Gets the derived path of the conversion.
    pub(crate) fn derived_path(&self) -> &syn::TypePath {
        &self.derived_path
    }
}

/// Mapping flows partitioned by the mapping direction.
pub(crate) enum MappingFlow<S: MappingShape> {
    /// Mapping in the `From` (Derived -> Base) direction.
    From(ImplStrategy<S, FromDirection>),
    /// Mapping in the `Into` (Base -> Derived) direction.
    Into(ImplStrategy<S, IntoDirection>),
}

/// Implementation strategies partitioned by the generated code style (trait impl vs custom function).
pub(crate) enum ImplStrategy<S: MappingShape, D>
where
    D: DirectionMode<TraitMode> + DirectionMode<CustomFnMode>,
{
    /// Standard trait implementations.
    Trait(FallibilityStrategy<S, D, TraitMode>),
    /// Custom named function generation.
    Custom(FallibilityStrategy<S, D, CustomFnMode>),
}

/// Fallibility strategies partitioned by the fallibility behavior of the conversion.
pub(crate) enum FallibilityStrategy<S: MappingShape, D: DirectionMode<I>, I: ImplMode> {
    /// Infallible conversion mappings.
    Infallible(S::Config<InfallibleMode, D, I>),
    /// Fallible conversion configurations.
    Fallible(S::Config<FallibleMode, D, I>),
}

// =========================================================================
// Generic typestate type aliases to simplify carrying generic arguments
// =========================================================================

/// Type alias for infallible Trait implementation in the From direction.
pub(crate) type FromTrait<S> = <S as MappingShape>::Config<InfallibleMode, FromDirection, TraitMode>;
/// Type alias for infallible Custom function implementation in the From direction.
pub(crate) type FromCustom<S> = <S as MappingShape>::Config<InfallibleMode, FromDirection, CustomFnMode>;

/// Type alias for fallible Trait implementation in the From direction.
pub(crate) type TryFromTrait<S> = <S as MappingShape>::Config<FallibleMode, FromDirection, TraitMode>;
/// Type alias for fallible Custom function implementation in the From direction.
pub(crate) type TryFromCustom<S> = <S as MappingShape>::Config<FallibleMode, FromDirection, CustomFnMode>;

/// Type alias for infallible Trait implementation in the Into direction.
pub(crate) type IntoTrait<S> = <S as MappingShape>::Config<InfallibleMode, IntoDirection, TraitMode>;
/// Type alias for infallible Custom function implementation in the Into direction.
pub(crate) type IntoCustom<S> = <S as MappingShape>::Config<InfallibleMode, IntoDirection, CustomFnMode>;

/// Type alias for fallible Trait implementation in the Into direction.
pub(crate) type TryIntoTrait<S> = <S as MappingShape>::Config<FallibleMode, IntoDirection, TraitMode>;
/// Type alias for fallible Custom function implementation in the Into direction.
pub(crate) type TryIntoCustom<S> = <S as MappingShape>::Config<FallibleMode, IntoDirection, CustomFnMode>;

#[cfg(test)]
mod tests {
    use proc_macro2::Span;

    use super::*;

    fn get_err<T>(res: Result<T, syn::Error>) -> syn::Error {
        match res {
            Ok(_) => panic!("expected Err, got Ok"),
            Err(err) => err,
        }
    }

    fn make_mock_derive() -> DeriveConfig {
        DeriveConfig {
            path: Located::new(syn::parse_quote!(Derived), Span::call_site()),
            from: None,
            into: None,
            try_from: None,
            try_into: None,
            add: Vec::new(),
            ignore_extra: None,
        }
    }

    fn make_mock_field(name: &str) -> BaseField {
        BaseField {
            ident: Some(syn::Ident::new(name, Span::call_site())),
            ty: syn::parse_quote!(String),
            mappings: FieldMappings::Default(Box::new(Located::new(FieldMapping::default(), Span::call_site()))),
        }
    }

    fn set_field_mapping(field: &mut BaseField, mapping: FieldMapping) {
        field.mappings = FieldMappings::Default(Box::new(Located::new(mapping, Span::call_site())));
    }

    #[test]
    fn accumulate_only_on_try() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: Some(Located::new(None, Span::call_site())),
        });

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "accumulate is only valid for try_from / try_into"
        );
    }

    #[test]
    fn err_exclusion() {
        let mut derive = make_mock_derive();
        derive.try_from = Some(ConversionOptions {
            custom: None,
            err: Some(syn::parse_quote!(Error)),
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        set_field_mapping(
            &mut field,
            FieldMapping {
                err: Some(syn::parse_quote!("error")),
                err_with: Some(syn::parse_quote!("error_with")),
                ..Default::default()
            },
        );

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(get_err(res).to_string(), "Only one of 'err' or 'err_with' can be set");
    }

    #[test]
    fn err_closure_check() {
        let mut derive = make_mock_derive();
        derive.try_from = Some(ConversionOptions {
            custom: None,
            err: Some(syn::parse_quote!(Error)),
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        set_field_mapping(
            &mut field,
            FieldMapping {
                err: Some(syn::parse_quote!(|_| "error")),
                ..Default::default()
            },
        );

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "Use 'err_with' instead of 'err' for closure-based error mapping"
        );
    }

    #[test]
    fn hint_count_check() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        set_field_mapping(
            &mut field,
            FieldMapping {
                hint: TransformHint {
                    with: Some(Located::new(syn::parse_quote!(with_fn), Span::call_site())),
                    opt: Some(Located::new(Box::new(TransformHint::default()), Span::call_site())),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "Only one of 'with', 'into_with'/'from_with', 'opt', 'iter', 'map', 'boxed', 'box' or 'unbox' can be set"
        );
    }

    #[test]
    fn into_with_not_allowed_on_from() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        set_field_mapping(
            &mut field,
            FieldMapping {
                hint: TransformHint {
                    into_with: Some(Located::new(syn::parse_quote!(into_fn), Span::call_site())),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "'into_with' is not allowed on a FROM/TRY_FROM mapping direction"
        );
    }

    #[test]
    fn from_with_not_allowed_on_into() {
        let mut derive = make_mock_derive();
        derive.into = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        set_field_mapping(
            &mut field,
            FieldMapping {
                hint: TransformHint {
                    from_with: Some(Located::new(syn::parse_quote!(from_fn), Span::call_site())),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "'from_with' is not allowed on an INTO/TRY_INTO mapping direction"
        );
    }

    #[test]
    fn skipped_field_no_default_on_from() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        set_field_mapping(
            &mut field,
            FieldMapping {
                skip: Some(Located::new(SkippedField { default: None }, Span::call_site())),
                ..Default::default()
            },
        );

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "Enable `default` here or include `custom` on `from` and `try_from` mappings"
        );
    }

    #[test]
    fn enum_add_illegal_attr() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });
        derive.add = vec![AddedField {
            field: Located::new(syn::Ident::new("extra", Span::call_site()), Span::call_site()),
            ty: Some(Located::new(syn::parse_quote!(u32), Span::call_site())),
            default: None,
        }];

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Enum(vec![]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(get_err(res).to_string(), "Illegal attribute for enums");
    }

    #[test]
    fn enum_add_no_default_on_from() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });
        derive.add = vec![AddedField {
            field: Located::new(syn::Ident::new("extra", Span::call_site()), Span::call_site()),
            ty: None,
            default: None,
        }];

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Enum(vec![]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "Missing mandatory `default` for enums when mapping `from` or `try_from`"
        );
    }

    #[test]
    fn duplicate_derive_type() {
        let mut derive1 = make_mock_derive();
        derive1.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });
        let mut derive2 = make_mock_derive();
        derive2.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![]),
            derives: vec![derive1.clone(), derive2.clone()],
        };

        let res = parse_mappings(&spec);
        assert!(res.is_err());
        assert_eq!(get_err(res).to_string(), "This type is duplicated: 'Derived'");
    }

    #[test]
    fn empty_derive_flags() {
        let derive = make_mock_derive(); // neither from, into, try_from, nor try_into is set
        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![]),
            derives: vec![derive.clone()],
        };

        let res = ItemMapping::try_new(&spec, &derive);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "One of 'from', 'into', 'try_from' or 'try_into' must be set"
        );
    }

    #[test]
    fn no_derive_defined_for_type_override() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        // Set an override for a path that is not "Derived" (the target of derive)
        let other_path: syn::TypePath = syn::parse_quote!(OtherDerived);
        field.mappings = FieldMappings::Overrides(vec![(
            other_path,
            Located::new(FieldMapping::default(), Span::call_site()),
        )]);

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = parse_mappings(&spec);
        assert!(res.is_err());
        assert_eq!(
            get_err(res).to_string(),
            "There is no derive defined for type: 'OtherDerived'"
        );
    }

    #[test]
    fn duplicate_override_type() {
        let mut derive = make_mock_derive();
        derive.from = Some(ConversionOptions {
            custom: None,
            err: None,
            accumulate: None,
        });

        let mut field = make_mock_field("field1");
        let derived_path: syn::TypePath = syn::parse_quote!(Derived);
        field.mappings = FieldMappings::Overrides(vec![
            (
                derived_path.clone(),
                Located::new(FieldMapping::default(), Span::call_site()),
            ),
            (
                derived_path.clone(),
                Located::new(FieldMapping::default(), Span::call_site()),
            ),
        ]);

        let spec = MappingSpec {
            ident: syn::Ident::new("Base", Span::call_site()),
            generics: syn::Generics::default(),
            data: BaseData::Struct(vec![field]),
            derives: vec![derive.clone()],
        };

        let res = parse_mappings(&spec);
        assert!(res.is_err());
        assert_eq!(get_err(res).to_string(), "This type is duplicated: 'Derived'");
    }
}
