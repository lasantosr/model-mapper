use darling::{
    ast::{Data, Fields},
    FromDeriveInput,
};
use macro_field_utils::{FieldInfo, FieldsCollector, FieldsHelper, VariantsHelper};
use proc_macro2::TokenStream;
use proc_macro_error::abort_if_dirty;
use quote::{quote, ToTokens};
use syn::DeriveInput;

use crate::input::*;

pub(crate) fn r#impl(input: DeriveInput) -> TokenStream {
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
    derive_items.iter().for_each(ItemInput::validate);
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
    if derive.from.is_present() {
        derive_struct_from(ident, generics, &derive, struct_fields, false).to_tokens(&mut output);
    }

    // Derive reverse `From`
    if derive.into.is_present() {
        derive_struct_into(ident, generics, &derive, struct_fields, false).to_tokens(&mut output);
    }

    // Derive `TryFrom`
    if derive.try_from.is_present() {
        derive_struct_from(ident, generics, &derive, struct_fields, true).to_tokens(&mut output);
    }

    // Derive reverse `TryFrom`
    if derive.try_into.is_present() {
        derive_struct_into(ident, generics, &derive, struct_fields, true).to_tokens(&mut output);
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
    if derive.from.is_present() {
        derive_enum_from(ident, generics, &derive, enum_variants, false).to_tokens(&mut output);
    }

    // Derive reverse `From`
    if derive.into.is_present() {
        derive_enum_into(ident, generics, &derive, enum_variants, false).to_tokens(&mut output);
    }

    // Derive `TryFrom`
    if derive.try_from.is_present() {
        derive_enum_from(ident, generics, &derive, enum_variants, true).to_tokens(&mut output);
    }

    // Derive reverse `TryFrom`
    if derive.try_into.is_present() {
        derive_enum_into(ident, generics, &derive, enum_variants, true).to_tokens(&mut output);
    }

    output
}

