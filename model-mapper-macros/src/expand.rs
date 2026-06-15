//! Code generation views and helper compilers for `domain` representation.

use core::ptr;
use std::collections::{HashMap, HashSet};

use heck::ToSnakeCase as _;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::{fold::Fold as _, parse_quote, visit::Visit as _};

use crate::{domain::*, type_path_ext::*};

/// Entry point for macro expansion: takes an [`ItemMapping`] AST node and generates token streams for all configured
/// conversion flows.
pub(crate) fn item_mapping(item: &ItemMapping) -> TokenStream {
    match expand_item_mapping(item) {
        Ok(tokens) => tokens,
        Err(err) => err.into_compile_error(),
    }
}

/// Iterates through all derivation flows defined on a struct or enum mapping and compiles them into a unified token
/// stream.
fn expand_item_mapping(item: &ItemMapping) -> syn::Result<TokenStream> {
    let mut output = TokenStream::new();

    match item {
        ItemMapping::Struct { spec, derives } => {
            for flow in derives {
                output.extend(flow.compile(spec));
            }
        }
        ItemMapping::Enum { spec, derives } => {
            for flow in derives {
                output.extend(flow.compile(spec)?);
            }
        }
    }
    Ok(output)
}

impl MappingFlow<StructShape> {
    /// Matches the concrete typestate matrix (`<F, D, I>`) for a struct mapping flow and delegates execution to
    /// [`compile_struct`].
    fn compile(&self, spec: &MappingContext) -> TokenStream {
        match self {
            Self::From(ImplStrategy::Trait(FallibilityStrategy::Infallible(mapping))) => compile_struct(spec, mapping),
            Self::From(ImplStrategy::Trait(FallibilityStrategy::Fallible(mapping))) => compile_struct(spec, mapping),
            Self::From(ImplStrategy::Custom(FallibilityStrategy::Infallible(mapping))) => compile_struct(spec, mapping),
            Self::From(ImplStrategy::Custom(FallibilityStrategy::Fallible(mapping))) => compile_struct(spec, mapping),
            Self::Into(ImplStrategy::Trait(FallibilityStrategy::Infallible(mapping))) => compile_struct(spec, mapping),
            Self::Into(ImplStrategy::Trait(FallibilityStrategy::Fallible(mapping))) => compile_struct(spec, mapping),
            Self::Into(ImplStrategy::Custom(FallibilityStrategy::Infallible(mapping))) => compile_struct(spec, mapping),
            Self::Into(ImplStrategy::Custom(FallibilityStrategy::Fallible(mapping))) => compile_struct(spec, mapping),
        }
    }
}

impl MappingFlow<EnumShape> {
    /// Matches the concrete typestate matrix (`<F, D, I>`) for an enum mapping flow and delegates execution to
    /// [`compile_enum`].
    fn compile(&self, spec: &MappingContext) -> syn::Result<TokenStream> {
        match self {
            Self::From(ImplStrategy::Trait(FallibilityStrategy::Infallible(mapping))) => compile_enum(spec, mapping),
            Self::From(ImplStrategy::Trait(FallibilityStrategy::Fallible(mapping))) => compile_enum(spec, mapping),
            Self::From(ImplStrategy::Custom(FallibilityStrategy::Infallible(mapping))) => compile_enum(spec, mapping),
            Self::From(ImplStrategy::Custom(FallibilityStrategy::Fallible(mapping))) => compile_enum(spec, mapping),
            Self::Into(ImplStrategy::Trait(FallibilityStrategy::Infallible(mapping))) => compile_enum(spec, mapping),
            Self::Into(ImplStrategy::Trait(FallibilityStrategy::Fallible(mapping))) => compile_enum(spec, mapping),
            Self::Into(ImplStrategy::Custom(FallibilityStrategy::Infallible(mapping))) => compile_enum(spec, mapping),
            Self::Into(ImplStrategy::Custom(FallibilityStrategy::Fallible(mapping))) => compile_enum(spec, mapping),
        }
    }
}

/// Compiles a struct mapping generic configuration (`<F, D, I>`) into a complete Rust implementation, deconstructing
/// the source struct and constructing the target.
pub(crate) fn compile_struct<F, D, I>(spec: &MappingContext, mapping: &StructMapping<F, D, I>) -> TokenStream
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplModeExpand,
    D::Skip: SkipExpand,
    D::StructExtra: ExtraFieldsExpand,
{
    let fallibility = mapping.fallibility();
    let impl_mode = mapping.impl_mode();

    // 1. Resolve and merge generic parameters, adding type conversion bounds for fields.
    let (mut all_generics, derived_ty_with_generics, generics_rename_map) =
        D::process_generics(spec.base_generics(), spec.derived_path());
    D::add_generics_bounds(&mut all_generics, mapping.fields(), &generics_rename_map);

    let (impl_generics, _, where_clause) = all_generics.split_for_impl();
    let derived_ty_stripped = strip_generics(&derived_ty_with_generics);
    let (_, local_ty_generics, _) = spec.base_generics().split_for_impl();

    // 2. Build the pattern matching expression to deconstruct fields from the input.
    let deconstructed_from = DeconstructionBuilder::<F, D, I>::new(mapping.fields())
        .with_extra(mapping.extra())
        .build_struct();

    // 3. Collect external function parameters and input struct declarations (for custom fn mode).
    let (external_params, inst) = D::struct_custom_params(spec.base_ident(), mapping.fields(), mapping.extra());

    // 4. Build the target struct instantiation expression.
    let construction_expr = StructConstructionBuilder::new(mapping.fields())
        .with_extra(mapping.extra())
        .build(&derived_ty_stripped);

    // 5. Generate local default variable declarations for added and skipped fields.
    let default_decls = build_struct_default_decls(mapping.fields(), mapping.extra(), impl_mode);

    // 6. Generate the conversion body (wrapping in Result/Ok if fallible).
    let body_expr = fallibility.compile_body(mapping.fields(), mapping.extra(), construction_expr);

    // 7. Combine input pattern deconstruction, default declarations, and body execution.
    let deconstruct_target = impl_mode.deconstruct_target::<D>();
    let input_type_prefix = D::input_type_prefix(&derived_ty_stripped, spec.base_ident());

    let function_body = quote!(
        let #input_type_prefix #deconstructed_from = #deconstruct_target;
        #default_decls
        #body_expr
    );

    // 8. Wrap the compiled body in a trait impl or custom inherent fn block.
    impl_mode.generate_wrapper::<F, D>(CodeGenWrapperArgs {
        spec,
        fallibility,
        impl_generics: &impl_generics,
        where_clause,
        derived_ty: &derived_ty_with_generics,
        local_generics: &local_ty_generics,
        body: function_body,
        external_params: &external_params,
        inst,
    })
}

/// Compiles an enum mapping generic configuration (`<F, D, I>`) into a complete Rust match expression implementation,
/// handling variant pattern matching and fallbacks.
pub(crate) fn compile_enum<F, D, I>(spec: &MappingContext, mapping: &EnumMapping<F, D, I>) -> syn::Result<TokenStream>
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplModeExpand,
    D::Skip: SkipExpand,
    D::VariantExtraFields: ExtraFieldsExpand,
    D::EnumExtra: ExtraFieldsExpand,
{
    let fallibility = mapping.fallibility();
    let impl_mode = mapping.impl_mode();

    // 1. Resolve and merge generic parameters, adding type conversion bounds across variant fields.
    let (mut all_generics, derived_ty_with_generics, generics_rename_map) =
        D::process_generics(spec.base_generics(), spec.derived_path());
    for variant in mapping.variants() {
        D::add_generics_bounds(&mut all_generics, variant.fields(), &generics_rename_map);
    }

    let (impl_generics, _, where_clause) = all_generics.split_for_impl();
    let derived_ty_stripped = strip_generics(&derived_ty_with_generics);
    let (_, local_ty_generics, _) = spec.base_generics().split_for_impl();

    let mut match_arms = Vec::new();

    // 2. Build pattern matching arms and RHS construction for each active variant.
    for variant in mapping.variants() {
        if variant.skip().is_none() {
            let derived_var_ident = variant.derived_ident();

            let pat = DeconstructionBuilder::<F, D, I>::new(variant.fields())
                .with_extra(variant.extra())
                .build_variant();

            let lhs = D::enum_lhs(
                &derived_ty_stripped,
                spec.base_ident(),
                variant.ident(),
                derived_var_ident,
                pat,
            );

            let constr = VariantConstructionBuilder::new(variant.fields())
                .with_extra(variant.extra())
                .build(&derived_ty_stripped);

            let rhs = D::enum_rhs(
                &derived_ty_stripped,
                spec.base_ident(),
                variant.ident(),
                derived_var_ident,
                constr,
            );

            let rhs_final = fallibility.compile_body(variant.fields(), variant.extra(), rhs);

            let variant_default_decls = build_variant_default_decls(variant, impl_mode);

            let rhs_final = quote!({
                #variant_default_decls
                #rhs_final
            });

            match_arms.push(quote!(#lhs => #rhs_final,));
        }
    }

    // 3. Append fallback match arms for captured or ignored extra variants.
    D::compile_enum_fallbacks(
        &mut match_arms,
        spec,
        mapping.extra(),
        &derived_ty_stripped,
        fallibility,
        mapping.variants(),
    )?;

    // 4. Assemble all match arms into a single match expression on the input.
    let deconstruct_target = impl_mode.deconstruct_target::<D>();
    let match_body = quote!(
        match #deconstruct_target {
            #( #match_arms )*
        }
    );

    // 5. Collect external provider closure parameters for custom function mappings.
    let external_params = D::enum_custom_params(mapping.variants());
    let inst = TokenStream::new();

    // 6. Wrap the compiled match expression in a trait impl or custom inherent fn block.
    let wrapper = impl_mode.generate_wrapper::<F, D>(CodeGenWrapperArgs {
        spec,
        fallibility,
        impl_generics: &impl_generics,
        where_clause,
        derived_ty: &derived_ty_with_generics,
        local_generics: &local_ty_generics,
        body: match_body,
        external_params: &external_params,
        inst,
    });

    Ok(wrapper)
}

// =========================================================================
// Plain Leaf Emit Functions
// =========================================================================

/// Arguments passed into [`FallibilityExpand::emit_trait`].
pub(crate) struct TraitEmitArgs<'a> {
    pub impl_generics: &'a syn::ImplGenerics<'a>,
    pub where_clause: Option<&'a syn::WhereClause>,
    pub input_ty: &'a TokenStream,
    pub output_ty: &'a TokenStream,
    pub body: TokenStream,
}

