//! Internal parsing and validation helper functions for attributes, resolved transforms, and resolved error handlers.

use syn::spanned::Spanned;

use super::*;

/// Checks if two [`syn::TypePath`] structures refer to the same type, ignoring leading colons
/// and comparing segment identifiers.
pub(super) fn matches_path(lhs: &syn::TypePath, rhs: &syn::TypePath) -> bool {
    DerivedTypeKey::new(lhs) == DerivedTypeKey::new(rhs)
}

/// Resolves error mapping expressions (`err` or `err_with`) for fallible mapping flows,
/// returning the corresponding [`ResolvedErrorHandler`].
pub(super) fn resolve_fallible_error_handler(
    err: Option<&syn::Expr>,
    err_with: Option<&syn::Expr>,
) -> Result<Option<ResolvedErrorHandler>, syn::Error> {
    match (err, err_with) {
        (Some(err), Some(_)) => Err(syn::Error::new(
            err.span(),
            "Only one of 'err' or 'err_with' can be set",
        )),
        (Some(err), None) => {
            if let syn::Expr::Closure(_) = err {
                return Err(syn::Error::new(
                    err.span(),
                    "Use 'err_with' instead of 'err' for closure-based error mapping",
                ));
            }
            Ok(Some(ResolvedErrorHandler::Value(Located::new(err.clone(), err.span()))))
        }
        (None, Some(err_with)) => Ok(Some(ResolvedErrorHandler::With(Located::new(
            err_with.clone(),
            err_with.span(),
        )))),
        (None, None) => Ok(None),
    }
}

/// Resolves standard trait skip configurations, ensuring a default expression is present.
pub(super) fn resolve_skip_from_trait(
    skip: &SkippedField,
    skip_span: proc_macro2::Span,
) -> Result<ResolvedSkipTrait, syn::Error> {
    if let Some(default_expr) = &skip.default {
        Ok(ResolvedSkipTrait {
            default_expr: Located::new((*default_expr.value()).clone(), default_expr.span()),
        })
    } else {
        Err(syn::Error::new(
            skip_span,
            "Enable `default` here or include `custom` on `from` and `try_from` mappings",
        ))
    }
}

/// Resolves custom mapping skip configurations, permitting skipped fields without default expressions.
pub(super) fn resolve_skip_from_custom(skip: &SkippedField) -> CustomSkippedFrom {
    CustomSkippedFrom {
        default_expr: skip
            .default
            .as_ref()
            .map(|expr| Located::new((*expr.value()).clone(), expr.span())),
    }
}

/// Resolves skip configurations for mappings targeting another type (Into/TryInto directions).
pub(super) fn resolve_skip_into(skip: &SkippedField) -> SkippedInto {
    SkippedInto {
        default: skip
            .default
            .as_ref()
            .map(|expr| Located::new((*expr.value()).clone(), expr.span())),
    }
}

/// Resolves added fields for standard trait-based mappings, validating that all fields provide default expressions.
pub(super) fn resolve_into_trait_added_fields(
    add: &[AddedField],
    ignore_extra: bool,
) -> Result<TraitSynthesizedFields, syn::Error> {
    let mut fields = Vec::new();
    for added in add {
        if let Some(default) = &added.default {
            fields.push(TraitAddedMappedField {
                field: added.field.clone(),
                default: default.clone(),
            });
        } else {
            return Err(added
                .field
                .error("Enable `default` here or include `custom` on `into` and `try_into` mappings"));
        }
    }
    Ok(TraitSynthesizedFields { fields, ignore_extra })
}

/// Resolves added fields for custom mapping functions, validating that each added field specifies either a default
/// expression or a type.
pub(super) fn resolve_into_custom_added_fields(
    add: &[AddedField],
    ignore_extra: bool,
) -> Result<CustomSynthesizedFields, syn::Error> {
    let mut fields = Vec::new();
    for added in add {
        if let Some(default) = &added.default {
            fields.push(CustomAddedMappedField::WithDefault {
                field: added.field.clone(),
                default: default.clone(),
            });
        } else if let Some(ty) = &added.ty {
            fields.push(CustomAddedMappedField::WithType {
                field: added.field.clone(),
                ty: ty.clone(),
            });
        } else {
            return Err(added
                .field
                .error("Provide a field type with `ty` if the field is not `default` to map `into` and `try_into`"));
        }
    }
    Ok(CustomSynthesizedFields { fields, ignore_extra })
}

