//! Structural target mappings (`StructMapping` and `EnumMapping`) and their resolution implementation blocks.

use core::marker::PhantomData;

use super::*;
use crate::input::*;

/// Struct mapping representation.
pub(crate) struct StructMapping<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> {
    /// Resolved mappings for struct fields.
    pub(super) fields: Vec<MappedField<F, D, I>>,
    /// Deconstruct or synthesis rules specific to the direction.
    pub(super) extra: D::StructExtra,
    /// Whether a custom function or standard trait is generated.
    pub(super) impl_mode: I,
    /// The fallibility strategy details.
    pub(super) fallibility: F,
    pub(super) _marker: PhantomData<D>,
}

impl<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> StructMapping<F, D, I> {
    /// Gets the resolved field mappings for this mapping.
    pub(crate) fn fields(&self) -> &[MappedField<F, D, I>] {
        &self.fields
    }

    /// Gets the direction-specific extra rules for this mapping.
    pub(crate) fn extra(&self) -> &D::StructExtra {
        &self.extra
    }

    /// Gets the implementation mode strategy for this mapping.
    pub(crate) fn impl_mode(&self) -> &I {
        &self.impl_mode
    }

    /// Gets the fallibility strategy for this mapping.
    pub(crate) fn fallibility(&self) -> &F {
        &self.fallibility
    }
}

/// Enum mapping representation.
pub(crate) struct EnumMapping<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> {
    /// Resolved mappings for enum variants.
    pub(super) variants: Vec<MappedVariant<F, D, I>>,
    /// Deconstruct or synthesis rules specific to the direction.
    pub(super) extra: D::EnumExtra,
    /// Whether a custom function or standard trait is generated.
    pub(super) impl_mode: I,
    /// The fallibility strategy details.
    pub(super) fallibility: F,
    pub(super) _marker: PhantomData<D>,
}

impl<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> EnumMapping<F, D, I> {
    /// Gets the resolved variant mappings for this mapping.
    pub(crate) fn variants(&self) -> &[MappedVariant<F, D, I>] {
        &self.variants
    }

    /// Gets the direction-specific extra rules for this mapping.
    pub(crate) fn extra(&self) -> &D::EnumExtra {
        &self.extra
    }

    /// Gets the implementation mode strategy for this mapping.
    pub(crate) fn impl_mode(&self) -> &I {
        &self.impl_mode
    }

    /// Gets the fallibility strategy for this mapping.
    pub(crate) fn fallibility(&self) -> &F {
        &self.fallibility
    }
}

// --- MappedField concrete resolution implementations ---

impl MappedField<InfallibleMode, FromDirection, TraitMode> {
    pub(super) fn resolve(bf: &BaseField, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_field_attr(bf, derived_path);
        let transform = parsing::parse_transform(&attr.hint, true, attr.span())?;
        let skip = if let Some(skip) = &attr.skip {
            Some(Located::new(
                parsing::resolve_skip_from_trait(skip, skip.span())?,
                skip.span(),
            ))
        } else {
            None
        };

        Ok(Self {
            base_ident: bf.ident.clone(),
            base_ty: bf.ty.clone(),
            rename: attr.rename.clone(),
            other_ty: attr.other_ty.clone(),
            transform,
            error_handler: NoErrorHandler,
            skip,
        })
    }
}

impl MappedField<InfallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn resolve(bf: &BaseField, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_field_attr(bf, derived_path);
        let transform = parsing::parse_transform(&attr.hint, true, attr.span())?;
        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_from_custom(skip), skip.span()));

        Ok(Self {
            base_ident: bf.ident.clone(),
            base_ty: bf.ty.clone(),
            rename: attr.rename.clone(),
            other_ty: attr.other_ty.clone(),
            transform,
            error_handler: NoErrorHandler,
            skip,
        })
    }
}

impl MappedField<FallibleMode, FromDirection, TraitMode> {
    pub(super) fn resolve(bf: &BaseField, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_field_attr(bf, derived_path);
        let error_handler = parsing::resolve_fallible_error_handler(attr.err.as_ref(), attr.err_with.as_ref())?;
        let transform = parsing::parse_transform(&attr.hint, true, attr.span())?;
        let skip = if let Some(skip) = &attr.skip {
            Some(Located::new(
                parsing::resolve_skip_from_trait(skip, skip.span())?,
                skip.span(),
            ))
        } else {
            None
        };

        Ok(Self {
            base_ident: bf.ident.clone(),
            base_ty: bf.ty.clone(),
            rename: attr.rename.clone(),
            other_ty: attr.other_ty.clone(),
            transform,
            error_handler,
            skip,
        })
    }
}