/// Arguments passed into [`FallibilityExpand::emit_custom`].
pub(crate) struct CustomEmitArgs<'a> {
    pub impl_generics: &'a syn::ImplGenerics<'a>,
    pub local_generics: &'a syn::TypeGenerics<'a>,
    pub where_clause: Option<&'a syn::WhereClause>,
    pub base_ident: &'a syn::Ident,
    pub fn_name: &'a syn::Ident,
    pub doc: &'a str,
    pub fn_params: &'a TokenStream,
    pub ret_ty: &'a TokenStream,
    pub inst: &'a TokenStream,
    pub body: TokenStream,
}

/// Emits an infallible `From<Input> for Output` trait implementation.
fn emit_trait_impl(
    impl_generics: &syn::ImplGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    input_ty: &TokenStream,
    output_ty: &TokenStream,
    body: TokenStream,
) -> TokenStream {
    quote!(
        #[automatically_derived]
        #[allow(non_shorthand_field_patterns, unused_variables)]
        impl #impl_generics From<#input_ty> for #output_ty #where_clause {
            fn from(other: #input_ty) -> Self {
                #body
            }
        }
    )
}

/// Emits a fallible `TryFrom<Input> for Output` trait implementation.
fn emit_try_trait_impl(
    impl_generics: &syn::ImplGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    input_ty: &TokenStream,
    output_ty: &TokenStream,
    error_ty: &TokenStream,
    body: TokenStream,
) -> TokenStream {
    quote!(
        #[automatically_derived]
        #[allow(non_shorthand_field_patterns, unused_variables)]
        impl #impl_generics TryFrom<#input_ty> for #output_ty #where_clause {
            type Error = #error_ty;
            fn try_from(other: #input_ty) -> ::core::result::Result<Self, Self::Error> {
                #body
            }
        }
    )
}

/// Emits a custom function implementation (infallible or fallible).
/// Note: `ret_ty` is pre-wrapped in `Result<T, E>` by `FallibleMode` prior to invoking this function.
fn emit_custom_fn(
    impl_generics: &syn::ImplGenerics<'_>,
    local_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    base_ident: &syn::Ident,
    fn_name: &syn::Ident,
    doc: &str,
    fn_params: &TokenStream,
    ret_ty: &TokenStream,
    inst: &TokenStream,
    body: TokenStream,
) -> TokenStream {
    quote!(
        #[automatically_derived]
        #[allow(non_shorthand_field_patterns, unused_variables)]
        impl #impl_generics #base_ident #local_generics #where_clause {
            #[doc = #doc]
            #[allow(clippy::too_many_arguments)]
            pub fn #fn_name(#fn_params) -> #ret_ty {
                #inst
                #body
            }
        }
    )
}

// =========================================================================
// Typestate Polymorphism Traits & Implementations
// =========================================================================

/// Direction mode expansion trait representing conversion directions.
pub(crate) trait DirectionExpand<I: ImplMode>: DirectionMode<I> {
    /// Merges base type generics with derived type parameters and builds generic rename mappings to prevent name
    /// collisions.
    fn process_generics(
        base_generics: &syn::Generics,
        derived_path: &syn::TypePath,
    ) -> (syn::Generics, syn::TypePath, HashMap<syn::Ident, syn::Ident>);

    /// Appends field-level `where` bound constraints for fields with explicit type mappings (`other_ty`).
    fn add_generics_bounds<F: FallibilityMode>(
        all_generics: &mut syn::Generics,
        fields: &[MappedField<F, Self, I>],
        generics_rename_map: &HashMap<syn::Ident, syn::Ident>,
    ) where
        Self: Sized;

    /// Resolves the input type for trait mappings (e.g. `DerivedType` for `From`, `BaseType` for `Into`).
    fn input_type(spec: &MappingContext, derived_ty: &syn::TypePath, local_generics: &syn::TypeGenerics)
    -> TokenStream;

    /// Resolves the target return type for trait mappings (e.g. `BaseType` for `From`, `DerivedType` for `Into`).
    fn output_type(
        spec: &MappingContext,
        derived_ty: &syn::TypePath,
        local_generics: &syn::TypeGenerics,
    ) -> TokenStream;

    /// Returns the variable expression name being deconstructed (e.g. `from` for `From`, `self` for `Into`).
    fn deconstruct_target() -> TokenStream;

    /// Generates the struct/type path prefix used in `let` pattern matching deconstruction.
    fn input_type_prefix(derived_type_stripped: &syn::TypePath, base_ident: &syn::Ident) -> TokenStream;

    /// Generates the parameter list tokens for custom helper functions.
    fn custom_fn_params(derived_ty: &syn::TypePath, external_params: &[TokenStream]) -> TokenStream;

    /// Resolves the un-wrapped return type signature for custom helper functions.
    fn custom_fn_ret_type(spec: &MappingContext, derived_ty: &syn::TypePath) -> TokenStream;

    /// Generates the doc comment string describing the generated custom mapping function.
    fn custom_fn_doc<F: FallibilityExpand>(
        fallibility: &F,
        spec: &MappingContext,
        derived_ty: &syn::TypePath,
    ) -> String;

    /// Gathers external parameters and custom input struct instantiations required for struct conversions.
    fn struct_custom_params<F: FallibilityMode>(
        base_ident: &syn::Ident,
        fields: &[MappedField<F, Self, I>],
        extra: &Self::StructExtra,
    ) -> (Vec<TokenStream>, TokenStream)
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::StructExtra: ExtraFieldsExpand;

    /// Gathers external provider closure parameters required for custom enum conversions.
    fn enum_custom_params<F: FallibilityMode>(variants: &[MappedVariant<F, Self, I>]) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::VariantExtraFields: ExtraFieldsExpand;

    /// Generates the left-hand side pattern matching arm for an enum variant.
    fn enum_lhs(
        derived_type_stripped: &syn::TypePath,
        base_ident: &syn::Ident,
        var_ident: &syn::Ident,
        derived_var_ident: &syn::Ident,
        pat: TokenStream,
    ) -> TokenStream;

    /// Generates the right-hand side construction expression arm for an enum variant.
    fn enum_rhs(
        derived_type_stripped: &syn::TypePath,
        base_ident: &syn::Ident,
        var_ident: &syn::Ident,
        derived_var_ident: &syn::Ident,
        constr: TokenStream,
    ) -> TokenStream;

    /// Compiles fallback match arms for captured or ignored extra enum variants.
    fn compile_enum_fallbacks<F: FallibilityExpand>(
        match_arms: &mut Vec<TokenStream>,
        spec: &MappingContext,
        extra: &Self::EnumExtra,
        derived_type_stripped: &syn::TypePath,
        fallibility: &F,
        variants: &[MappedVariant<F, Self, I>],
    ) -> syn::Result<()>
    where
        Self: Sized,
        Self::EnumExtra: ExtraFieldsExpand;

    /// Returns the default function name suffix string (e.g. `"from"` or `"into"`).
    fn default_fn_suffix() -> &'static str;

    /// Determines the local variable binding name used on the destination side during field deconstruction.
    fn dest_ident(src_ident: &syn::Ident, pat_ident: &syn::Ident) -> syn::Ident;

    /// Determines the local variable binding name used on the source side during field deconstruction.
    fn bound_ident(src_ident: &syn::Ident, pat_ident: &syn::Ident) -> syn::Ident;

    /// Generates the field pattern tokens used when deconstructing named struct/variant fields.
    fn field_deconstruct_pattern(f_ident: &syn::Ident, pat_ident: &syn::Ident, has_skip: bool) -> Option<TokenStream>;

    /// Generates the field pattern tokens used when deconstructing unnamed tuple fields.
    fn tuple_field_deconstruct_pattern(ix: usize, has_skip: bool) -> TokenStream;

    /// Generates individual field initialization expressions for struct construction.
    fn struct_construction_parts<F: FallibilityExpand>(
        fields: &[MappedField<F, Self, I>],
        extra: Option<&Self::StructExtra>,
        derived_type_stripped: &syn::TypePath,
    ) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::StructExtra: ExtraFieldsExpand;

    /// Generates individual field initialization expressions for enum variant construction.
    fn variant_construction_parts<F: FallibilityExpand>(
        fields: &[MappedField<F, Self, I>],
        extra: Option<&Self::VariantExtraFields>,
        derived_type_stripped: &syn::TypePath,
    ) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::VariantExtraFields: ExtraFieldsExpand;

    /// Returns the type path prefix used when constructing a target struct.
    fn struct_construction_prefix(derived_type_stripped: &syn::TypePath) -> TokenStream;

    /// Determines whether struct construction should append `..Default::default()` to ignore extra fields.
    fn struct_construction_ignore_extra(extra: Option<&Self::StructExtra>) -> bool;

    /// Determines whether variant construction should ignore extra unmapped fields.
    fn variant_construction_ignore_extra(extra: Option<&Self::VariantExtraFields>) -> bool;

    /// Generates field binding tokens for skipped fields in custom function mode.
    fn custom_skipped_field_binding(var_ident: &syn::Ident, ty: &syn::Type) -> Option<TokenStream>;
}