/// Asserts that added variants mapping from another type are valid (no type paths allowed, defaults are mandatory).
pub(super) fn validate_enum_add_from(add: &[AddedField]) -> Result<(), syn::Error> {
    for added in add {
        if added.ty.is_some() {
            let err_span = added
                .ty
                .as_ref()
                .map(Located::span)
                .unwrap_or_else(|| added.field.span());
            return Err(syn::Error::new(err_span, "Illegal attribute for enums"));
        }
        if added.default.is_none() {
            return Err(added
                .field
                .error("Missing mandatory `default` for enums when mapping `from` or `try_from`"));
        }
    }
    Ok(())
}

/// Asserts that no added variants are defined when mapping into another type (this is illegal for enums).
pub(super) fn validate_enum_add_into(add: &[AddedField]) -> Result<(), syn::Error> {
    if let Some(added) = add.first() {
        return Err(added.field.error("Illegal attribute for enums"));
    }
    Ok(())
}

/// Collects the identifiers of all added fields to be captured during deconstruction.
pub(super) fn make_captured_fields(add: &[AddedField]) -> CapturedFields {
    CapturedFields {
        fields: add.iter().map(|added| (*added.field).clone()).collect(),
    }
}

/// Collects all added variants to be captured during deconstruction.
pub(super) fn make_captured_variants(add: &[AddedField]) -> CapturedVariants {
    CapturedVariants {
        variants: add
            .iter()
            .map(|added| CapturedVariant {
                ident: (*added.field).clone(),
                default: added.default.as_ref().map(|expr| expr.value().clone()),
            })
            .collect(),
    }
}