impl MappedField<FallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn resolve(bf: &BaseField, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_field_attr(bf, derived_path);
        let error_handler = parsing::resolve_fallible_error_handler(attr.err.as_ref(), attr.err_with.as_ref())?;
        let transform = parsing::parse_transform(&attr.hint, true, attr.span())?;
        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_from_custom(skip), skip.span()));

        Ok(Self {
            base_ident: bf.ident.clone(),
            base_ty: bf.ty.clone(),
            rename: attr.rename.clone(),
            other_ty: attr.other_ty.clone(),
            transform,
            error_handler,
            skip,
        })
    }
}

impl<I: ImplMode> MappedField<InfallibleMode, IntoDirection, I> {
    pub(super) fn resolve(bf: &BaseField, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_field_attr(bf, derived_path);
        let transform = parsing::parse_transform(&attr.hint, false, attr.span())?;
        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_into(skip), skip.span()));

        Ok(Self {
            base_ident: bf.ident.clone(),
            base_ty: bf.ty.clone(),
            rename: attr.rename.clone(),
            other_ty: attr.other_ty.clone(),
            transform,
            error_handler: NoErrorHandler,
            skip,
        })
    }
}

impl<I: ImplMode> MappedField<FallibleMode, IntoDirection, I> {
    pub(super) fn resolve(bf: &BaseField, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_field_attr(bf, derived_path);
        let error_handler = parsing::resolve_fallible_error_handler(attr.err.as_ref(), attr.err_with.as_ref())?;
        let transform = parsing::parse_transform(&attr.hint, false, attr.span())?;
        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_into(skip), skip.span()));

        Ok(Self {
            base_ident: bf.ident.clone(),
            base_ty: bf.ty.clone(),
            rename: attr.rename.clone(),
            other_ty: attr.other_ty.clone(),
            transform,
            error_handler,
            skip,
        })
    }
}

// --- MappedVariant concrete resolution implementations ---

impl MappedVariant<InfallibleMode, FromDirection, TraitMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);
        let skip = if let Some(skip) = &attr.skip {
            Some(Located::new(
                parsing::resolve_skip_from_trait(skip, skip.span())?,
                skip.span(),
            ))
        } else {
            None
        };

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<InfallibleMode, FromDirection, TraitMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: VariantDeconstructRules {
                ignore_extra: attr.ignore_extra,
                captured_fields: parsing::make_captured_fields(&attr.add),
            },
        })
    }
}

impl MappedVariant<InfallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);
        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_from_custom(skip), skip.span()));

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<InfallibleMode, FromDirection, CustomFnMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: VariantDeconstructRules {
                ignore_extra: attr.ignore_extra,
                captured_fields: parsing::make_captured_fields(&attr.add),
            },
        })
    }
}

impl MappedVariant<FallibleMode, FromDirection, TraitMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);
        let skip = if let Some(skip) = &attr.skip {
            Some(Located::new(
                parsing::resolve_skip_from_trait(skip, skip.span())?,
                skip.span(),
            ))
        } else {
            None
        };

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<FallibleMode, FromDirection, TraitMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: VariantDeconstructRules {
                ignore_extra: attr.ignore_extra,
                captured_fields: parsing::make_captured_fields(&attr.add),
            },
        })
    }
}

impl MappedVariant<FallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);
        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_from_custom(skip), skip.span()));

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<FallibleMode, FromDirection, CustomFnMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: VariantDeconstructRules {
                ignore_extra: attr.ignore_extra,
                captured_fields: parsing::make_captured_fields(&attr.add),
            },
        })
    }
}

impl MappedVariant<InfallibleMode, IntoDirection, TraitMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);

        if let Some(skip) = &attr.skip
            && skip.default.is_none()
        {
            return Err(skip.error("Enable `default` here required for `into` and `try_into` mappings"));
        }

        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_into(skip), skip.span()));

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<InfallibleMode, IntoDirection, TraitMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        let added_fields = parsing::resolve_into_trait_added_fields(&attr.add, attr.ignore_extra)?;

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: added_fields,
        })
    }
}