impl<I: ImplMode> DirectionExpand<I> for FromDirection {
    fn process_generics(
        base_generics: &syn::Generics,
        derived_path: &syn::TypePath,
    ) -> (syn::Generics, syn::TypePath, HashMap<syn::Ident, syn::Ident>) {
        process_generics(
            base_generics,
            derived_path,
            |ident, new_ident| parse_quote!(#new_ident: Into<#ident>),
        )
    }

    fn add_generics_bounds<F: FallibilityMode>(
        all_generics: &mut syn::Generics,
        fields: &[MappedField<F, Self, I>],
        generics_rename_map: &HashMap<syn::Ident, syn::Ident>,
    ) where
        Self: Sized,
    {
        add_generics_bounds(
            all_generics,
            fields,
            generics_rename_map,
            |resolved_ident, field_ty| parse_quote!(#resolved_ident: Into<#field_ty>),
        );
    }

    fn input_type(
        _spec: &MappingContext,
        derived_ty: &syn::TypePath,
        _local_generics: &syn::TypeGenerics,
    ) -> TokenStream {
        quote!(#derived_ty)
    }

    fn output_type(
        spec: &MappingContext,
        _derived_ty: &syn::TypePath,
        local_generics: &syn::TypeGenerics,
    ) -> TokenStream {
        let base_ident = spec.base_ident();
        quote!(#base_ident #local_generics)
    }

    fn deconstruct_target() -> TokenStream {
        quote!(from)
    }

    fn input_type_prefix(derived_type_stripped: &syn::TypePath, _base_ident: &syn::Ident) -> TokenStream {
        quote!(#derived_type_stripped)
    }

    fn custom_fn_params(derived_ty: &syn::TypePath, external_params: &[TokenStream]) -> TokenStream {
        quote!(from: #derived_ty, #( #external_params ),*)
    }

    fn custom_fn_ret_type(_spec: &MappingContext, _derived_ty: &syn::TypePath) -> TokenStream {
        quote!(Self)
    }

    fn custom_fn_doc<F: FallibilityExpand>(
        fallibility: &F,
        spec: &MappingContext,
        derived_ty: &syn::TypePath,
    ) -> String {
        let prefix = fallibility.doc_prefix();
        let target_str = derived_ty.to_token_stream().to_string().replace(' ', "");
        let base_ident = spec.base_ident();
        format!("{prefix} a new [{base_ident}] from a [{target_str}]")
    }

    fn struct_custom_params<F: FallibilityMode>(
        base_ident: &syn::Ident,
        fields: &[MappedField<F, Self, I>],
        _extra: &Self::StructExtra,
    ) -> (Vec<TokenStream>, TokenStream)
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::StructExtra: ExtraFieldsExpand,
    {
        let (params, param_info) = get_struct_from_custom_external_params(fields);
        let inst = get_struct_custom_param_instantiation(base_ident, &param_info);
        (params, inst)
    }

    fn enum_custom_params<F: FallibilityMode>(variants: &[MappedVariant<F, Self, I>]) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::VariantExtraFields: ExtraFieldsExpand,
    {
        get_enum_from_custom_external_params(variants)
    }

    fn enum_lhs(
        derived_type_stripped: &syn::TypePath,
        _base_ident: &syn::Ident,
        _var_ident: &syn::Ident,
        derived_var_ident: &syn::Ident,
        pat: TokenStream,
    ) -> TokenStream {
        quote!(#derived_type_stripped::#derived_var_ident #pat)
    }

    fn enum_rhs(
        _derived_type_stripped: &syn::TypePath,
        base_ident: &syn::Ident,
        var_ident: &syn::Ident,
        _derived_var_ident: &syn::Ident,
        constr: TokenStream,
    ) -> TokenStream {
        quote!(#base_ident::#var_ident #constr)
    }

    fn compile_enum_fallbacks<F: FallibilityExpand>(
        match_arms: &mut Vec<TokenStream>,
        _spec: &MappingContext,
        extra: &Self::EnumExtra,
        derived_type_stripped: &syn::TypePath,
        fallibility: &F,
        _variants: &[MappedVariant<F, Self, I>],
    ) -> syn::Result<()>
    where
        Self: Sized,
        Self::EnumExtra: ExtraFieldsExpand,
    {
        // 1. Generate match arms for captured unmapped variants.
        for variant in extra.captured_variants().variants() {
            let val = variant
                .default_expr()
                .cloned()
                .unwrap_or_else(|| syn::parse_quote!(Default::default()));
            let rhs = fallibility.wrap_expr_ok(val);
            let variant_ident = variant.ident();
            match_arms.push(quote!(#derived_type_stripped::#variant_ident { .. } => #rhs,));
        }

        // 2. Generate catch-all wildcard (_) match arm when extra variants are ignored.
        if extra.ignore_extra() {
            let rhs = fallibility.wrap_expr_ok(syn::parse_quote!(Default::default()));
            match_arms.push(quote!(_ => #rhs,));
        }
        Ok(())
    }

    fn default_fn_suffix() -> &'static str {
        "from"
    }

    fn dest_ident(src_ident: &syn::Ident, _pat_ident: &syn::Ident) -> syn::Ident {
        src_ident.clone()
    }

    fn bound_ident(_src_ident: &syn::Ident, pat_ident: &syn::Ident) -> syn::Ident {
        pat_ident.clone()
    }

    fn field_deconstruct_pattern(_f_ident: &syn::Ident, pat_ident: &syn::Ident, has_skip: bool) -> Option<TokenStream> {
        if has_skip { None } else { Some(quote!(#pat_ident)) }
    }

    fn tuple_field_deconstruct_pattern(ix: usize, has_skip: bool) -> TokenStream {
        if has_skip {
            quote!(_)
        } else {
            let var_ident = format_ident!("_{}", ix);
            quote!(#var_ident)
        }
    }

    fn struct_construction_parts<F: FallibilityExpand>(
        fields: &[MappedField<F, Self, I>],
        _extra: Option<&Self::StructExtra>,
        _derived_type_stripped: &syn::TypePath,
    ) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::StructExtra: ExtraFieldsExpand,
    {
        let mut parts = Vec::new();
        for field in with_fields_first(fields) {
            let Some(f_ident) = field.base_ident().cloned() else {
                continue;
            };

            let pat_ident = field.pat_ident().unwrap_or_else(|| f_ident.clone());

            if field.skip().is_some() {
                parts.push(quote!(#f_ident: #pat_ident));
            } else {
                let expr = F::field_construction_expr(&f_ident, &pat_ident, field.transform());
                parts.push(expr);
            }
        }
        parts
    }

    fn variant_construction_parts<F: FallibilityExpand>(
        fields: &[MappedField<F, Self, I>],
        _extra: Option<&Self::VariantExtraFields>,
        _derived_type_stripped: &syn::TypePath,
    ) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::VariantExtraFields: ExtraFieldsExpand,
    {
        let mut parts = Vec::new();
        for field in with_fields_first(fields) {
            let Some(field_name) = field.base_ident().cloned() else {
                continue;
            };

            let pat_name = field.pat_ident().unwrap_or_else(|| field_name.clone());

            if field.skip().is_some() {
                parts.push(quote!(#field_name: #pat_name));
            } else {
                let expr = F::field_construction_expr(&field_name, &pat_name, field.transform());
                parts.push(expr);
            }
        }
        parts
    }

    fn struct_construction_prefix(_derived_type_stripped: &syn::TypePath) -> TokenStream {
        quote!(Self)
    }

    fn struct_construction_ignore_extra(_extra: Option<&Self::StructExtra>) -> bool {
        false
    }

    fn variant_construction_ignore_extra(_extra: Option<&Self::VariantExtraFields>) -> bool {
        false
    }

    fn custom_skipped_field_binding(var_ident: &syn::Ident, ty: &syn::Type) -> Option<TokenStream> {
        Some(quote!(let #var_ident: #ty = input.#var_ident;))
    }
}

impl<I: ImplMode + ImplModeExpand> DirectionExpand<I> for IntoDirection
where
    I::StructIntoAddedFields: ExtraFieldsExpand,
    I::VariantIntoAddedFields: ExtraFieldsExpand,
{
    fn process_generics(
        base_generics: &syn::Generics,
        derived_path: &syn::TypePath,
    ) -> (syn::Generics, syn::TypePath, HashMap<syn::Ident, syn::Ident>) {
        process_generics(
            base_generics,
            derived_path,
            |ident, new_ident| parse_quote!(#ident: Into<#new_ident>),
        )
    }

    fn add_generics_bounds<F: FallibilityMode>(
        all_generics: &mut syn::Generics,
        fields: &[MappedField<F, Self, I>],
        generics_rename_map: &HashMap<syn::Ident, syn::Ident>,
    ) where
        Self: Sized,
    {
        add_generics_bounds(
            all_generics,
            fields,
            generics_rename_map,
            |resolved_ident, field_ty| parse_quote!(#field_ty: Into<#resolved_ident>),
        );
    }

    fn input_type(
        spec: &MappingContext,
        _derived_ty: &syn::TypePath,
        local_generics: &syn::TypeGenerics,
    ) -> TokenStream {
        let base_ident = spec.base_ident();
        quote!(#base_ident #local_generics)
    }

    fn output_type(
        _spec: &MappingContext,
        derived_ty: &syn::TypePath,
        _local_generics: &syn::TypeGenerics,
    ) -> TokenStream {
        quote!(#derived_ty)
    }

    fn deconstruct_target() -> TokenStream {
        quote!(self)
    }

    fn input_type_prefix(_derived_type_stripped: &syn::TypePath, base_ident: &syn::Ident) -> TokenStream {
        quote!(#base_ident)
    }

    fn custom_fn_params(_derived_ty: &syn::TypePath, external_params: &[TokenStream]) -> TokenStream {
        quote!(self, #( #external_params ),*)
    }

    fn custom_fn_ret_type(_spec: &MappingContext, derived_ty: &syn::TypePath) -> TokenStream {
        quote!(#derived_ty)
    }

    fn custom_fn_doc<F: FallibilityExpand>(
        fallibility: &F,
        spec: &MappingContext,
        derived_ty: &syn::TypePath,
    ) -> String {
        let prefix = fallibility.doc_prefix();
        let target_str = derived_ty.to_token_stream().to_string().replace(' ', "");
        let base_ident = spec.base_ident();
        format!("{prefix} a new [{target_str}] from this [{base_ident}]")
    }

    fn struct_custom_params<F: FallibilityMode>(
        base_ident: &syn::Ident,
        _fields: &[MappedField<F, Self, I>],
        extra: &Self::StructExtra,
    ) -> (Vec<TokenStream>, TokenStream)
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::StructExtra: ExtraFieldsExpand,
    {
        let mut params = Vec::new();
        let mut param_info = Vec::new();
        extra.for_each_added_field(|name, default_expr, ty| {
            if default_expr.is_none() {
                let actual_ty = ty.cloned().unwrap_or_else(|| syn::parse_quote!(String));
                param_info.push((name.clone(), actual_ty.clone()));
                params.push(quote!(#name: #actual_ty));
            }
        });
        let inst = get_struct_custom_param_instantiation(base_ident, &param_info);
        (params, inst)
    }

    fn enum_custom_params<F: FallibilityMode>(variants: &[MappedVariant<F, Self, I>]) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::VariantExtraFields: ExtraFieldsExpand,
    {
        get_enum_into_custom_external_params(variants)
    }

    fn enum_lhs(
        _derived_type_stripped: &syn::TypePath,
        base_ident: &syn::Ident,
        var_ident: &syn::Ident,
        _derived_var_ident: &syn::Ident,
        pat: TokenStream,
    ) -> TokenStream {
        quote!(#base_ident::#var_ident #pat)
    }

    fn enum_rhs(
        derived_type_stripped: &syn::TypePath,
        _base_ident: &syn::Ident,
        _var_ident: &syn::Ident,
        derived_var_ident: &syn::Ident,
        constr: TokenStream,
    ) -> TokenStream {
        quote!(#derived_type_stripped::#derived_var_ident #constr)
    }

    fn compile_enum_fallbacks<F: FallibilityExpand>(
        match_arms: &mut Vec<TokenStream>,
        spec: &MappingContext,
        _extra: &Self::EnumExtra,
        _derived_type_stripped: &syn::TypePath,
        fallibility: &F,
        variants: &[MappedVariant<F, Self, I>],
    ) -> syn::Result<()>
    where
        Self: Sized,
        Self::EnumExtra: ExtraFieldsExpand,
    {
        let from_ty = spec.base_ident();
        for variant in variants {
            if let Some(skip) = variant.skip() {
                let val = <Self::Skip as SkipExpand>::default_expr(skip.value())
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(Default::default()));
                let rhs = fallibility.wrap_expr_ok(val);
                let var_ident = variant.ident();
                match_arms.push(quote!(#from_ty::#var_ident { .. } => #rhs,));
            }
        }
        Ok(())
    }

    fn default_fn_suffix() -> &'static str {
        "into"
    }

    fn dest_ident(_src_ident: &syn::Ident, pat_ident: &syn::Ident) -> syn::Ident {
        pat_ident.clone()
    }

    fn bound_ident(src_ident: &syn::Ident, _pat_ident: &syn::Ident) -> syn::Ident {
        src_ident.clone()
    }

    fn field_deconstruct_pattern(f_ident: &syn::Ident, pat_ident: &syn::Ident, has_skip: bool) -> Option<TokenStream> {
        if has_skip {
            Some(quote!(#f_ident: #pat_ident))
        } else {
            Some(quote!(#f_ident))
        }
    }

    fn tuple_field_deconstruct_pattern(ix: usize, _has_skip: bool) -> TokenStream {
        let var_ident = format_ident!("_{}", ix);
        quote!(#var_ident)
    }

    fn struct_construction_parts<F: FallibilityExpand>(
        fields: &[MappedField<F, Self, I>],
        extra: Option<&Self::StructExtra>,
        _derived_type_stripped: &syn::TypePath,
    ) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::StructExtra: ExtraFieldsExpand,
    {
        let mut parts = Vec::new();

        // Build initialization expressions for added fields:
        if let Some(extra_fields) = extra {
            extra_fields.for_each_added_field(|field_name, default_expr, _ty| {
                if default_expr.is_some() {
                    parts.push(quote!(#field_name: #field_name));
                } else {
                    parts.push(quote!(#field_name: input.#field_name));
                }
            });
        }

        // Build initialization expressions for mapped fields:
        for field in with_fields_first(fields) {
            if field.skip().is_some() {
                continue;
            }

            let Some(f_ident) = field.base_ident().cloned() else {
                continue;
            };

            let pat_ident = field.pat_ident().unwrap_or_else(|| f_ident.clone());

            let expr = F::field_construction_expr(&pat_ident, &f_ident, field.transform());
            parts.push(expr);
        }
        parts
    }

    fn variant_construction_parts<F: FallibilityExpand>(
        fields: &[MappedField<F, Self, I>],
        extra: Option<&Self::VariantExtraFields>,
        _derived_type_stripped: &syn::TypePath,
    ) -> Vec<TokenStream>
    where
        Self: Sized,
        Self::Skip: SkipExpand,
        Self::VariantExtraFields: ExtraFieldsExpand,
    {
        let mut parts = Vec::new();

        // Build initialization expressions for added fields:
        if let Some(extra_fields) = extra {
            extra_fields.for_each_added_field(|field_name, _default_expr, _ty| {
                parts.push(quote!(#field_name: #field_name));
            });
        }

        // Build initialization expressions for mapped fields:
        for field in with_fields_first(fields) {
            if field.skip().is_some() {
                continue;
            }

            let Some(field_name) = field.base_ident().cloned() else {
                continue;
            };

            let pat_name = field.pat_ident().unwrap_or_else(|| field_name.clone());

            let expr = F::field_construction_expr(&pat_name, &field_name, field.transform());
            parts.push(expr);
        }
        parts
    }

    fn struct_construction_prefix(derived_type_stripped: &syn::TypePath) -> TokenStream {
        quote!(#derived_type_stripped)
    }

    fn struct_construction_ignore_extra(extra: Option<&Self::StructExtra>) -> bool {
        extra.map(ExtraFieldsExpand::ignore_extra).unwrap_or(false)
    }

    fn variant_construction_ignore_extra(_extra: Option<&Self::VariantExtraFields>) -> bool {
        false
    }

    fn custom_skipped_field_binding(_var_ident: &syn::Ident, _ty: &syn::Type) -> Option<TokenStream> {
        None
    }
}

// =========================================================================
// Typestate Fallibility Codegen Specialization Traits
// =========================================================================

/// Fallibility mode expansion trait representing conversion fallibilities.
pub(crate) trait FallibilityExpand: FallibilityMode + Sized {
    /// Retrieves the optional error handler strategy configured for fallible conversions.
    fn get_error_handler(_handler: &Self::ErrorHandler) -> Option<&ResolvedErrorHandler> {
        None
    }

    /// Wraps a return type in `Result<T, E>` if the conversion is fallible, or returns `T` directly if infallible.
    fn wrap_return_type(&self, ty: TokenStream) -> TokenStream;

    /// Resolves the associated error type for `TryFrom` trait implementations.
    fn error_type(&self) -> TokenStream;

    /// Returns the descriptive documentation prefix verb (`"Builds"` or `"Tries to build"`).
    fn doc_prefix(&self) -> &'static str;

    /// Wraps an expression in `Ok(...)` if the conversion is fallible, or leaves it unwrapped if infallible.
    fn wrap_expr_ok(&self, expr: syn::Expr) -> TokenStream;

    /// Resolves or synthesizes the function name for custom mapping functions based on fallibility.
    fn fn_name<D: DirectionExpand<I>, I: ImplModeExpand>(&self, impl_mode: &I, spec: &MappingContext) -> syn::Ident;

    /// Compiles the inner function body block, generating short-circuiting (`?`) or error-accumulating execution logic.
    fn compile_body<D: DirectionExpand<I>, I: ImplMode, E: ExtraFieldsExpand>(
        &self,
        fields: &[MappedField<Self, D, I>],
        extra: &E,
        construction_expr: TokenStream,
    ) -> TokenStream
    where
        D::Skip: SkipExpand;

    /// Generates the field conversion expression (e.g. `Into::into`, `TryInto::try_into`, or custom closure/mapping).
    fn transform_field_expr(transform: &ResolvedTransform, ident: &syn::Ident) -> TokenStream;

    /// Generates the `field_name: expr` pair for named field construction based on fallibility.
    fn field_construction_expr(
        target_ident: &syn::Ident,
        source_ident: &syn::Ident,
        transform: &ResolvedTransform,
    ) -> TokenStream;

    /// Generates the conversion expression for an unnamed tuple field.
    fn tuple_construction_part(var_ident: &syn::Ident, transform: &ResolvedTransform) -> TokenStream;

    /// Generates transformation logic for heap-allocated `Box<T>` fields.
    fn build_box_expr(
        inner: &ResolvedTransform,
        ident: &syn::Ident,
        unbox_input: bool,
        box_output: bool,
    ) -> TokenStream;

    /// Dispatches code generation for `From` or `TryFrom` trait implementation blocks.
    fn emit_trait(&self, args: TraitEmitArgs<'_>) -> TokenStream;

    /// Dispatches code generation for custom helper function implementation blocks.
    fn emit_custom(&self, args: CustomEmitArgs<'_>) -> TokenStream;
}

impl FallibilityExpand for InfallibleMode {
    fn wrap_return_type(&self, ty: TokenStream) -> TokenStream {
        ty
    }

    fn error_type(&self) -> TokenStream {
        TokenStream::new()
    }

    fn doc_prefix(&self) -> &'static str {
        "Builds"
    }

    fn wrap_expr_ok(&self, expr: syn::Expr) -> TokenStream {
        quote!(#expr)
    }

    fn fn_name<D: DirectionExpand<I>, I: ImplModeExpand>(&self, impl_mode: &I, spec: &MappingContext) -> syn::Ident {
        impl_mode.fn_name().cloned().unwrap_or_else(|| {
            let prefix = "";
            let suffix = D::default_fn_suffix();
            format_ident!(
                "{}{}_{}",
                prefix,
                suffix,
                spec.derived_path().to_token_stream().to_string().to_snake_case()
            )
        })
    }

    fn compile_body<D: DirectionExpand<I>, I: ImplMode, E: ExtraFieldsExpand>(
        &self,
        _fields: &[MappedField<Self, D, I>],
        _extra: &E,
        construction_expr: TokenStream,
    ) -> TokenStream
    where
        D::Skip: SkipExpand,
    {
        construction_expr
    }

    fn transform_field_expr(transform: &ResolvedTransform, ident: &syn::Ident) -> TokenStream {
        match transform {
            ResolvedTransform::Default => {
                quote!(Into::into(#ident))
            }
            ResolvedTransform::With(expr) => {
                if let syn::Expr::Path(with_path) = expr.value() {
                    let crate_ident = model_mapper_crate();
                    quote!({
                        use #crate_ident::private::{RefMapper, ValueMapper};
                        (&(#with_path)).map_value(#ident)
                    })
                } else {
                    quote!(#expr)
                }
            }
            ResolvedTransform::Option(inner) => {
                let inner_expr = Self::transform_field_expr(inner, ident);
                quote!(#ident.map(|#ident| #inner_expr))
            }
            ResolvedTransform::Iterator(inner) => {
                let inner_expr = Self::transform_field_expr(inner, ident);
                quote!(#ident.into_iter().map(|#ident| #inner_expr).collect())
            }
            ResolvedTransform::Map(inner) => {
                let inner_expr = Self::transform_field_expr(inner, ident);
                quote!(#ident.into_iter().map(|(k, #ident)| (k, #inner_expr)).collect())
            }
            ResolvedTransform::Boxed(inner) => Self::build_box_expr(inner, ident, true, true),
            ResolvedTransform::BoxSource(inner) => Self::build_box_expr(inner, ident, true, false),
            ResolvedTransform::BoxTarget(inner) => Self::build_box_expr(inner, ident, false, true),
        }
    }

    fn field_construction_expr(
        target_ident: &syn::Ident,
        source_ident: &syn::Ident,
        transform: &ResolvedTransform,
    ) -> TokenStream {
        let build_expr = Self::transform_field_expr(transform, source_ident);
        quote!(#target_ident: #build_expr)
    }

    fn tuple_construction_part(var_ident: &syn::Ident, transform: &ResolvedTransform) -> TokenStream {
        Self::transform_field_expr(transform, var_ident)
    }

    fn build_box_expr(
        inner: &ResolvedTransform,
        ident: &syn::Ident,
        unbox_input: bool,
        box_output: bool,
    ) -> TokenStream {
        let crate_ident = model_mapper_crate();
        let inner_expr = Self::transform_field_expr(inner, ident);
        let input_expr = if unbox_input { quote!(*#ident) } else { quote!(#ident) };

        if box_output {
            quote!({
                let #ident = #input_expr;
                #crate_ident::private::Box::new(#inner_expr)
            })
        } else {
            quote!({
                let #ident = #input_expr;
                #inner_expr
            })
        }
    }

    fn emit_trait(&self, args: TraitEmitArgs<'_>) -> TokenStream {
        emit_trait_impl(
            args.impl_generics,
            args.where_clause,
            args.input_ty,
            args.output_ty,
            args.body,
        )
    }

    fn emit_custom(&self, args: CustomEmitArgs<'_>) -> TokenStream {
        emit_custom_fn(
            args.impl_generics,
            args.local_generics,
            args.where_clause,
            args.base_ident,
            args.fn_name,
            args.doc,
            args.fn_params,
            args.ret_ty,
            args.inst,
            args.body,
        )
    }
}

impl FallibilityExpand for FallibleMode {
    fn get_error_handler(handler: &Self::ErrorHandler) -> Option<&ResolvedErrorHandler> {
        handler.as_ref()
    }

    fn wrap_return_type(&self, ty: TokenStream) -> TokenStream {
        let base_err = self.base_error_ty_direct();
        let trait_err = match self.mode_direct() {
            FallibilityModeDetails::ShortCircuit => base_err.clone(),
            FallibilityModeDetails::Accumulate { accumulator_ty } => accumulator_ty.clone(),
        };
        quote!(Result<#ty, #trait_err>)
    }

    fn error_type(&self) -> TokenStream {
        let base_err = self.base_error_ty_direct();
        match self.mode_direct() {
            FallibilityModeDetails::ShortCircuit => quote!(#base_err),
            FallibilityModeDetails::Accumulate { accumulator_ty } => quote!(#accumulator_ty),
        }
    }

    fn doc_prefix(&self) -> &'static str {
        "Tries to build"
    }

    fn wrap_expr_ok(&self, expr: syn::Expr) -> TokenStream {
        quote!(::core::result::Result::Ok(#expr))
    }

    fn fn_name<D: DirectionExpand<I>, I: ImplModeExpand>(&self, custom: &I, spec: &MappingContext) -> syn::Ident {
        custom.fn_name().cloned().unwrap_or_else(|| {
            let prefix = "try_";
            let suffix = D::default_fn_suffix();
            format_ident!(
                "{}{}_{}",
                prefix,
                suffix,
                spec.derived_path().to_token_stream().to_string().to_snake_case()
            )
        })
    }

    fn compile_body<D: DirectionExpand<I>, I: ImplMode, E: ExtraFieldsExpand>(
        &self,
        fields: &[MappedField<Self, D, I>],
        extra: &E,
        construction_expr: TokenStream,
    ) -> TokenStream
    where
        D::Skip: SkipExpand,
    {
        build_try_body(
            fields,
            extra,
            construction_expr,
            self.base_error_ty_direct(),
            self.mode_direct(),
        )
    }

    fn transform_field_expr(transform: &ResolvedTransform, ident: &syn::Ident) -> TokenStream {
        match transform {
            ResolvedTransform::Default => {
                quote!(TryInto::try_into(#ident))
            }
            ResolvedTransform::With(expr) => {
                if let syn::Expr::Path(with_path) = expr.value() {
                    let crate_ident = model_mapper_crate();
                    quote!({
                        use #crate_ident::private::{RefMapper, ValueMapper};
                        (&(#with_path)).map_value(#ident)
                    })
                } else {
                    quote!(Ok::<_, anyhow::Error>(#expr))
                }
            }
            ResolvedTransform::Option(inner) => {
                let inner_expr = Self::transform_field_expr(inner, ident);
                quote!(#ident.map(|#ident| #inner_expr).transpose())
            }
            ResolvedTransform::Iterator(inner) => {
                let inner_expr = Self::transform_field_expr(inner, ident);
                quote!(#ident.into_iter().map(|#ident| #inner_expr).collect::<Result<_, _>>())
            }
            ResolvedTransform::Map(inner) => {
                let inner_expr = Self::transform_field_expr(inner, ident);
                quote!(
                    #ident.into_iter().map(|(k, #ident)| #inner_expr.map(|v| (k, v))).collect::<Result<_, _>>()
                )
            }
            ResolvedTransform::Boxed(inner) => Self::build_box_expr(inner, ident, true, true),
            ResolvedTransform::BoxSource(inner) => Self::build_box_expr(inner, ident, true, false),
            ResolvedTransform::BoxTarget(inner) => Self::build_box_expr(inner, ident, false, true),
        }
    }

    fn field_construction_expr(
        target_ident: &syn::Ident,
        _source_ident: &syn::Ident,
        _transform: &ResolvedTransform,
    ) -> TokenStream {
        quote!(#target_ident: #target_ident)
    }

    fn tuple_construction_part(var_ident: &syn::Ident, _transform: &ResolvedTransform) -> TokenStream {
        quote!(#var_ident)
    }

    fn build_box_expr(
        inner: &ResolvedTransform,
        ident: &syn::Ident,
        unbox_input: bool,
        box_output: bool,
    ) -> TokenStream {
        let crate_ident = model_mapper_crate();
        let inner_expr = Self::transform_field_expr(inner, ident);
        let input_expr = if unbox_input { quote!(*#ident) } else { quote!(#ident) };

        if box_output {
            quote!({
                let #ident = #input_expr;
                #inner_expr.map(|v| #crate_ident::private::Box::new(v))
            })
        } else {
            quote!({
                let #ident = #input_expr;
                #inner_expr
            })
        }
    }

    fn emit_trait(&self, args: TraitEmitArgs<'_>) -> TokenStream {
        let err_ty = self.error_type();
        emit_try_trait_impl(
            args.impl_generics,
            args.where_clause,
            args.input_ty,
            args.output_ty,
            &err_ty,
            args.body,
        )
    }

    fn emit_custom(&self, args: CustomEmitArgs<'_>) -> TokenStream {
        let ret_ty = self.wrap_return_type(args.ret_ty.clone());
        emit_custom_fn(
            args.impl_generics,
            args.local_generics,
            args.where_clause,
            args.base_ident,
            args.fn_name,
            args.doc,
            args.fn_params,
            &ret_ty,
            args.inst,
            args.body,
        )
    }
}

// =========================================================================
// ImplMode Codegen Extension Traits
// =========================================================================

/// Arguments passed into [`ImplModeExpand::generate_wrapper`].
pub(crate) struct CodeGenWrapperArgs<'a, F> {
    pub spec: &'a MappingContext,
    pub fallibility: &'a F,
    pub impl_generics: &'a syn::ImplGenerics<'a>,
    pub where_clause: Option<&'a syn::WhereClause>,
    pub derived_ty: &'a syn::TypePath,
    pub local_generics: &'a syn::TypeGenerics<'a>,
    pub body: TokenStream,
    pub external_params: &'a [TokenStream],
    pub inst: TokenStream,
}

/// Code generation mode expansion trait representing generated output configurations.
pub(crate) trait ImplModeExpand: ImplMode + Sized {
    /// Returns the explicit custom function name if configured, or `None` for trait implementations.
    fn fn_name(&self) -> Option<&syn::Ident>;

    /// Returns the binding variable name being deconstructed (`other`, `from`, or `self`).
    fn deconstruct_target<D: DirectionExpand<Self>>(&self) -> TokenStream;

    /// Wraps the compiled conversion body in either a trait `impl` block or a custom `impl` function.
    fn generate_wrapper<F: FallibilityExpand, D: DirectionExpand<Self>>(
        &self,
        args: CodeGenWrapperArgs<F>,
    ) -> TokenStream;

    /// Resolves local variable declarations for skipped fields in custom mapping functions.
    fn get_skipped_field_binding<D: DirectionExpand<Self>>(
        var_ident: &syn::Ident,
        ty: &syn::Type,
        has_default: bool,
    ) -> Option<TokenStream>;

    /// Generates the closure invocation expression for field value providers.
    fn provider_expression(&self, provider_ident: &syn::Ident) -> TokenStream;
}

impl ImplModeExpand for TraitMode {
    fn fn_name(&self) -> Option<&syn::Ident> {
        None
    }

    fn deconstruct_target<D: DirectionExpand<Self>>(&self) -> TokenStream {
        quote!(other)
    }

    fn generate_wrapper<F: FallibilityExpand, D: DirectionExpand<Self>>(
        &self,
        args: CodeGenWrapperArgs<F>,
    ) -> TokenStream {
        let input_ty = D::input_type(args.spec, args.derived_ty, args.local_generics);
        let output_ty = D::output_type(args.spec, args.derived_ty, args.local_generics);

        args.fallibility.emit_trait(TraitEmitArgs {
            impl_generics: args.impl_generics,
            where_clause: args.where_clause,
            input_ty: &input_ty,
            output_ty: &output_ty,
            body: args.body,
        })
    }

    fn get_skipped_field_binding<D: DirectionExpand<Self>>(
        _var_ident: &syn::Ident,
        _ty: &syn::Type,
        _has_default: bool,
    ) -> Option<TokenStream> {
        None
    }

    fn provider_expression(&self, provider_ident: &syn::Ident) -> TokenStream {
        quote!(#provider_ident)
    }
}

impl ImplModeExpand for CustomFnMode {
    fn fn_name(&self) -> Option<&syn::Ident> {
        self.fn_name_direct()
    }

    fn deconstruct_target<D: DirectionExpand<Self>>(&self) -> TokenStream {
        D::deconstruct_target()
    }

    fn generate_wrapper<F: FallibilityExpand, D: DirectionExpand<Self>>(
        &self,
        args: CodeGenWrapperArgs<F>,
    ) -> TokenStream {
        let from_ty = args.spec.base_ident();
        let fn_ident = args.fallibility.fn_name::<D, Self>(self, args.spec);
        let doc = D::custom_fn_doc(args.fallibility, args.spec, args.derived_ty);
        let ret_ty = D::custom_fn_ret_type(args.spec, args.derived_ty);
        let fn_params = D::custom_fn_params(args.derived_ty, args.external_params);

        args.fallibility.emit_custom(CustomEmitArgs {
            impl_generics: args.impl_generics,
            local_generics: args.local_generics,
            where_clause: args.where_clause,
            base_ident: from_ty,
            fn_name: &fn_ident,
            doc: &doc,
            fn_params: &fn_params,
            ret_ty: &ret_ty,
            inst: &args.inst,
            body: args.body,
        })
    }

    fn get_skipped_field_binding<D: DirectionExpand<Self>>(
        var_ident: &syn::Ident,
        ty: &syn::Type,
        has_default: bool,
    ) -> Option<TokenStream> {
        if has_default {
            None
        } else {
            D::custom_skipped_field_binding(var_ident, ty)
        }
    }

    fn provider_expression(&self, provider_ident: &syn::Ident) -> TokenStream {
        quote!(#provider_ident)
    }
}

// =========================================================================
// Fluent Builders
// =========================================================================

/// Fluent builder for constructing pattern matching deconstruction expressions for structs and enum variants.
struct DeconstructionBuilder<'a, F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> {
    fields: &'a [MappedField<F, D, I>],
    captured_fields: &'a [syn::Ident],
    ignore_extra: bool,
}

impl<'a, F, D, I> DeconstructionBuilder<'a, F, D, I>
where
    F: FallibilityMode,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
{
    fn new(fields: &'a [MappedField<F, D, I>]) -> Self {
        Self {
            fields,
            captured_fields: &[],
            ignore_extra: false,
        }
    }

    fn with_extra<E: ExtraFieldsExpand>(mut self, extra: &'a E) -> Self {
        self.captured_fields = extra.captured_fields().unwrap_or(&[]);
        self.ignore_extra = extra.ignore_extra();
        self
    }

    fn build_struct(self) -> TokenStream {
        if self.fields.is_empty() && self.captured_fields.is_empty() {
            return quote!({});
        }

        let is_named = self.fields.iter().any(|field| field.base_ident().is_some()) || !self.captured_fields.is_empty();
        if is_named {
            build_named_deconstruct_pattern::<F, D, I>(self.fields, self.captured_fields, self.ignore_extra)
        } else {
            let mut parts = Vec::new();
            for (ix, field) in self.fields.iter().enumerate() {
                let part = D::tuple_field_deconstruct_pattern(ix, field.skip().is_some());
                parts.push(part);
            }
            quote!(( #( #parts ),* ))
        }
    }

    fn build_variant(self) -> TokenStream {
        if self.fields.is_empty() && self.captured_fields.is_empty() {
            return quote!();
        }

        let is_named = self.fields.iter().any(|field| field.base_ident().is_some()) || !self.captured_fields.is_empty();
        if is_named {
            build_named_deconstruct_pattern::<F, D, I>(self.fields, self.captured_fields, self.ignore_extra)
        } else {
            let mut parts = Vec::new();
            for (ix, field) in self.fields.iter().enumerate() {
                let part = D::tuple_field_deconstruct_pattern(ix, field.skip().is_some());
                parts.push(part);
            }
            quote!(( #( #parts ),* ))
        }
    }
}

/// Constructs a `{ field1, field2, .. }` pattern matching deconstruction expression.
fn build_named_deconstruct_pattern<F, D, I>(
    fields: &[MappedField<F, D, I>],
    captured_fields: &[syn::Ident],
    ignore_extra: bool,
) -> TokenStream
where
    F: FallibilityMode,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
{
    let mut parts = Vec::new();
    for field in fields {
        let Some(f_ident) = field.base_ident().cloned() else {
            continue;
        };
        let pat_ident = field.pat_ident().unwrap_or(f_ident.clone());

        if let Some(part) = D::field_deconstruct_pattern(&f_ident, &pat_ident, field.skip().is_some()) {
            parts.push(part);
        }
    }
    for field in captured_fields {
        parts.push(quote!(#field));
    }

    if ignore_extra {
        parts.push(quote!(..));
    }
    quote!({ #( #parts ),* })
}

/// Fluent builder for constructing struct instantiation expressions.
struct StructConstructionBuilder<'a, F: FallibilityExpand, D: DirectionMode<I>, I: ImplMode> {
    fields: &'a [MappedField<F, D, I>],
    extra: Option<&'a D::StructExtra>,
}

impl<'a, F, D, I> StructConstructionBuilder<'a, F, D, I>
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
    D::StructExtra: ExtraFieldsExpand,
{
    fn new(fields: &'a [MappedField<F, D, I>]) -> Self {
        Self { fields, extra: None }
    }

    fn with_extra(mut self, extra: &'a D::StructExtra) -> Self {
        self.extra = Some(extra);
        self
    }

    fn build(self, derived_type_stripped: &syn::TypePath) -> TokenStream {
        let fields = self.fields;
        let extra = self.extra;

        let ignore_extra = D::struct_construction_ignore_extra(extra);
        let prefix = D::struct_construction_prefix(derived_type_stripped);

        if fields.is_empty() {
            let mut has_added = false;
            if let Some(extra_fields) = extra {
                extra_fields.for_each_added_field(|_, _, _| has_added = true);
            }
            if !has_added {
                return quote!(#prefix {});
            }
        }

        let mut is_named = fields.iter().any(|field| field.base_ident().is_some());
        if let Some(extra_fields) = extra {
            extra_fields.for_each_added_field(|_, _, _| is_named = true);
        }

        if is_named {
            let parts = D::struct_construction_parts::<F>(fields, extra, derived_type_stripped);
            let mut parts_stream = TokenStream::new();
            for part in parts {
                parts_stream.extend(quote!(#part,));
            }
            if ignore_extra {
                parts_stream.extend(quote!(..Default::default()));
            }
            quote!(#prefix { #parts_stream })
        } else {
            let mut parts = Vec::new();
            for (ix, field) in fields.iter().enumerate() {
                if field.skip().is_none() {
                    let var_ident = format_ident!("_{}", ix);
                    let expr = F::tuple_construction_part(&var_ident, field.transform());
                    parts.push(expr);
                }
            }
            quote!(#prefix ( #( #parts ),* ))
        }
    }
}

/// Fluent builder for constructing enum variant initialization expressions.
struct VariantConstructionBuilder<'a, F: FallibilityExpand, D: DirectionMode<I>, I: ImplMode> {
    fields: &'a [MappedField<F, D, I>],
    extra: Option<&'a D::VariantExtraFields>,
}

impl<'a, F, D, I> VariantConstructionBuilder<'a, F, D, I>
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
    D::VariantExtraFields: ExtraFieldsExpand,
{
    fn new(fields: &'a [MappedField<F, D, I>]) -> Self {
        Self { fields, extra: None }
    }

    fn with_extra(mut self, extra: &'a D::VariantExtraFields) -> Self {
        self.extra = Some(extra);
        self
    }

    fn build(self, derived_type_stripped: &syn::TypePath) -> TokenStream {
        let fields = self.fields;
        let extra = self.extra;

        let ignore_extra = D::variant_construction_ignore_extra(extra);

        if fields.is_empty() {
            let mut has_added = false;
            if let Some(extra_fields) = extra {
                extra_fields.for_each_added_field(|_, _, _| has_added = true);
            }
            if !has_added {
                return quote!();
            }
        }

        let mut is_named = fields.iter().any(|field| field.base_ident().is_some());
        if let Some(extra_fields) = extra {
            extra_fields.for_each_added_field(|_, _, _| is_named = true);
        }

        if is_named {
            let parts = D::variant_construction_parts::<F>(fields, extra, derived_type_stripped);
            let mut parts_stream = TokenStream::new();
            for part in parts {
                parts_stream.extend(quote!(#part,));
            }
            if ignore_extra {
                parts_stream.extend(quote!(..Default::default()));
            }
            quote!({ #parts_stream })
        } else {
            let mut parts = Vec::new();
            for (ix, field) in fields.iter().enumerate() {
                if field.skip().is_some() {
                    continue;
                }

                let var_ident = format_ident!("_{}", ix);
                let expr = F::tuple_construction_part(&var_ident, field.transform());
                parts.push(expr);
            }
            quote!(( #( #parts ),* ))
        }
    }
}

// =========================================================================
// AST Extension Traits & Implementations
// =========================================================================

/// AST extension trait for `MappedField` codegen helpers.
pub(crate) trait MappedFieldExpand {
    /// Resolves the pattern identifier to use when deconstructing a field (rename if specified, otherwise base ident).
    fn pat_ident(&self) -> Option<syn::Ident>;
}

impl<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> MappedFieldExpand for MappedField<F, D, I> {
    fn pat_ident(&self) -> Option<syn::Ident> {
        self.base_ident()
            .map(|base| self.rename().cloned().unwrap_or_else(|| base.clone()))
    }
}

/// AST extension trait for `MappedVariant` codegen helpers.
pub(crate) trait MappedVariantExpand {
    /// Resolves the target variant identifier on the derived enum (rename if specified, otherwise base ident).
    fn derived_ident(&self) -> &syn::Ident;
}

impl<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> MappedVariantExpand for MappedVariant<F, D, I> {
    fn derived_ident(&self) -> &syn::Ident {
        self.rename().unwrap_or_else(|| self.ident())
    }
}

/// AST extension trait representing skipped mapping configurations.
pub(crate) trait SkipExpand {
    /// Extracts the statically configured default value expression for skipped fields or variants, if present.
    fn default_expr(&self) -> Option<&syn::Expr>;
}

impl SkipExpand for ResolvedSkipTrait {
    fn default_expr(&self) -> Option<&syn::Expr> {
        Some(self.default_expr().value())
    }
}

impl SkipExpand for CustomSkippedFrom {
    fn default_expr(&self) -> Option<&syn::Expr> {
        self.default_expr().map(super::input::Located::value)
    }
}

impl SkipExpand for SkippedInto {
    fn default_expr(&self) -> Option<&syn::Expr> {
        self.default_expr().map(super::input::Located::value)
    }
}

impl SkipExpand for () {
    fn default_expr(&self) -> Option<&syn::Expr> {
        None
    }
}

/// AST extension trait representing synthesized/added fields and deconstruction rules.
pub(crate) trait ExtraFieldsExpand {
    /// Returns true if unmapped extra fields should be ignored during pattern deconstruction (via `..`) and target
    /// construction (via `..Default::default()`).
    fn ignore_extra(&self) -> bool;

    /// Returns the list of extra fields captured during struct or variant deconstruction, if any.
    fn captured_fields(&self) -> Option<&[syn::Ident]>;

    /// Executes a callback for each explicitly added field alongside its default expression and optional type.
    fn for_each_added_field<F>(&self, func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>);
}

impl ExtraFieldsExpand for () {
    fn ignore_extra(&self) -> bool {
        false
    }

    fn captured_fields(&self) -> Option<&[syn::Ident]> {
        None
    }

    fn for_each_added_field<F>(&self, _func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>),
    {
    }
}

impl ExtraFieldsExpand for StructDeconstructRules {
    fn ignore_extra(&self) -> bool {
        self.ignore_extra()
    }

    fn captured_fields(&self) -> Option<&[syn::Ident]> {
        Some(self.captured_fields().fields())
    }

    fn for_each_added_field<F>(&self, _func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>),
    {
    }
}

impl ExtraFieldsExpand for EnumDeconstructRules {
    fn ignore_extra(&self) -> bool {
        self.ignore_extra()
    }

    fn captured_fields(&self) -> Option<&[syn::Ident]> {
        None
    }

    fn for_each_added_field<F>(&self, _func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>),
    {
    }
}

impl ExtraFieldsExpand for VariantDeconstructRules {
    fn ignore_extra(&self) -> bool {
        self.ignore_extra()
    }

    fn captured_fields(&self) -> Option<&[syn::Ident]> {
        Some(self.captured_fields().fields())
    }

    fn for_each_added_field<F>(&self, _func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>),
    {
    }
}

impl ExtraFieldsExpand for TraitSynthesizedFields {
    fn ignore_extra(&self) -> bool {
        self.ignore_extra()
    }

    fn captured_fields(&self) -> Option<&[syn::Ident]> {
        None
    }

    fn for_each_added_field<F>(&self, mut func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>),
    {
        for added in self.fields() {
            func(added.field().value(), Some(added.mandatory_default().value()), None);
        }
    }
}

impl ExtraFieldsExpand for CustomSynthesizedFields {
    fn ignore_extra(&self) -> bool {
        self.ignore_extra()
    }

    fn captured_fields(&self) -> Option<&[syn::Ident]> {
        None
    }

    fn for_each_added_field<F>(&self, mut func: F)
    where
        F: FnMut(&syn::Ident, Option<&syn::Expr>, Option<&syn::Type>),
    {
        for added in self.fields() {
            match added {
                CustomAddedMappedField::WithDefault { field, default } => {
                    func(field.value(), Some(default.value()), None);
                }
                CustomAddedMappedField::WithType { field, ty } => {
                    let type_path = syn::Type::Path(ty.value().clone());
                    func(field.value(), None, Some(&type_path));
                }
            }
        }
    }
}

// =========================================================================
// Code Generation Helpers
// =========================================================================

/// Yields fields with skip or `with` transform first, then regular fields.
///
/// Strictly honors README priority order:
/// 1. Added fields defaults are evaluated in `default_decls` prior to body conversions.
/// 2. Custom-mapped (`with`) and skipped fields with defaults/providers are evaluated first in body conversions.
/// 3. Regular fields that move/consume source fields are evaluated last in body conversions.
fn with_fields_first<F, D, I>(fields: &[MappedField<F, D, I>]) -> impl Iterator<Item = &MappedField<F, D, I>>
where
    F: FallibilityMode,
    D: DirectionMode<I>,
    I: ImplMode,
{
    let is_priority = |field: &&MappedField<F, D, I>| {
        field.skip().is_some() || matches!(field.transform(), ResolvedTransform::With(_))
    };
    fields
        .iter()
        .filter(is_priority)
        .chain(fields.iter().filter(move |field| !is_priority(field)))
}

/// Generates `let` declaration statements for added fields and skipped fields with default expressions on structs.
fn build_struct_default_decls<F, D, I>(
    fields: &[MappedField<F, D, I>],
    extra: &D::StructExtra,
    _impl_mode: &I,
) -> TokenStream
where
    F: FallibilityMode,
    D: DirectionExpand<I>,
    I: ImplModeExpand,
    D::Skip: SkipExpand,
    D::StructExtra: ExtraFieldsExpand,
{
    let mut default_decls = TokenStream::new();
    extra.for_each_added_field(|name, default_expr, _| {
        if let Some(expr) = default_expr {
            default_decls.extend(quote!(let #name = #expr;));
        }
    });
    for field in fields {
        if let Some(skip) = field.skip() {
            let Some(f_ident) = field.base_ident() else {
                continue;
            };
            let var_ident = field.pat_ident().unwrap_or_else(|| f_ident.clone());
            let ty = field.base_ty();
            if let Some(expr) = skip.value().default_expr() {
                default_decls.extend(quote!(let #var_ident: #ty = #expr;));
            } else if let Some(binding) = I::get_skipped_field_binding::<D>(&var_ident, ty, false) {
                default_decls.extend(binding);
            } else {
                // No default expression or skipped field binding available.
            }
        }
    }
    default_decls
}

/// Generates `let` declaration statements for added fields and skipped fields (including closure providers) within an
/// enum variant match arm.
fn build_variant_default_decls<F, D, I>(variant: &MappedVariant<F, D, I>, impl_mode: &I) -> TokenStream
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplModeExpand,
    D::Skip: SkipExpand,
    D::VariantExtraFields: ExtraFieldsExpand,
{
    let mut variant_default_decls = TokenStream::new();
    variant.extra().for_each_added_field(|name, default_expr, ty| {
        if let Some(expr) = default_expr {
            variant_default_decls.extend(quote!(let #name = #expr;));
            return;
        }
        if ty.is_some() {
            let field_provider = format_ident!("{}_provider", name);
            let mut in_scope_vars = Vec::new();
            for (ix_inner, f_inner) in variant.fields().iter().enumerate() {
                let var_ident = f_inner.pat_ident().unwrap_or_else(|| format_ident!("_{}", ix_inner));
                in_scope_vars.push(quote!(&#var_ident));
            }
            let provider_expr = impl_mode.provider_expression(&field_provider);
            variant_default_decls.extend(quote!(let #name = (#provider_expr)( #( #in_scope_vars ),* );));
        }
    });
    for field in variant.fields() {
        let Some(skip) = field.skip() else {
            continue;
        };
        let Some(f_ident) = field.base_ident() else {
            continue;
        };
        let var_ident = field.pat_ident().unwrap_or_else(|| f_ident.clone());
        let ty = field.base_ty();
        if let Some(expr) = skip.default_expr() {
            variant_default_decls.extend(quote!(let #var_ident: #ty = #expr;));
        } else {
            let field_provider = format_ident!("{}_provider", var_ident);
            let mut in_scope_vars = Vec::new();
            for (ix_inner, f_inner) in variant.fields().iter().enumerate() {
                if f_inner.skip().is_none() {
                    let var_inner = f_inner.pat_ident().unwrap_or_else(|| format_ident!("_{}", ix_inner));
                    in_scope_vars.push(quote!(&#var_inner));
                }
            }
            let provider_expr = impl_mode.provider_expression(&field_provider);
            variant_default_decls.extend(quote!(let #var_ident: #ty = (#provider_expr)( #( #in_scope_vars ),* );));
        }
    }
    variant_default_decls
}

/// Compiles the fallible function body block, delegating to either error-accumulating or short-circuiting mode.
fn build_try_body<F, D, I, E>(
    fields: &[MappedField<F, D, I>],
    extra: &E,
    construction_expr: TokenStream,
    base_err_ty: &syn::Type,
    mode: &FallibilityModeDetails,
) -> TokenStream
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
    E: ExtraFieldsExpand,
{
    let mut in_scope_vars = Vec::new();
    for (ix, field) in fields.iter().enumerate() {
        if field.skip().is_none() {
            let src_ident = field.base_ident().cloned().unwrap_or_else(|| format_ident!("_{}", ix));
            let pat_ident = field.pat_ident().unwrap_or_else(|| src_ident.clone());
            let dest_ident = D::dest_ident(&src_ident, &pat_ident);
            in_scope_vars.push(quote!(&#dest_ident));
        }
    }

    if let FallibilityModeDetails::Accumulate { accumulator_ty } = mode {
        build_try_body_accumulate(
            fields,
            extra,
            construction_expr,
            base_err_ty,
            accumulator_ty,
            &in_scope_vars,
        )
    } else {
        build_try_body_short_circuit(fields, extra, construction_expr, &in_scope_vars)
    }
}

/// Compiles an error-accumulating fallible function body that evaluates all field conversions and collects errors into
/// a container.
fn build_try_body_accumulate<F, D, I, E>(
    fields: &[MappedField<F, D, I>],
    _extra: &E,
    construction_expr: TokenStream,
    base_err_ty: &syn::Type,
    accumulator_ty: &syn::Type,
    _in_scope_vars: &[TokenStream],
) -> TokenStream
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
    E: ExtraFieldsExpand,
{
    let mut field_conversions = TokenStream::new();
    let mut res_idents = Vec::new();
    let mut dest_idents = Vec::new();

    for field in with_fields_first(fields) {
        if field.skip().is_some() {
            continue;
        }

        let ix = fields.iter().position(|x| ptr::eq(x, field)).unwrap_or(0);
        let src_ident = field.base_ident().cloned().unwrap_or_else(|| format_ident!("_{}", ix));
        let pat_ident = field.pat_ident().unwrap_or_else(|| src_ident.clone());
        let dest_ident = D::dest_ident(&src_ident, &pat_ident);
        let bound_ident = D::bound_ident(&src_ident, &pat_ident);

        let res_ident = format_ident!("{}_res", dest_ident);
        let raw_expr = F::transform_field_expr(field.transform(), &bound_ident);

        let map_err = if let Some(handler) = F::get_error_handler(field.error_handler()) {
            match handler {
                ResolvedErrorHandler::Value(val) => quote!(|_| #val),
                ResolvedErrorHandler::With(expr) => quote!(#expr),
            }
        } else {
            quote!(<#base_err_ty as ::core::convert::From<_>>::from)
        };

        field_conversions.extend(quote!(
            let #res_ident = #raw_expr.map_err(#map_err);
        ));

        res_idents.push(res_ident);
        dest_idents.push(dest_ident);
    }

    // Guard against unreachable pattern on 0-field accumulations
    if res_idents.is_empty() {
        return quote!({
            ::core::result::Result::Ok(#construction_expr)
        });
    }

    let crate_ident = model_mapper_crate();

    // Generate error accumulation pushes:
    let mut error_pushes = TokenStream::new();
    for res_ident in &res_idents {
        error_pushes.extend(quote!(
            if let ::core::result::Result::Err(e) = #res_ident {
                errors.push(e);
            }
        ));
    }

    quote!({
        #field_conversions
        match ( #( #res_idents ),* ) {
            ( #( ::core::result::Result::Ok(#dest_idents) ),* ) => {
                ::core::result::Result::Ok(#construction_expr)
            }
            ( #( #res_idents ),* ) => {
                let mut errors = #crate_ident::private::Vec::new();
                #error_pushes
                ::core::result::Result::Err(errors.into_iter().collect::<#accumulator_ty>())
            }
        }
    })
}

/// Compiles a short-circuiting fallible function body using `?` operators to abort early on the first error.
fn build_try_body_short_circuit<F, D, I, E>(
    fields: &[MappedField<F, D, I>],
    _extra: &E,
    construction_expr: TokenStream,
    _in_scope_vars: &[TokenStream],
) -> TokenStream
where
    F: FallibilityExpand,
    D: DirectionExpand<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
    E: ExtraFieldsExpand,
{
    let mut field_conversions = TokenStream::new();

    for field in with_fields_first(fields) {
        let ix = fields.iter().position(|x| ptr::eq(x, field)).unwrap_or(0);
        if field.skip().is_some() {
            continue;
        }

        let src_ident = field.base_ident().cloned().unwrap_or_else(|| format_ident!("_{}", ix));
        let pat_ident = field.pat_ident().unwrap_or_else(|| src_ident.clone());
        let dest_ident = D::dest_ident(&src_ident, &pat_ident);
        let bound_ident = D::bound_ident(&src_ident, &pat_ident);

        let raw_expr = F::transform_field_expr(field.transform(), &bound_ident);

        let stmt = if let Some(handler) = F::get_error_handler(field.error_handler()) {
            match handler {
                ResolvedErrorHandler::Value(val) => quote!(let #dest_ident = #raw_expr.map_err(|_| #val)?;),
                ResolvedErrorHandler::With(expr) => quote!(let #dest_ident = #raw_expr.map_err(#expr)?;),
            }
        } else {
            quote!(let #dest_ident = #raw_expr ?;)
        };

        field_conversions.extend(stmt);
    }

    quote!({
        #field_conversions
        ::core::result::Result::Ok(#construction_expr)
    })
}

/// Core generic parameter processor: merges base generics with derived type parameters and builds generic rename
/// mappings to prevent collisions.
fn process_generics<P>(
    base_generics: &syn::Generics,
    derived_ty: &syn::TypePath,
    make_predicate: P,
) -> (syn::Generics, syn::TypePath, HashMap<syn::Ident, syn::Ident>)
where
    P: Fn(&syn::Ident, &syn::Ident) -> syn::WherePredicate,
{
    let mut all_generics = base_generics.clone();
    let base_params: HashSet<syn::Ident> = base_generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(ty) = param {
                Some(ty.ident.clone())
            } else {
                None
            }
        })
        .collect();

    let mut collector = TypePathCollector { idents: HashSet::new() };
    collector.visit_type_path(derived_ty);
    let original_derived_idents = collector.idents;

    let mut generics_rename_map = HashMap::new();
    let mut new_params = Vec::new();
    let mut new_predicates: Vec<syn::WherePredicate> = Vec::new();

    for ident in &original_derived_idents {
        if base_params.contains(ident) {
            let new_ident = format_ident!("{}Src", ident);
            generics_rename_map.insert(ident.clone(), new_ident.clone());

            new_params.push(syn::GenericParam::Type(parse_quote!(#new_ident)));

            let predicate = make_predicate(ident, &new_ident);
            new_predicates.push(predicate);
        } else {
            let is_concrete_type = if derived_ty.path.segments.len() == 1
                && let Some(seg) = derived_ty.path.segments.first()
            {
                seg.ident == *ident && seg.arguments.is_empty()
            } else {
                false
            };

            if !is_concrete_type {
                new_params.push(syn::GenericParam::Type(parse_quote!(#ident)));
            }
        }
    }

    all_generics.params.extend(new_params);
    if !new_predicates.is_empty() {
        let where_clause = all_generics.make_where_clause();
        where_clause.predicates.extend(new_predicates);
    }

    let mut replacer = TypePathReplacer {
        map: &generics_rename_map,
    };
    let derived_ty_with_generics = replacer.fold_type_path(derived_ty.clone());

    (all_generics, derived_ty_with_generics, generics_rename_map)
}

/// Appends `where` predicates for field-level type overrides (`other_ty`) to the generic constraints.
fn add_generics_bounds<F, D, I, P>(
    all_generics: &mut syn::Generics,
    fields: &[MappedField<F, D, I>],
    generics_rename_map: &HashMap<syn::Ident, syn::Ident>,
    make_predicate: P,
) where
    F: FallibilityMode,
    D: DirectionMode<I>,
    I: ImplMode,
    P: Fn(&syn::Ident, &syn::Type) -> syn::WherePredicate,
{
    for field in fields {
        if let Some(other_ty) = field.other_ty() {
            let resolved_ident = generics_rename_map.get(other_ty).unwrap_or(other_ty);
            let field_ty = field.base_ty();
            let where_clause = all_generics.make_where_clause();
            let predicate = make_predicate(resolved_ident, field_ty);
            where_clause.predicates.push(predicate);
        }
    }
}

/// Strips generic type arguments from a [`syn::TypePath`], leaving only the un-parameterized path segments.
fn strip_generics(ty: &syn::TypePath) -> syn::TypePath {
    let mut new_ty = ty.clone();
    for segment in &mut new_ty.path.segments {
        segment.arguments = syn::PathArguments::None;
    }
    new_ty
}

/// Extracts external parameter tokens and type info for skipped fields without defaults in `From` custom struct
/// mappings.
fn get_struct_from_custom_external_params<F, D, I>(
    fields: &[MappedField<F, D, I>],
) -> (Vec<TokenStream>, Vec<(syn::Ident, syn::Type)>)
where
    F: FallibilityMode,
    D: DirectionMode<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
{
    let mut params = Vec::new();
    let mut param_info = Vec::new();
    for field in fields {
        let Some(skip) = field.skip() else {
            continue;
        };

        if skip.value().default_expr().is_some() {
            continue;
        }

        let Some(i) = field.base_ident() else {
            continue;
        };

        let ty = field.base_ty().clone();
        let ident = field.pat_ident().unwrap_or_else(|| i.clone());
        param_info.push((ident.clone(), ty.clone()));
        params.push(quote!(#ident: #ty));
    }
    (params, param_info)
}

/// Generates the transient input struct declaration and instantiation tokens for custom function parameters.
fn get_struct_custom_param_instantiation(
    base_ident: &syn::Ident,
    param_info: &[(syn::Ident, syn::Type)],
) -> TokenStream {
    if param_info.is_empty() {
        return TokenStream::new();
    }
    let input_struct_ident = format_ident!("{}Input", base_ident);
    let fields_decl: Vec<TokenStream> = param_info.iter().map(|(ident, ty)| quote!(#ident: #ty)).collect();
    let fields_inst: Vec<syn::Ident> = param_info.iter().map(|(ident, _)| ident.clone()).collect();
    quote!(
        struct #input_struct_ident {
            #( #fields_decl ),*
        }
        let input = #input_struct_ident {
            #( #fields_inst ),*
        };
    )
}

/// Extracts provider closure parameter tokens for skipped fields without defaults in `From` custom enum mappings.
fn get_enum_from_custom_external_params<F, D, I>(variants: &[MappedVariant<F, D, I>]) -> Vec<TokenStream>
where
    F: FallibilityMode,
    D: DirectionMode<I>,
    I: ImplMode,
    D::Skip: SkipExpand,
{
    let mut params = Vec::new();
    for variant in variants {
        for field in variant.fields() {
            let Some(skip) = field.skip() else {
                continue;
            };
            if skip.value().default_expr().is_some() {
                continue;
            }
            let Some(i) = field.base_ident() else {
                continue;
            };

            let ident = field.pat_ident().unwrap_or_else(|| i.clone());
            let field_provider = format_ident!("{ident}_provider");
            let ty = field.base_ty();
            let source_tys: Vec<syn::Type> = variant
                .fields()
                .iter()
                .filter(|f_inner| f_inner.skip().is_none())
                .map(|f_inner| {
                    f_inner
                        .other_ty()
                        .map(|other_ident| syn::parse_quote!(#other_ident))
                        .unwrap_or_else(|| f_inner.base_ty().clone())
                })
                .collect();
            params.push(quote!(#field_provider: impl FnOnce( #( &#source_tys ),* ) -> #ty));
        }
    }
    params
}

/// Extracts provider closure parameter tokens for added fields without defaults in `Into` custom enum mappings.
fn get_enum_into_custom_external_params<F, D, I>(variants: &[MappedVariant<F, D, I>]) -> Vec<TokenStream>
where
    F: FallibilityMode,
    D: DirectionMode<I>,
    I: ImplMode,
    D::VariantExtraFields: ExtraFieldsExpand,
{
    let mut params = Vec::new();
    for variant in variants {
        variant.extra().for_each_added_field(|field_name, default_expr, ty| {
            if default_expr.is_none() {
                let field_provider = format_ident!("{field_name}_provider");
                let actual_ty = ty.cloned().unwrap_or_else(|| syn::parse_quote!(String));
                let source_tys: Vec<&syn::Type> = variant.fields().iter().map(MappedField::base_ty).collect();
                params.push(quote!(#field_provider: impl FnOnce( #( &#source_tys ),* ) -> #actual_ty));
            }
        });
    }
    params
}

/// Resolves the crate path (`::model_mapper` or renamed dependency) for runtime macro imports.
fn model_mapper_crate() -> TokenStream {
    match crate_name("model-mapper") {
        Ok(FoundCrate::Name(name)) => {
            let id = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#id)
        }
        Ok(FoundCrate::Itself) | Err(_) => quote!(::model_mapper),
    }
}