fn derive_struct_from(
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    struct_fields: &Fields<FieldReceiver>,
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_ty = derive.path.as_ref();
    let into_ty = ident;
    let from_ty_fields_helper = FieldsHelper::new(struct_fields)
        .filtering(|_ix, f| !f.is_skip_for(from_ty))
        .ignore_extra(derive.ignore.iter().map(|f| f.as_ref()));
    let into_ty_fields_helper = FieldsHelper::new(struct_fields)
        .filtering(|_ix, f| !f.is_skip_for(from_ty))
        .include_default(
            struct_fields
                .iter()
                .filter(|f| f.is_skip_for(from_ty))
                .filter_map(|f| f.ident.as_ref()),
        );

    // Deconstruct the `from` input
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

    // Produce `into` body
    let into_body = into_ty_fields_helper
        .right_collector(|ix, f| {
            let ident = f.as_ident(ix);
            if is_try {
                if let Some(try_with) = f.try_with_for(from_ty) {
                    quote!(#try_with(#ident)?)
                } else {
                    quote!(TryInto::try_into(#ident)?)
                }
            } else if let Some(with) = f.with_for(from_ty) {
                quote!(#with(#ident))
            } else {
                quote!(Into::into(#ident))
            }
        })
        .collect();

    // Derive
    if is_try {
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

fn derive_struct_into(
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    struct_fields: &Fields<FieldReceiver>,
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_ty = ident;
    let into_ty = derive.path.as_ref();
    let from_ty_fields_helper = FieldsHelper::new(struct_fields)
        .filtering(|_ix, f| !f.is_skip_for(into_ty))
        .ignore_extra(
            struct_fields
                .iter()
                .filter(|f| f.is_skip_for(into_ty))
                .filter_map(|f| f.ident.as_ref()),
        );
    let into_ty_fields_helper = FieldsHelper::new(struct_fields)
        .filtering(|_ix, f| !f.is_skip_for(into_ty))
        .include_default(derive.ignore.iter().map(|f| f.as_ref()));

    // Deconstruct the `from` input
    let deconstructed_from = from_ty_fields_helper.right_collector(FieldsCollector::ident).collect();

    // Produce `into` body
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
                if let Some(try_with) = f.try_with_for(into_ty) {
                    quote!(#try_with(#ident)?)
                } else {
                    quote!(TryInto::try_into(#ident)?)
                }
            } else if let Some(with) = f.with_for(into_ty) {
                quote!(#with(#ident))
            } else {
                quote!(Into::into(#ident))
            }
        })
        .collect();

    // Derive
    if is_try {
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
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    enum_variants: &[VariantReceiver],
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_ty = derive.path.as_ref();
    let into_ty = ident;

    let match_body = VariantsHelper::new(enum_variants)
        .filtering_variants(|v| !v.is_skip_for(from_ty))
        .left_collector(|v, fields| {
            let ident = if let Some(rename) = v.rename_for(from_ty) {
                rename
            } else {
                &v.ident
            };
            let ignore_extra = v
                .ignore_for(from_ty)
                .map(|i| i.iter().map(|i| i.as_ref()).collect::<Vec<_>>())
                .unwrap_or_default();
            let left = fields
                .filtering(|_ix, f| !f.is_skip_for(from_ty))
                .ignore_extra(ignore_extra)
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
            quote!( #from_ty::#ident #left )
        })
        .right_collector(|v, fields| {
            let ident = &v.ident;
            let right = fields
                .filtering(|_ix, f| !f.is_skip_for(from_ty))
                .include_default(
                    v.fields
                        .iter()
                        .filter(|f| f.is_skip_for(from_ty))
                        .filter_map(|f| f.ident.as_ref()),
                )
                .right_collector(|ix, f| {
                    let ident = f.as_ident(ix);
                    if is_try {
                        if let Some(try_with) = f.try_with_for(from_ty) {
                            quote!(#try_with(#ident)?)
                        } else {
                            quote!(TryInto::try_into(#ident)?)
                        }
                    } else if let Some(with) = f.with_for(from_ty) {
                        quote!(#with(#ident))
                    } else {
                        quote!(Into::into(#ident))
                    }
                })
                .collect();
            quote!( #into_ty::#ident #right )
        })
        .include_extra_variants(derive.ignore.iter().map(|i| {
            let field = i.as_ref();
            (quote!(#from_ty::#field { .. }), Some(quote!(Default::default())))
        }))
        .collect();

    // Derive
    if is_try {
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
    ident: &syn::Ident,
    generics: &syn::Generics,
    derive: &ItemInput,
    enum_variants: &[VariantReceiver],
    is_try: bool,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_ty = ident;
    let into_ty = derive.path.as_ref();

    let match_body = VariantsHelper::new(enum_variants)
        .filtering_variants(|v| !v.is_skip_for(into_ty))
        .left_collector(|v, fields| {
            let ident = &v.ident;
            let left = fields
                .filtering(|_ix, f| !f.is_skip_for(into_ty))
                .ignore_extra(
                    v.fields
                        .iter()
                        .filter(|f| f.is_skip_for(into_ty))
                        .filter_map(|f| f.ident.as_ref()),
                )
                .right_collector(FieldsCollector::ident)
                .collect();
            quote!( #from_ty::#ident #left )
        })
        .right_collector(|v, fields| {
            let ident = if let Some(rename) = v.rename_for(into_ty) {
                rename
            } else {
                &v.ident
            };
            let right = fields
                .filtering(|_ix, f| !f.is_skip_for(into_ty))
                .include_default(
                    v.ignore_for(into_ty)
                        .map(|i| i.iter().map(|f| f.as_ref()).collect::<Vec<_>>())
                        .unwrap_or_default(),
                )
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
                        if let Some(try_with) = f.try_with_for(into_ty) {
                            quote!(#try_with(#ident)?)
                        } else {
                            quote!(TryInto::try_into(#ident)?)
                        }
                    } else if let Some(with) = f.with_for(into_ty) {
                        quote!(#with(#ident))
                    } else {
                        quote!(Into::into(#ident))
                    }
                })
                .collect();
            quote!( #into_ty::#ident #right )
        })
        .include_extra_variants(enum_variants.iter().filter(|v| v.is_skip_for(into_ty)).map(|v| {
            let variant = &v.ident;
            (quote!(#from_ty::#variant { .. }), Some(quote!(Default::default())))
        }))
        .collect();

    // Derive
    if is_try {
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