impl MappedVariant<InfallibleMode, IntoDirection, CustomFnMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);

        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_into(skip), skip.span()));

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<InfallibleMode, IntoDirection, CustomFnMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        let added_fields = parsing::resolve_into_custom_added_fields(&attr.add, attr.ignore_extra)?;

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: added_fields,
        })
    }
}

impl MappedVariant<FallibleMode, IntoDirection, TraitMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);

        if let Some(skip) = &attr.skip
            && skip.default.is_none()
        {
            return Err(skip.error("Enable `default` here required for `into` and `try_into` mappings"));
        }

        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_into(skip), skip.span()));

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<FallibleMode, IntoDirection, TraitMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        let added_fields = parsing::resolve_into_trait_added_fields(&attr.add, attr.ignore_extra)?;

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: added_fields,
        })
    }
}

impl MappedVariant<FallibleMode, IntoDirection, CustomFnMode> {
    pub(super) fn resolve(bv: &BaseVariant, derived_path: &syn::TypePath) -> Result<Self, syn::Error> {
        let attr = parsing::resolve_variant_attr(bv, derived_path);

        let skip = attr
            .skip
            .as_ref()
            .map(|skip| Located::new(parsing::resolve_skip_into(skip), skip.span()));

        let mut fields = Vec::new();
        for bf in &bv.fields {
            fields.push(MappedField::<FallibleMode, IntoDirection, CustomFnMode>::resolve(
                bf,
                derived_path,
            )?);
        }

        let added_fields = parsing::resolve_into_custom_added_fields(&attr.add, attr.ignore_extra)?;

        Ok(Self {
            ident: bv.ident.clone(),
            fields,
            rename: attr.rename.clone(),
            skip,
            extra: added_fields,
        })
    }
}

// --- StructMapping concrete resolution implementations ---