/// Resolves the fallibility strategy details for a given conversion target.
pub(super) fn resolve_fallibility_mapping(opts: &ConversionOptions) -> FallibleMode {
    let base_error_ty: syn::Type = opts
        .err
        .clone()
        .map(syn::Type::Path)
        .unwrap_or_else(|| syn::parse_quote!(::anyhow::Error));

    let is_accumulate = opts.accumulate.is_some();

    let mode = if is_accumulate {
        let accumulator_ty = if let Some(acc_spanned) = &opts.accumulate
            && let Some(acc_path) = acc_spanned.as_ref()
        {
            syn::Type::Path(acc_path.clone())
        } else {
            syn::parse_quote!(::std::vec::Vec<#base_error_ty>)
        };
        FallibilityModeDetails::Accumulate { accumulator_ty }
    } else {
        FallibilityModeDetails::ShortCircuit
    };

    FallibleMode { base_error_ty, mode }
}

/// Performs an infallible lookup to resolve the field mapping attribute configuration for a specific derived target.
pub(super) fn resolve_field_attr(bf: &BaseField, derived_path: &syn::TypePath) -> Located<FieldMapping> {
    match &bf.mappings {
        FieldMappings::Default(attr) => *attr.clone(),
        FieldMappings::Overrides(overrides) => {
            if let Some((_path, attr)) = overrides.iter().find(|(path, _)| matches_path(path, derived_path)) {
                attr.clone()
            } else {
                let fallback_span = bf.ident.as_ref().map(syn::Ident::span).unwrap_or_else(|| bf.ty.span());
                Located::new(FieldMapping::default(), fallback_span)
            }
        }
    }
}

/// Performs an infallible lookup to resolve the variant mapping attribute configuration for a specific derived target.
pub(super) fn resolve_variant_attr(bv: &BaseVariant, derived_path: &syn::TypePath) -> VariantMapping {
    match &bv.mappings {
        VariantMappings::Default(attr) => *attr.clone(),
        VariantMappings::Overrides(overrides) => {
            if let Some((_path, attr)) = overrides.iter().find(|(path, _)| matches_path(path, derived_path)) {
                attr.clone()
            } else {
                VariantMapping::default()
            }
        }
    }
}

/// Parses the nested transform hint hierarchy for a field, validating direction constraints
/// and ensuring at most one hint is active at any nesting level.
pub(super) fn parse_transform(
    hint: &TransformHint,
    direction_is_from: bool,
    field_span: proc_macro2::Span,
) -> Result<ResolvedTransform, syn::Error> {
    let mut hint_count: usize = 0;
    let mut last_hint_span = field_span;

    if let Some(w) = &hint.with {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }
    if hint.from_with.is_some() || hint.into_with.is_some() {
        hint_count = hint_count.saturating_add(1);
        if let Some(with) = hint.from_with.as_ref().or(hint.into_with.as_ref()) {
            last_hint_span = with.span();
        }
    }
    if let Some(w) = &hint.opt {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }
    if let Some(w) = &hint.iter {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }
    if let Some(w) = &hint.map {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }
    if let Some(w) = &hint.boxed {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }
    if let Some(w) = &hint.r#box {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }
    if let Some(w) = &hint.unbox {
        hint_count = hint_count.saturating_add(1);
        last_hint_span = w.span();
    }

    if hint_count > 1 {
        return Err(syn::Error::new(
            last_hint_span,
            "Only one of 'with', 'into_with'/'from_with', 'opt', 'iter', 'map', 'boxed', 'box' or 'unbox' can be set",
        ));
    }

    if let Some(w) = &hint.with {
        Ok(ResolvedTransform::With(Located::new(w.value().clone(), w.span())))
    } else if direction_is_from {
        if let Some(w) = &hint.from_with {
            Ok(ResolvedTransform::With(Located::new(w.value().clone(), w.span())))
        } else if let Some(opt) = &hint.opt {
            let inner = parse_transform(opt.value(), direction_is_from, opt.span())?;
            Ok(ResolvedTransform::Option(Box::new(inner)))
        } else if let Some(iter) = &hint.iter {
            let inner = parse_transform(iter.value(), direction_is_from, iter.span())?;
            Ok(ResolvedTransform::Iterator(Box::new(inner)))
        } else if let Some(map) = &hint.map {
            let inner = parse_transform(map.value(), direction_is_from, map.span())?;
            Ok(ResolvedTransform::Map(Box::new(inner)))
        } else if let Some(boxed) = &hint.boxed {
            let inner = parse_transform(boxed.value(), direction_is_from, boxed.span())?;
            Ok(ResolvedTransform::Boxed(Box::new(inner)))
        } else if let Some(r#box) = &hint.r#box {
            let inner = parse_transform(r#box.value(), direction_is_from, r#box.span())?;
            // Derived (source) is boxed, Base (target) is not.
            Ok(ResolvedTransform::BoxSource(Box::new(inner)))
        } else if let Some(unbox) = &hint.unbox {
            let inner = parse_transform(unbox.value(), direction_is_from, unbox.span())?;
            // Derived (source) is not, Base (target) is boxed.
            Ok(ResolvedTransform::BoxTarget(Box::new(inner)))
        } else {
            Ok(ResolvedTransform::Default)
        }
    } else {
        if let Some(w) = &hint.into_with {
            Ok(ResolvedTransform::With(Located::new(w.value().clone(), w.span())))
        } else if let Some(opt) = &hint.opt {
            let inner = parse_transform(opt.value(), direction_is_from, opt.span())?;
            Ok(ResolvedTransform::Option(Box::new(inner)))
        } else if let Some(iter) = &hint.iter {
            let inner = parse_transform(iter.value(), direction_is_from, iter.span())?;
            Ok(ResolvedTransform::Iterator(Box::new(inner)))
        } else if let Some(map) = &hint.map {
            let inner = parse_transform(map.value(), direction_is_from, map.span())?;
            Ok(ResolvedTransform::Map(Box::new(inner)))
        } else if let Some(boxed) = &hint.boxed {
            let inner = parse_transform(boxed.value(), direction_is_from, boxed.span())?;
            Ok(ResolvedTransform::Boxed(Box::new(inner)))
        } else if let Some(r#box) = &hint.r#box {
            let inner = parse_transform(r#box.value(), direction_is_from, r#box.span())?;
            // Base (source) is not, Derived (target) is boxed.
            Ok(ResolvedTransform::BoxTarget(Box::new(inner)))
        } else if let Some(unbox) = &hint.unbox {
            let inner = parse_transform(unbox.value(), direction_is_from, unbox.span())?;
            // Base (source) is boxed, Derived (target) is not.
            Ok(ResolvedTransform::BoxSource(Box::new(inner)))
        } else {
            Ok(ResolvedTransform::Default)
        }
    }
}

/// Performs a single-pass validation across all overrides in the specification, checking
/// for duplicate overrides and ensuring that all override target types have a matching derive.
pub(super) fn validate_spec_overrides(
    spec: &MappingSpec,
    spec_derives: &HashSet<DerivedTypeKey>,
) -> Result<(), syn::Error> {
    match &spec.data {
        BaseData::Struct(fields) => {
            for bf in fields {
                validate_field_overrides(bf, spec, spec_derives)?;
            }
        }
        BaseData::Enum(variants) => {
            for bv in variants {
                validate_variant_overrides(bv, spec_derives)?;
                for bf in &bv.fields {
                    validate_field_overrides(bf, spec, spec_derives)?;
                }
            }
        }
    }
    Ok(())
}

/// Validates a specific field against a target derive configuration.
pub(super) fn validate_field_against_derive(bf: &BaseField, derive: &DeriveConfig) -> Result<(), syn::Error> {
    match &bf.mappings {
        FieldMappings::Default(attr) => {
            validate_field_mapping(attr, derive)?;
        }
        FieldMappings::Overrides(overrides) => {
            if let Some((_, attr)) = overrides.iter().find(|(path, _)| matches_path(path, &derive.path)) {
                validate_field_mapping(attr, derive)?;
            }
        }
    }
    Ok(())
}

fn validate_field_overrides(
    bf: &BaseField,
    spec: &MappingSpec,
    spec_derives: &HashSet<DerivedTypeKey>,
) -> Result<(), syn::Error> {
    match &bf.mappings {
        FieldMappings::Default(attr) => {
            for derive in &spec.derives {
                validate_field_mapping(attr, derive)?;
            }
        }
        FieldMappings::Overrides(overrides) => {
            let mut seen = HashSet::new();
            for (path, attr) in overrides {
                let key = DerivedTypeKey::new(path);
                if !seen.insert(key.clone()) {
                    return Err(syn::Error::new(
                        path.span(),
                        format!("This type is duplicated: '{key}'"),
                    ));
                }
                if !spec_derives.contains(&key) {
                    return Err(syn::Error::new(
                        path.span(),
                        format!("There is no derive defined for type: '{key}'"),
                    ));
                }
                if let Some(derive) = spec.derives.iter().find(|derive| matches_path(&derive.path, path)) {
                    validate_field_mapping(attr, derive)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_field_mapping(attr: &FieldMapping, derive: &DeriveConfig) -> Result<(), syn::Error> {
    if attr.err.is_some() && attr.err_with.is_some() {
        let span = attr
            .err
            .as_ref()
            .map(Spanned::span)
            .or_else(|| attr.err_with.as_ref().map(Spanned::span))
            .unwrap_or_else(|| derive.path.span());
        return Err(syn::Error::new(span, "Only one of 'err' or 'err_with' can be set"));
    }

    if (attr.err.is_some() || attr.err_with.is_some()) && derive.try_from.is_none() && derive.try_into.is_none() {
        let span = attr
            .err
            .as_ref()
            .or(attr.err_with.as_ref())
            .map(Spanned::span)
            .unwrap_or_else(|| derive.path.span());
        return Err(syn::Error::new(
            span,
            "'err' and 'err_with' are only valid when 'try_from' or 'try_into' is set",
        ));
    }

    if let Some(err) = &attr.err
        && let syn::Expr::Closure(_) = err
    {
        return Err(syn::Error::new(
            err.span(),
            "Use 'err_with' instead of 'err' for closure-based error mapping",
        ));
    }

    if let Some(into_with) = &attr.hint.into_with
        && derive.into.is_none()
        && derive.try_into.is_none()
    {
        return Err(into_with.error("'into_with' is not allowed on a FROM/TRY_FROM mapping direction"));
    }

    if let Some(from_with) = &attr.hint.from_with
        && derive.from.is_none()
        && derive.try_from.is_none()
    {
        return Err(from_with.error("'from_with' is not allowed on an INTO/TRY_INTO mapping direction"));
    }

    Ok(())
}

fn validate_variant_overrides(bv: &BaseVariant, spec_derives: &HashSet<DerivedTypeKey>) -> Result<(), syn::Error> {
    if let VariantMappings::Overrides(overrides) = &bv.mappings {
        let mut seen = HashSet::new();
        for (path, _) in overrides {
            let key = DerivedTypeKey::new(path);
            if !seen.insert(key.clone()) {
                return Err(syn::Error::new(
                    path.span(),
                    format!("This type is duplicated: '{key}'"),
                ));
            }
            if !spec_derives.contains(&key) {
                return Err(syn::Error::new(
                    path.span(),
                    format!("There is no derive defined for type: '{key}'"),
                ));
            }
        }
    }
    Ok(())
}
