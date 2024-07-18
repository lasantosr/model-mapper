use darling::{
    ast::{Data, Fields},
    util::{Override, SpannedValue},
    FromDeriveInput,
};
use heck::ToSnakeCase;
use macro_field_utils::{FieldInfo, FieldsCollector, FieldsHelper, VariantsHelper};
use proc_macro2::TokenStream;
use proc_macro_error::abort_if_dirty;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::input::*;

pub(crate) fn r#impl(input: syn::DeriveInput) -> TokenStream {
    // Parse input
    let opts = match MapperOpts::from_derive_input(&input) {
        Ok(o) => o,
        Err(e) => {
            return e.write_errors();
        }
    };

    let ident = &opts.ident;

    // Retrieve the items to derive
    let derive_items = opts.items();

    // Validate input
    derive_items
        .iter()
        .for_each(|i| ItemInput::validate(i, opts.data.is_enum()));
    abort_if_dirty();

    match &opts.data {
        Data::Struct(s) => {
            s.iter().for_each(|f| f.validate(&derive_items));
        }
        Data::Enum(e) => {
            e.iter().for_each(|v| v.validate(&derive_items));
        }
    }
    abort_if_dirty();

    let mut output = TokenStream::new();

    // Derive each requested type
    for derive in derive_items {
        match &opts.data {
            Data::Struct(struct_fields) => {
                // Derive the struct
                derive_struct(ident, &opts.generics, derive, struct_fields).to_tokens(&mut output);
            }
            Data::Enum(enum_variants) => {
                // Derive the enum
                derive_enum(ident, &opts.generics, derive, enum_variants).to_tokens(&mut output);
            }
        }
    }

    output
}

fn derive_struct(
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: ItemInput,
    struct_fields: &Fields<FieldReceiver>,
) -> TokenStream {
    let mut output = TokenStream::new();

    // Derive `From`
    if let Some(from) = derive.from.as_ref() {
        derive_struct_from(from, ident, generics, &derive, struct_fields, false).to_tokens(&mut output);
    }

    // Derive reverse `From`
    if let Some(into) = derive.into.as_ref() {
        derive_struct_into(into, ident, generics, &derive, struct_fields, false).to_tokens(&mut output);
    }

    // Derive `TryFrom`
    if let Some(try_from) = derive.try_from.as_ref() {
        derive_struct_from(try_from, ident, generics, &derive, struct_fields, true).to_tokens(&mut output);
    }

    // Derive reverse `TryFrom`
    if let Some(try_into) = derive.try_into.as_ref() {
        derive_struct_into(try_into, ident, generics, &derive, struct_fields, true).to_tokens(&mut output);
    }

    output
}

fn derive_enum(
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: ItemInput,
    enum_variants: &[VariantReceiver],
) -> TokenStream {
    let mut output = TokenStream::new();

    // Derive `From`
    if let Some(from) = derive.from.as_ref() {
        derive_enum_from(from, ident, generics, &derive, enum_variants, false).to_tokens(&mut output);
    }

    // Derive reverse `From`
    if let Some(into) = derive.into.as_ref() {
        derive_enum_into(into, ident, generics, &derive, enum_variants, false).to_tokens(&mut output);
    }

    // Derive `TryFrom`
    if let Some(try_from) = derive.try_from.as_ref() {
        derive_enum_from(try_from, ident, generics, &derive, enum_variants, true).to_tokens(&mut output);
    }

    // Derive reverse `TryFrom`
    if let Some(try_into) = derive.try_into.as_ref() {
        derive_enum_into(try_into, ident, generics, &derive, enum_variants, true).to_tokens(&mut output);
    }

    output
}