impl StructMapping<InfallibleMode, FromDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        for added in &derive_config.add {
            if added.default.is_none() {
                return Err(added
                    .field
                    .error("Enable `default` here or include `custom` on `from` and `try_from` mappings"));
            }
        }

        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<InfallibleMode, FromDirection, TraitMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra = StructDeconstructRules {
            captured_fields: parsing::make_captured_fields(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        Ok(Self {
            fields,
            extra,
            impl_mode: TraitMode,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<InfallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<InfallibleMode, FromDirection, CustomFnMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra = StructDeconstructRules {
            captured_fields: parsing::make_captured_fields(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            fields,
            extra,
            impl_mode: custom,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<FallibleMode, FromDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        for added in &derive_config.add {
            if added.default.is_none() {
                return Err(added
                    .field
                    .error("Enable `default` here or include `custom` on `from` and `try_from` mappings"));
            }
        }

        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<FallibleMode, FromDirection, TraitMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra = StructDeconstructRules {
            captured_fields: parsing::make_captured_fields(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        let fallibility = parsing::resolve_fallibility_mapping(opts);

        Ok(Self {
            fields,
            extra,
            impl_mode: TraitMode,
            fallibility,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<FallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<FallibleMode, FromDirection, CustomFnMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra = StructDeconstructRules {
            captured_fields: parsing::make_captured_fields(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        let fallibility = parsing::resolve_fallibility_mapping(opts);
        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            fields,
            extra,
            impl_mode: custom,
            fallibility,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<InfallibleMode, IntoDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<InfallibleMode, IntoDirection, TraitMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra = parsing::resolve_into_trait_added_fields(&derive_config.add, derive_config.ignore_extra.is_some())?;

        Ok(Self {
            fields,
            extra,
            impl_mode: TraitMode,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<InfallibleMode, IntoDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<InfallibleMode, IntoDirection, CustomFnMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra =
            parsing::resolve_into_custom_added_fields(&derive_config.add, derive_config.ignore_extra.is_some())?;
        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            fields,
            extra,
            impl_mode: custom,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<FallibleMode, IntoDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<FallibleMode, IntoDirection, TraitMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra = parsing::resolve_into_trait_added_fields(&derive_config.add, derive_config.ignore_extra.is_some())?;
        let fallibility = parsing::resolve_fallibility_mapping(opts);

        Ok(Self {
            fields,
            extra,
            impl_mode: TraitMode,
            fallibility,
            _marker: PhantomData,
        })
    }
}

impl StructMapping<FallibleMode, IntoDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        let BaseData::Struct(base_fields) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected a struct mapping, found enum",
            ));
        };

        let mut fields = Vec::new();
        for bf in base_fields {
            fields.push(MappedField::<FallibleMode, IntoDirection, CustomFnMode>::resolve(
                bf,
                &derive_config.path,
            )?);
        }

        let extra =
            parsing::resolve_into_custom_added_fields(&derive_config.add, derive_config.ignore_extra.is_some())?;
        let fallibility = parsing::resolve_fallibility_mapping(opts);
        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            fields,
            extra,
            impl_mode: custom,
            fallibility,
            _marker: PhantomData,
        })
    }
}

// --- EnumMapping concrete resolution implementations ---

impl EnumMapping<InfallibleMode, FromDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        parsing::validate_enum_add_from(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<InfallibleMode, FromDirection, TraitMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let extra = EnumDeconstructRules {
            captured_variants: parsing::make_captured_variants(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        Ok(Self {
            variants,
            extra,
            impl_mode: TraitMode,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<InfallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        parsing::validate_enum_add_from(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<InfallibleMode, FromDirection, CustomFnMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let extra = EnumDeconstructRules {
            captured_variants: parsing::make_captured_variants(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            variants,
            extra,
            impl_mode: custom,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<FallibleMode, FromDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        parsing::validate_enum_add_from(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<FallibleMode, FromDirection, TraitMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let extra = EnumDeconstructRules {
            captured_variants: parsing::make_captured_variants(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        let fallibility = parsing::resolve_fallibility_mapping(opts);

        Ok(Self {
            variants,
            extra,
            impl_mode: TraitMode,
            fallibility,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<FallibleMode, FromDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        parsing::validate_enum_add_from(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<FallibleMode, FromDirection, CustomFnMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let extra = EnumDeconstructRules {
            captured_variants: parsing::make_captured_variants(&derive_config.add),
            ignore_extra: derive_config.ignore_extra.is_some(),
        };

        let fallibility = parsing::resolve_fallibility_mapping(opts);
        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            variants,
            extra,
            impl_mode: custom,
            fallibility,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<InfallibleMode, IntoDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        parsing::validate_enum_add_into(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<InfallibleMode, IntoDirection, TraitMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        Ok(Self {
            variants,
            extra: (),
            impl_mode: TraitMode,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<InfallibleMode, IntoDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        if let Some(acc) = &opts.accumulate {
            return Err(acc.error("accumulate is only valid for try_from / try_into"));
        }

        parsing::validate_enum_add_into(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<InfallibleMode, IntoDirection, CustomFnMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            variants,
            extra: (),
            impl_mode: custom,
            fallibility: InfallibleMode,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<FallibleMode, IntoDirection, TraitMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        parsing::validate_enum_add_into(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<FallibleMode, IntoDirection, TraitMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let fallibility = parsing::resolve_fallibility_mapping(opts);

        Ok(Self {
            variants,
            extra: (),
            impl_mode: TraitMode,
            fallibility,
            _marker: PhantomData,
        })
    }
}

impl EnumMapping<FallibleMode, IntoDirection, CustomFnMode> {
    pub(super) fn try_new(
        spec: &MappingSpec,
        derive_config: &DeriveConfig,
        opts: &ConversionOptions,
    ) -> Result<Self, syn::Error> {
        parsing::validate_enum_add_into(&derive_config.add)?;

        let BaseData::Enum(base_variants) = &spec.data else {
            return Err(syn::Error::new_spanned(
                &spec.ident,
                "Expected an enum mapping, found struct",
            ));
        };

        let mut variants = Vec::new();
        for bv in base_variants {
            variants.push(MappedVariant::<FallibleMode, IntoDirection, CustomFnMode>::resolve(
                bv,
                &derive_config.path,
            )?);
        }

        let fallibility = parsing::resolve_fallibility_mapping(opts);
        let custom = CustomFnMode {
            fn_name: opts.custom.as_ref().and_then(|idn| (**idn).clone()),
        };

        Ok(Self {
            variants,
            extra: (),
            impl_mode: custom,
            fallibility,
            _marker: PhantomData,
        })
    }
}