fn derive_struct_from(
    from: &SpannedValue<Override<DeriveInput>>,
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    struct_fields: &Fields<FieldReceiver>,
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Derive from the other type into self
    let from_ty = derive.path.as_ref();
    let into_ty = ident;

    // The other type has
    let from_ty_fields_helper = FieldsHelper::new(struct_fields)
        // every non-skipped field of self
        .filtering(|_ix, f| f.skip_for(from_ty).is_none())
        // every additional field explicitly set
        .ignore_extra(derive.add.iter().map(|f| f.field.as_ref()))
        // any other field ignored, if set
        .ignore_all_extra(derive.ignore_extra.is_present());

    // Self type has
    let into_ty_fields_helper = FieldsHelper::new(struct_fields)
        // every non-skipped field (as it's on the from)
        .filtering(|_ix, f| f.skip_for(from_ty).is_none())
        // skipped fields with the custom value provided
        .include_default_with(
            struct_fields
                .iter()
                .filter_map(|f| f.skip_for(from_ty).map(|skip| (f, skip)))
                .filter_map(|(f, skip)| {
                    f.ident.as_ref().map(|field| {
                        (
                            field,
                            // populated with
                            skip.as_ref()
                                .explicit()
                                .and_then(|s| s.default.as_deref())
                                // if default enabled: the default expression provided or Default::default()
                                .map(|d|
                                    d.clone()
                                        .explicit()
                                        .map(|e| e.value)
                                        .unwrap_or_else(|| parse_quote!(Default::default()))
                                )
                                // or just the field ident, as it will be provided on the function parameters
                                .unwrap_or_else(|| parse_quote!(#field)),
                        )
                    })
                }),
        );

    // Deconstruct the `from` input to retrieve the inner fields
    let deconstructed_from = from_ty_fields_helper
        .left_collector(|ix, f| {
            let ident = if let Some(rename) = f.rename_for(from_ty) {
                rename.clone()
            } else {
                f.as_ident(ix)
            };
            quote!(#ident)
        })
        .right_collector(FieldsCollector::ident)
        .collect();

    // Produce `into` body using the `with`
    let into_body = into_ty_fields_helper
        .right_collector(|ix, f| {
            let ident = f.as_ident(ix);
            if is_try {
                if let Some(try_with) = f.with_from_for(from_ty) {
                    quote!(#try_with(#ident)?)
                } else {
                    quote!(TryInto::try_into(#ident)?)
                }
            } else if let Some(with) = f.with_from_for(from_ty) {
                quote!(#with(#ident))
            } else {
                quote!(Into::into(#ident))
            }
        })
        .collect();

    // If we're deriving a custom function
    if let Some(custom) = Override::as_ref(from).explicit().and_then(|e| e.custom.as_deref()) {
        // Collect the skipped fields that doesn't have a default value
        let external_fields = struct_fields
            .iter()
            .filter(|f| {
                f.skip_for(from_ty)
                    .filter(|map| map.as_ref().explicit().map(|s| s.default.is_none()).unwrap_or(true))
                    .is_some()
            })
            .filter_map(|f| {
                let ty = &f.ty;
                f.ident.as_ref().map(|i| quote!(#i: #ty))
            })
            .collect::<Vec<_>>();

        // Compute the function name, wether is provided or not
        let fn_name = custom.clone().explicit().unwrap_or_else(|| {
            format_ident!(
                "{}_{}",
                if is_try { "try_from" } else { "from" },
                from_ty.to_token_stream().to_string().to_snake_case()
            )
        });

        // Compute the method doc
        let doc = format!(
            "{} a new [{into_ty}] from a [{}]",
            if is_try { "Tries to build" } else { "Builds" },
            from_ty.to_token_stream().to_string().replace(' ', "")
        );

        // Implement the custom function
        if is_try {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #into_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    pub fn #fn_name(from: #from_ty #ty_generics, #( #external_fields ),*)
                        -> ::std::result::Result<Self, ::anyhow::Error> {
                        let #from_ty #deconstructed_from = from;
                        Ok(Self #into_body)
                    }
                }
            )
        } else {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #into_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    pub fn #fn_name(from: #from_ty #ty_generics, #( #external_fields ),*) -> Self {
                        let #from_ty #deconstructed_from = from;
                        Self #into_body
                    }
                }
            )
        }
    } else if is_try {
        // Implement the [TryFrom] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics TryFrom<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                type Error = ::anyhow::Error;

                fn try_from(#from_ty #deconstructed_from: #from_ty #ty_generics)
                    -> ::std::result::Result<Self, <Self as TryFrom<#from_ty #ty_generics>>::Error> {

                    Ok(Self #into_body)
                }
            }
        )
    } else {
        // Implement the [From] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics From<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                fn from(#from_ty #deconstructed_from: #from_ty #ty_generics) -> Self {
                    Self #into_body
                }
            }
        )
    }
}

fn derive_struct_into(
    into: &SpannedValue<Override<DeriveInput>>,
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    struct_fields: &Fields<FieldReceiver>,
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Derive from self into the other type
    let from_ty = ident;
    let into_ty = derive.path.as_ref();

    // Self type has
    let from_ty_fields_helper = FieldsHelper::new(struct_fields)
        // every non-skipped field
        .filtering(|_ix, f| f.skip_for(into_ty).is_none())
        // and skipped fields ignored
        .ignore_extra(
            struct_fields
                .iter()
                .filter(|f| f.skip_for(into_ty).is_some())
                .filter_map(|f| f.ident.as_ref()),
        );

    // The other type has
    let into_ty_fields_helper = FieldsHelper::new(struct_fields)
        // every non-skipped field
        .filtering(|_ix, f| f.skip_for(into_ty).is_none())
        // every additional field explicitly set
        .include_default_with(derive.add.iter().map(|i| {
            let field = i.field.as_ref();
            (
                field,
                // populated with
                i.default
                    .as_deref()
                    // if default enabled: the default expression provided or Default::default()
                    .map(|d| d
                        .clone()
                        .explicit()
                        .map(|d| d.value)
                        .unwrap_or_else(|| parse_quote!(Default::default()))
                    )
                    // or just the field ident, as it will be provided on the function parameters
                    .unwrap_or_else(|| parse_quote!(#field)),
            )
        }))
        // any other ignored field, with the default value
        .include_all_default(derive.ignore_extra.is_present());

    // Deconstruct the `from` input to retrieve the inner fields
    let deconstructed_from = from_ty_fields_helper.right_collector(FieldsCollector::ident).collect();

    // Produce `into` body using the `with`
    let into_body = into_ty_fields_helper
        .left_collector(|ix, f| {
            let ident = if let Some(rename) = f.rename_for(into_ty) {
                rename.clone()
            } else {
                f.as_ident(ix)
            };
            quote!(#ident)
        })
        .right_collector(|ix, f| {
            let ident = f.as_ident(ix);
            if is_try {
                if let Some(try_with) = f.with_into_for(into_ty) {
                    quote!(#try_with(#ident)?)
                } else {
                    quote!(TryInto::try_into(#ident)?)
                }
            } else if let Some(with) = f.with_into_for(into_ty) {
                quote!(#with(#ident))
            } else {
                quote!(Into::into(#ident))
            }
        })
        .collect();

    // If we're deriving a custom function
    if let Some(custom) = Override::as_ref(into).explicit().and_then(|e| e.custom.as_deref()) {
        // Collect the additional fields that doesn't have a default value
        let external_fields = derive
            .add
            .iter()
            .filter(|a| a.default.is_none())
            .map(|f| {
                let ident = f.field.as_ref();
                let ty = f.ty.as_ref().expect("'ty' must be provided").as_ref();
                quote!(#ident: #ty)
            })
            .collect::<Vec<_>>();

        // Compute the function name, wether is provided or not
        let fn_name = custom.clone().explicit().unwrap_or_else(|| {
            format_ident!(
                "{}_{}",
                if is_try { "try_into" } else { "into" },
                into_ty.to_token_stream().to_string().to_snake_case()
            )
        });

        // Compute the method doc
        let doc = format!(
            "{} a new [{}] from a [{from_ty}]",
            if is_try { "Tries to build" } else { "Builds" },
            into_ty.to_token_stream().to_string().replace(' ', "")
        );

        // Implement the custom function
        if is_try {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #from_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    pub fn #fn_name(self, #( #external_fields ),*)
                        -> ::std::result::Result<#into_ty #ty_generics, ::anyhow::Error> {
                        let #from_ty #deconstructed_from = self;
                        Ok(#into_ty #into_body)
                    }
                }
            )
        } else {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #from_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    pub fn #fn_name(self, #( #external_fields ),*) -> #into_ty #ty_generics {
                        let #from_ty #deconstructed_from = self;
                        #into_ty #into_body
                    }
                }
            )
        }
    } else if is_try {
        // Implement the [TryFrom] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics TryFrom<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                type Error = ::anyhow::Error;

                fn try_from(#from_ty #deconstructed_from: #from_ty #ty_generics)
                    -> ::std::result::Result<Self, <Self as TryFrom<#from_ty #ty_generics>>::Error> {

                    Ok(#into_ty #into_body)
                }
            }
        )
    } else {
        // Implement the [From] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics From<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                fn from(#from_ty #deconstructed_from: #from_ty #ty_generics) -> Self {
                    #into_ty #into_body
                }
            }
        )
    }
}

fn derive_enum_from(
    from: &SpannedValue<Override<DeriveInput>>,
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    enum_variants: &[VariantReceiver],
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Derive from the other type into self
    let from_ty = derive.path.as_ref();
    let into_ty = ident;

    // The other type has
    let match_body = VariantsHelper::new(enum_variants)
        // every non-skipped variant of self
        .filtering_variants(|v| v.skip_for(from_ty).is_none())
        // every additional variant explicitly set
        .include_extra_variants(derive.add.iter().map(|i| {
            let field = i.field.as_ref();
            (
                quote!(#from_ty::#field { .. }),
                // populated with
                Some(i.default
                    .as_deref()
                    .expect("'default' must be provided")
                    .clone()
                    // the default expression provided
                    .explicit()
                    .map(|d| d.value)
                    // or Default::default()
                    .unwrap_or_else(|| parse_quote!(Default::default()))
                ),
            )
        }))
        // any other variant ignored, if any
        .ignore_all_extra_variants(if derive.ignore_extra.is_present(){
            Some(quote!(Default::default()))
        } else {
            None
        })
        // the left side of the match will be the from variant, along with its fields (if any)
        .left_collector(|v, fields| {
            // the other type variant name will be the same name or the rename ident
            let ident = if let Some(rename) = v.rename_for(from_ty) {
                rename
            } else {
                &v.ident
            };
            // the other type variant has
            let from_fields = fields
                // every none-skipped field of self variant
                .filtering(|_ix, f| f.skip_for(from_ty).is_none())
                // ignoring every additional field explicitly set
                .ignore_extra(v
                    .additional_for(from_ty)
                    .map(|i| i.iter().map(|i| i.field.as_ref()).collect::<Vec<_>>())
                    .unwrap_or_default())
                // and ignoring any other field, if set
                .ignore_all_extra(v.ignore_extra_for(from_ty))
                // where we collect each field ident (or the rename) deconstructed
                .left_collector(|ix, f| {
                    let ident = if let Some(rename) = f.rename_for(from_ty) {
                        rename.clone()
                    } else {
                        f.as_ident(ix)
                    };
                    quote!(#ident)
                })
                // as the field ident
                .right_collector(FieldsCollector::ident)
                .collect();

            quote!( #from_ty::#ident #from_fields )
        })
        // the right side of the match will be the into variant, along with its fields (if any)
        .right_collector(|v, fields| {
            let ident = &v.ident;
            // Self type variant has
            let into_fields = fields
                // every non-skipped field (as it's on the from)
                .filtering(|_ix, f| f.skip_for(from_ty).is_none())
                // skipped fields with the custom value provided
                .include_default_with(
                    v.fields
                        .iter()
                        .filter_map(|f| f.skip_for(from_ty).map(|skip| (f, skip)))
                        .filter_map(|(f, skip)| {
                            f.ident.as_ref().map(|field| {
                                (
                                    field,
                                    // populated with
                                    skip.as_ref()
                                        .explicit()
                                        .and_then(|s| s.default.as_deref())
                                        // if default enabled: the default expression provided or Default::default()
                                        .map(|d| d
                                            .clone()
                                            .explicit()
                                            .map(|e| e.value)
                                            .unwrap_or_else(|| parse_quote!(Default::default())))
                                        // or just the field ident, as it will be provided on the function parameters
                                        .unwrap_or_else(|| {
                                            let field_provider = format_ident!("{field}_provider");
                                            parse_quote!(#field_provider())
                                        }),
                                )
                            })
                        }),
                )
                // collecting the fields using the `with`
                .right_collector(|ix, f| {
                    let ident = f.as_ident(ix);
                    if is_try {
                        if let Some(try_with) = f.with_from_for(from_ty) {
                            quote!(#try_with(#ident)?)
                        } else {
                            quote!(TryInto::try_into(#ident)?)
                        }
                    } else if let Some(with) = f.with_from_for(from_ty) {
                        quote!(#with(#ident))
                    } else {
                        quote!(Into::into(#ident))
                    }
                })
                .collect();

            quote!( #into_ty::#ident #into_fields )
        })
        .collect();

    // If we're deriving a custom function
    if let Some(custom) = Override::as_ref(from).explicit().and_then(|e| e.custom.as_deref()) {
        // Collect the skipped fields that doesn't have a default value
        let external_fields = enum_variants
            .iter()
            .flat_map(|v| &v.fields.fields)
            .filter(|f| {
                f.skip_for(from_ty)
                    .filter(|map| map.as_ref().explicit().map(|s| s.default.is_none()).unwrap_or(true))
                    .is_some()
            })
            .filter_map(|f| {
                let ty = &f.ty;
                f.ident.as_ref().map(|i| {
                    let field_provider = format_ident!("{i}_provider");
                    quote!(#field_provider: impl FnOnce() -> #ty)
                })
            })
            .collect::<Vec<_>>();

        // Compute the function name, wether is provided or not
        let fn_name = custom.clone().explicit().unwrap_or_else(|| {
            format_ident!(
                "{}_{}",
                if is_try { "try_from" } else { "from" },
                from_ty.to_token_stream().to_string().to_snake_case()
            )
        });

        // Compute the method doc
        let doc = format!(
            "{} a new [{into_ty}] from a [{}]",
            if is_try { "Tries to build" } else { "Builds" },
            from_ty.to_token_stream().to_string().replace(' ', "")
        );

        // Implement the custom function
        if is_try {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #into_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    pub fn #fn_name(from: #from_ty #ty_generics, #( #external_fields ),*)
                        -> ::std::result::Result<Self, ::anyhow::Error> {
                        Ok(match from #match_body)
                    }
                }
            )
        } else {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #into_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    fn #fn_name(from: #from_ty #ty_generics, #( #external_fields ),*) -> Self {
                        match from #match_body
                    }
                }
            )
        }
    } else if is_try {
        // Implement the [TryFrom] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics TryFrom<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                type Error = ::anyhow::Error;

                fn try_from(other: #from_ty #ty_generics)
                    -> ::std::result::Result<Self, <Self as TryFrom<#from_ty #ty_generics>>::Error> {

                    Ok(match other #match_body)
                }
            }
        )
    } else {
        // Implement the [From] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics From<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                fn from(other: #from_ty #ty_generics) -> Self {
                    match other #match_body
                }
            }
        )
    }
}

fn derive_enum_into(
    into: &SpannedValue<Override<DeriveInput>>,
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    enum_variants: &[VariantReceiver],
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Derive from self into the other type
    let from_ty = ident;
    let into_ty = derive.path.as_ref();

    // Self type has
    let match_body = VariantsHelper::new(enum_variants)
        // every non-skipped variant 
        .filtering_variants(|v| v.skip_for(into_ty).is_none())
        // and skipped variants ignored
        .include_extra_variants(
            enum_variants
                .iter()
                .filter_map(|v| v.skip_for(into_ty).map(|skip| (v, skip)))
                .map(|(v, skip)| {
                    let variant = &v.ident;
                    (
                        quote!(#from_ty::#variant { .. }),
                        // populated with
                        Some(skip.as_ref()
                            .explicit()
                            .and_then(|s| s.default.as_deref())
                            // if default enabled: the default expression provided or Default::default()
                            .map(|d| d
                                .clone()
                                .explicit()
                                .map(|e| e.value)
                                .unwrap_or_else(|| parse_quote!(Default::default())))
                            // if default disabled error, as it must be enabled
                            .expect("'default' is required")
                        ),
                    )
                }),
        )
        // the left side of the match will be the from variant, along with its fields (if any)
        .left_collector(|v, fields| {
            let ident = &v.ident;
            // Self variant has
            let from_fields = fields
                // every non-skipped field
                .filtering(|_ix, f| f.skip_for(into_ty).is_none())
                // and skipped fields ignored
                .ignore_extra(
                    v.fields
                        .iter()
                        .filter(|f| f.skip_for(into_ty).is_some())
                        .filter_map(|f| f.ident.as_ref()),
                )
                // collecting as the field ident
                .right_collector(FieldsCollector::ident)
                .collect();

            quote!( #from_ty::#ident #from_fields )
        })
        // the right side of the match will be the into variant, along with its fields (if any)
        .right_collector(|v, fields| {
            // the other type variant name will be the same name or the rename ident
            let ident = if let Some(rename) = v.rename_for(into_ty) {
                rename
            } else {
                &v.ident
            };
            // the other type variant has
            let into_fields = fields
                // every non-skipped field
                .filtering(|_ix, f| f.skip_for(into_ty).is_none())
                // every additional field explicitly set
                .include_default_with(
                    v.additional_for(into_ty)
                        .map(|i| {
                            i.iter()
                                .map(|i| {
                                    let field = i.field.as_ref();
                                    (
                                        field,
                                        // populated with
                                        i.default
                                            .as_deref()
                                            // if default enabled: the default expression provided or Default::default()
                                            .map(|d| d
                                                .clone()
                                                .explicit()
                                                .map(|d| d.value)
                                                .unwrap_or_else(|| parse_quote!(Default::default()))
                                            )
                                            // or just the field ident, as it will be provided on the function parameters
                                            .unwrap_or_else(|| {
                                                let field_provider = format_ident!("{field}_provider");
                                                parse_quote!(#field_provider())
                                            }),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                )
                // where we collect each field ident (or the rename)
                .left_collector(|ix, f| {
                    let ident = if let Some(rename) = f.rename_for(into_ty) {
                        rename.clone()
                    } else {
                        f.as_ident(ix)
                    };
                    quote!(#ident)
                })
                // using the `with`
                .right_collector(|ix, f| {
                    let ident = f.as_ident(ix);
                    if is_try {
                        if let Some(try_with) = f.with_into_for(into_ty) {
                            quote!(#try_with(#ident)?)
                        } else {
                            quote!(TryInto::try_into(#ident)?)
                        }
                    } else if let Some(with) = f.with_into_for(into_ty) {
                        quote!(#with(#ident))
                    } else {
                        quote!(Into::into(#ident))
                    }
                })
                .collect();

            quote!( #into_ty::#ident #into_fields )
        })
        .collect();

    // If we're deriving a custom function
    if let Some(custom) = Override::as_ref(into).explicit().and_then(|e| e.custom.as_deref()) {
        // Collect the additional fields that doesn't have a default value
        let external_fields = enum_variants
            .iter()
            .filter_map(|v| v.additional_for(into_ty))
            .flatten()
            .filter(|a| a.default.is_none())
            .map(|f| {
                let ident = f.field.as_ref();
                let field_provider = format_ident!("{ident}_provider");
                let ty = f.ty.as_ref().expect("'ty' must be provided").as_ref();
                quote!(#field_provider: impl FnOnce() -> #ty)
            })
            .collect::<Vec<_>>();

        // Compute the function name, wether is provided or not
        let fn_name = custom.clone().explicit().unwrap_or_else(|| {
            format_ident!(
                "{}_{}",
                if is_try { "try_into" } else { "into" },
                into_ty.to_token_stream().to_string().to_snake_case()
            )
        });

        // Compute the method doc
        let doc = format!(
            "{} a new [{}] from a [{from_ty}]",
            if is_try { "Tries to build" } else { "Builds" },
            into_ty.to_token_stream().to_string().replace(' ', "")
        );

        // Implement the custom function
        if is_try {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #from_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    pub fn #fn_name(self, #( #external_fields ),*)
                        -> ::std::result::Result<#into_ty #ty_generics, ::anyhow::Error> {
                        Ok(match self #match_body)
                    }
                }
            )
        } else {
            quote!(
                #[automatically_derived]
                #[allow(non_shorthand_field_patterns)]
                impl #impl_generics #from_ty #ty_generics #where_clause {
                    #[doc = #doc]
                    #[allow(clippy::too_many_arguments)]
                    fn #fn_name(self, #( #external_fields ),*) -> #into_ty #ty_generics {
                        match self #match_body
                    }
                }
            )
        }
    } else if is_try {
        // Implement the [TryFrom] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics TryFrom<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                type Error = ::anyhow::Error;

                fn try_from(other: #from_ty #ty_generics)
                    -> ::std::result::Result<Self, <Self as TryFrom<#from_ty #ty_generics>>::Error> {

                    Ok(match other #match_body)
                }
            }
        )
    } else {
        // Implement the [From] trait
        quote!(
            #[automatically_derived]
            #[allow(non_shorthand_field_patterns)]
            impl #impl_generics From<#from_ty #ty_generics> for #into_ty #ty_generics #where_clause {
                fn from(other: #from_ty #ty_generics) -> Self {
                    match other #match_body
                }
            }
        )
    }
}
