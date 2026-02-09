#![allow(clippy::manual_unwrap_or_default)] // darling macros

use darling::{
    ast::{Data, Fields},
    util::{Flag, Override, SpannedValue},
    FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error2::{abort_call_site, emit_error};
use quote::quote;
use syn::spanned::Spanned;

use crate::type_path_ext::TypePathWrapper;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(mapper), supports(struct_named, struct_newtype, struct_tuple, enum_any))]
pub(super) struct MapperOpts {
    /// The identifier of the passed-in type
    pub(super) ident: syn::Ident,
    /// The generics of the passed-in type
    pub(super) generics: syn::Generics,
    /// The body of the passed-in type
    pub(super) data: Data<VariantReceiver, FieldReceiver>,

    #[darling(default, multiple, rename = "derive")]
    items: Vec<ItemInput>,

    /// Path of the struct or enum
    #[darling(default, rename = "ty")]
    path: Option<SpannedValue<TypePathWrapper>>,
    /// Whether to derive [From] the type to self
    #[darling(default)]
    from: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether to derive [From] self to the type
    #[darling(default)]
    into: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether to derive [TryFrom] the type to self
    #[darling(default)]
    try_from: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether to derive [TryFrom] self to the type
    #[darling(default)]
    try_into: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether the other type have some additional variants/fields
    #[darling(default, multiple)]
    add: Vec<AddInput>,
    /// Whether to ignore all extra fields/variants of the other type
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct ItemInput {
    /// Path of the struct or enum
    #[darling(rename = "ty")]
    pub(super) path: SpannedValue<TypePathWrapper>,
    /// Whether to derive [From] the type to self
    #[darling(default)]
    pub(super) from: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether to derive [From] self to the type
    #[darling(default)]
    pub(super) into: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether to derive [TryFrom] the type to self
    #[darling(default)]
    pub(super) try_from: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether to derive [TryFrom] self to the type
    #[darling(default)]
    pub(super) try_into: Option<SpannedValue<Override<DeriveInput>>>,
    /// Whether the other type have some additional variants/fields
    #[darling(default, multiple)]
    pub(super) add: Vec<AddInput>,
    /// Whether to ignore all extra fields/variants of the other type
    #[darling(default)]
    pub(super) ignore_extra: SpannedValue<Flag>,
}

#[derive(Debug, FromVariant, Clone)]
#[darling(attributes(mapper))]
pub(super) struct VariantReceiver {
    /// The identifier of the passed-in variant
    pub(super) ident: syn::Ident,
    /// For a variant such as `Example = 2`, the `2`
    pub(super) discriminant: Option<syn::Expr>,
    /// The fields associated with the variant
    pub(super) fields: Fields<FieldReceiver>,

    #[darling(default, multiple, rename = "when")]
    items: Vec<ItemVariantInput>,

    /// Path of the struct or enum to derive
    #[darling(default, rename = "ty")]
    path: Option<SpannedValue<TypePathWrapper>>,
    /// To rename the variant
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// Whether the other type have some additional fields on the variant
    #[darling(default, multiple)]
    add: Vec<AddInput>,
    /// To skip this variant when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// Whether to ignore all extra fields of the other variant
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
}
macro_field_utils::variant_info!(VariantReceiver, FieldReceiver);

#[derive(Debug, FromMeta, Clone)]
struct ItemVariantInput {
    /// Path of the struct or enum to derive
    #[darling(rename = "ty")]
    path: SpannedValue<TypePathWrapper>,
    /// To rename the variant
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// Whether the other type have some additional fields on the variant
    #[darling(default, multiple)]
    add: Vec<AddInput>,
    /// To skip this variant when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// Whether to ignore all extra fields of the other variant
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
}

#[derive(Debug, FromField, Clone)]
#[darling(attributes(mapper))]
pub(super) struct FieldReceiver {
    /// The identifier of the passed-in field, or [None] for tuple fields
    pub(super) ident: Option<syn::Ident>,
    /// The visibility of the passed-in field
    pub(super) vis: syn::Visibility,
    /// The type of the passed-in field
    pub(super) ty: syn::Type,

    #[darling(default, multiple, rename = "when")]
    items: Vec<ItemFieldInput>,

    /// Path of the struct or enum to derive
    #[darling(default, rename = "ty")]
    path: Option<SpannedValue<TypePathWrapper>>,
    /// To rename the field
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// To skip this field when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// To use another source generic type for mapping
    #[darling(default)]
    other_ty: Option<SpannedValue<syn::Ident>>,
    /// Mapper hints
    #[darling(flatten)]
    hint: MapperHint,
}
macro_field_utils::field_info!(FieldReceiver);

#[derive(Debug, FromMeta, Clone)]
struct ItemFieldInput {
    /// Path of the struct or enum to derive
    #[darling(rename = "ty")]
    path: SpannedValue<TypePathWrapper>,
    /// To rename the field
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// To skip this field when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// To use another source generic type for mapping
    #[darling(default)]
    other_ty: Option<SpannedValue<syn::Ident>>,
    /// Mapper hints
    #[darling(flatten)]
    hint: MapperHint,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct MapperHint {
    /// To use some function to map the values
    #[darling(default)]
    with: Option<SpannedValue<syn::Expr>>,
    /// To use some function to map the values
    #[darling(default)]
    into_with: Option<SpannedValue<syn::Expr>>,
    /// To use some function to map the values
    #[darling(default)]
    from_with: Option<SpannedValue<syn::Expr>>,
    /// Whether the field is an option
    #[darling(default)]
    opt: Option<SpannedValue<Override<Box<MapperHint>>>>,
    /// Wether the field is an iterator
    #[darling(default)]
    iter: Option<SpannedValue<Override<Box<MapperHint>>>>,
    /// Wether the field is a HashMap-like iter
    #[darling(default)]
    map: Option<SpannedValue<Override<Box<MapperHint>>>>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct DeriveInput {
    /// Whether the derive has external properties or not (name of the custom function if populated)
    #[darling(default)]
    pub(super) custom: Option<SpannedValue<Override<syn::Ident>>>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct AddInput {
    /// Name of the ignored field or variant
    pub(super) field: SpannedValue<syn::Ident>,
    /// Type of the field
    #[darling(default)]
    pub(super) ty: Option<SpannedValue<TypePathWrapper>>,
    /// Default value for the field
    #[darling(default)]
    pub(super) default: Option<SpannedValue<Override<DefaultInput>>>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct DefaultInput {
    // The default value expression
    pub(crate) value: syn::Expr,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct SkipInput {
    /// Default value for the field
    #[darling(default)]
    pub(super) default: Option<SpannedValue<Override<DefaultInput>>>,
}

impl MapperOpts {
    /// Retrieve the [ItemInput] of the [MapperOpts]
    pub(super) fn items(&self) -> Vec<ItemInput> {
        if !self.items.is_empty() {
            // If there are multiple derives, the root-level fields are not allowed
            if let Some(path) = self.path.as_ref() {
                emit_error!(path.span(), "Illegal attribute when 'derive' is set")
            }
            if let Some(from) = self.from.as_ref() {
                emit_error!(from.span(), "Illegal attribute when 'derive' is set")
            }
            if let Some(into) = self.into.as_ref() {
                emit_error!(into.span(), "Illegal attribute when 'derive' is set")
            }
            if let Some(try_from) = self.try_from.as_ref() {
                emit_error!(try_from.span(), "Illegal attribute when 'derive' is set")
            }
            if let Some(try_into) = self.try_into.as_ref() {
                emit_error!(try_into.span(), "Illegal attribute when 'derive' is set")
            }
            if self.ignore_extra.is_present() {
                emit_error!(self.ignore_extra.span(), "Illegal attribute when 'derive' is set")
            }
            if !self.add.is_empty() {
                for i in &self.add {
                    emit_error!(i.field.span(), "Illegal attribute when 'derive' is set")
                }
            }
            // Verify there same type is not duplicated
            let paths = self.items.iter().map(|i| &i.path).collect::<Vec<_>>();
            for i in 0..paths.len() {
                for j in 0..paths.len() {
                    if i != j && paths[i].as_ref() == paths[j].as_ref() {
                        emit_error!(paths[j].span(), "This type is duplicated")
                    }
                }
            }
            // Return all of the items
            self.items.to_vec()
        } else if let Some(path) = self.path.as_ref() {
            // If there are a single derive, wrap it on a vec
            vec![ItemInput {
                path: path.clone(),
                from: self.from.clone(),
                into: self.into.clone(),
                try_from: self.try_from.clone(),
                try_into: self.try_into.clone(),
                ignore_extra: self.ignore_extra,
                add: self.add.clone(),
            }]
        } else {
            // If there are no derives, abort
            abort_call_site!("One of 'ty' or 'derive' must be set")
        }
    }
}

impl ItemInput {
    /// Validates the input is well formed, emitting errors if not
    pub(super) fn validate(&self, is_enum: bool) {
        // At least one kind of derive must be set
        if self.from.is_none() && self.into.is_none() && self.try_from.is_none() && self.try_into.is_none() {
            emit_error!(
                self.path.span(),
                "One of 'from', 'into', 'try_from' or 'try_into' must be set"
            );
        }
        // If there are additional items without a default value (for structs only)
        let items = self.add.iter().filter(|a| a.default.is_none()).collect::<Vec<_>>();
        if !items.is_empty() && !is_enum {
            // into and try_into must have the custom flag if enabled and ty must be set
            let mut has_into = false;
            let mut missing_custom = false;
            if let Some(into) = self.into.as_deref() {
                has_into = true;
                if into.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true) {
                    missing_custom = true;
                }
            }
            if let Some(try_into) = self.try_into.as_deref() {
                has_into = true;
                if try_into.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true) {
                    missing_custom = true;
                }
            }
            if missing_custom || has_into {
                for e in items {
                    if missing_custom {
                        emit_error!(
                            e.field.span(),
                            "Enable `default` here or include `custom` on `into` and `try_into` derives"
                        );
                    }
                    if e.ty.is_none() && has_into {
                        emit_error!(
                            e.field.span(),
                            "Provide a field type with `ty` if the field is not `default` to derive `into` and \
                             `try_into`"
                        );
                    }
                }
            }
        }
        // Verify additional variants for enums
        if is_enum {
            for a in &self.add {
                if let Some(ty) = &a.ty {
                    emit_error!(ty.span(), "Illegal attribute for enums")
                }
                if a.default.is_none() && (self.from.is_some() || self.try_from.is_some()) {
                    emit_error!(
                        a.field.span(),
                        "Missing mandatory `default` for enums when deriving `from` or `try_from`"
                    )
                }
            }
        }
    }
}

impl ItemVariantInput {
    fn validate(&self, derives: &[ItemInput]) {
        let derive = derives.iter().find(|d| d.path.as_ref() == self.path.as_ref());
        if let Some(derive) = derive {
            // If there are additional items without a default value
            let items = self.add.iter().filter(|a| a.default.is_none()).collect::<Vec<_>>();
            if !items.is_empty() {
                // into and try_into must have the custom flag if enabled and ty must be set
                let mut has_into = false;
                let mut missing_custom = false;
                if let Some(into) = derive.into.as_deref() {
                    has_into = true;
                    if into.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true) {
                        missing_custom = true;
                    }
                }
                if let Some(try_into) = derive.try_into.as_deref() {
                    has_into = true;
                    if try_into.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true) {
                        missing_custom = true;
                    }
                }
                if missing_custom || has_into {
                    for e in items {
                        if missing_custom {
                            emit_error!(
                                e.field.span(),
                                "Enable `default` here or include `custom` on `into` and `try_into` derives"
                            );
                        }
                        if e.ty.is_none() && has_into {
                            emit_error!(
                                e.field.span(),
                                "Provide a field type with `ty` if the field is not `default` to derive `into` and \
                                 `try_into`"
                            );
                        }
                    }
                }
            }
            // If there are skipped variants without a default value
            if let Some(skip) = self.skip.as_ref()
                && Override::as_ref(skip)
                    .explicit()
                    .map(|e| e.default.is_none())
                    .unwrap_or(true)
            {
                // into and try_into requires a default value
                if derive.into.is_some() || derive.try_into.is_some() {
                    emit_error!(
                        skip.span(),
                        "Enable `default` here required for `into` and `try_into` derives"
                    );
                }
            }
        } else {
            emit_error!(self.path.span(), "There is no derive defined for this type");
        }
    }
}

impl VariantReceiver {
    pub(super) fn validate(&self, derives: &[ItemInput]) {
        if !self.items.is_empty() {
            // If there are multiple derives, the root-level fields are not allowed
            if let Some(path) = self.path.as_ref() {
                emit_error!(path.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(rename) = self.rename.as_ref() {
                emit_error!(rename.span(), "Illegal attribute if 'when' is set")
            }
            if !self.add.is_empty() {
                for i in &self.add {
                    emit_error!(i.field.span(), "Illegal attribute if 'when' is set")
                }
            }
            if let Some(skip) = self.skip.as_ref() {
                emit_error!(skip.span(), "Illegal attribute if 'when' is set")
            }
            if self.ignore_extra.is_present() {
                emit_error!(self.ignore_extra.span(), "Illegal attribute if 'when' is set")
            }
            // Verify there same type is not duplicated
            let paths = self.items.iter().map(|i| &i.path).collect::<Vec<_>>();
            for i in 0..paths.len() {
                for j in 0..paths.len() {
                    if i != j && paths[i].as_ref() == paths[j].as_ref() {
                        emit_error!(paths[j].span(), "This type is duplicated")
                    }
                }
            }
        }

        if let Some(path) = &self.path {
            ItemVariantInput {
                path: path.clone(),
                rename: self.rename.clone(),
                add: self.add.to_vec(),
                skip: self.skip.clone(),
                ignore_extra: self.ignore_extra,
            }
            .validate(derives);
        } else {
            for d in derives {
                ItemVariantInput {
                    path: d.path.clone(),
                    rename: self.rename.clone(),
                    add: self.add.to_vec(),
                    skip: self.skip.clone(),
                    ignore_extra: self.ignore_extra,
                }
                .validate(derives);
            }
        }
        for item in self.items.iter() {
            item.validate(derives);
        }

        self.fields.iter().for_each(|f| f.validate(derives));
    }

    pub(super) fn rename_for(&self, derive_path: &syn::TypePath) -> Option<&syn::Ident> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.rename.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.rename.as_deref()
            } else {
                None
            }
        } else {
            self.rename.as_deref()
        }
    }

    pub(super) fn additional_for(&self, derive_path: &syn::TypePath) -> Option<&Vec<AddInput>> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                if item.add.is_empty() {
                    return None;
                } else {
                    return Some(item.add.as_ref());
                }
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                if self.add.is_empty() {
                    None
                } else {
                    Some(self.add.as_ref())
                }
            } else {
                None
            }
        } else if self.add.is_empty() {
            None
        } else {
            Some(self.add.as_ref())
        }
    }

    pub(super) fn skip_for(&self, derive_path: &syn::TypePath) -> Option<&Override<SkipInput>> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.skip.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.skip.as_deref()
            } else {
                None
            }
        } else {
            self.skip.as_deref()
        }
    }

    pub(super) fn ignore_extra_for(&self, derive_path: &syn::TypePath) -> bool {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.ignore_extra.is_present();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.ignore_extra.is_present()
            } else {
                false
            }
        } else {
            self.ignore_extra.is_present()
        }
    }
}

impl ItemFieldInput {
    fn validate(&self, span: Span, derives: &[ItemInput]) {
        let derive = derives.iter().find(|d| d.path.as_ref() == self.path.as_ref());
        if let Some(derive) = derive {
            // If there are skipped fields without a default value
            if let Some(skip) = self.skip.as_ref()
                && Override::as_ref(skip)
                    .explicit()
                    .map(|e| e.default.is_none())
                    .unwrap_or(true)
            {
                // from and try_from must have the custom flag if enabled
                let mut missing_custom = false;
                if let Some(from) = derive.from.as_deref()
                    && from.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true)
                {
                    missing_custom = true;
                }
                if let Some(try_from) = derive.try_from.as_deref()
                    && try_from.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true)
                {
                    missing_custom = true;
                }
                if missing_custom {
                    emit_error!(
                        skip.span(),
                        "Enable `default` here or include `custom` on `from` and `try_from` derives"
                    );
                }
            }
            // validate only one hint
            let mut hint_count = 0;
            if self.hint.with.is_some() {
                hint_count += 1;
            }
            if self.hint.from_with.is_some() || self.hint.into_with.is_some() {
                hint_count += 1;
            }
            if self.hint.opt.is_some() {
                hint_count += 1;
            }
            if self.hint.iter.is_some() {
                hint_count += 1;
            }
            if self.hint.map.is_some() {
                hint_count += 1;
            }
            if hint_count > 1 {
                emit_error!(
                    span,
                    "Only one of 'with', 'into_with'/'from_with', 'opt', 'iter' or 'map' can be set"
                );
            }
        } else {
            emit_error!(self.path.span(), "There is no derive defined for this type");
        }
    }
}
impl FieldReceiver {
    pub(super) fn validate(&self, derives: &[ItemInput]) {
        if !self.items.is_empty() {
            // If there are multiple derives, the root-level fields are not allowed
            if let Some(path) = self.path.as_ref() {
                emit_error!(path.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(rename) = self.rename.as_ref() {
                emit_error!(rename.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(skip) = self.skip.as_ref() {
                emit_error!(skip.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(with) = self.hint.with.as_ref() {
                emit_error!(with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(into_with) = self.hint.into_with.as_ref() {
                emit_error!(into_with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(from_with) = self.hint.from_with.as_ref() {
                emit_error!(from_with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(opt) = self.hint.opt.as_ref() {
                emit_error!(opt.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(iter) = self.hint.iter.as_ref() {
                emit_error!(iter.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(map) = self.hint.map.as_ref() {
                emit_error!(map.span(), "Illegal attribute if 'when' is set")
            }
            // Verify there same type is not duplicated
            let paths = self.items.iter().map(|i| &i.path).collect::<Vec<_>>();
            for i in 0..paths.len() {
                for j in 0..paths.len() {
                    if i != j && paths[i].as_ref() == paths[j].as_ref() {
                        emit_error!(paths[j].span(), "This type is duplicated")
                    }
                }
            }
        }
        let span = self.ident.as_ref().map(|i| i.span()).unwrap_or_else(|| self.ty.span());
        if let Some(path) = &self.path {
            ItemFieldInput {
                path: path.clone(),
                rename: self.rename.clone(),
                skip: self.skip.clone(),
                other_ty: self.other_ty.clone(),
                hint: self.hint.clone(),
            }
            .validate(span, derives);
        } else {
            for d in derives {
                ItemFieldInput {
                    path: d.path.clone(),
                    rename: self.rename.clone(),
                    skip: self.skip.clone(),
                    other_ty: self.other_ty.clone(),
                    hint: self.hint.clone(),
                }
                .validate(span, derives);
            }
        }
        for item in self.items.iter() {
            item.validate(span, derives);
        }
    }

    pub(super) fn rename_for(&self, derive_path: &syn::TypePath) -> Option<&syn::Ident> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.rename.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.rename.as_deref()
            } else {
                None
            }
        } else {
            self.rename.as_deref()
        }
    }

    pub(super) fn skip_for(&self, derive_path: &syn::TypePath) -> Option<&Override<SkipInput>> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.skip.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.skip.as_deref()
            } else {
                None
            }
        } else {
            self.skip.as_deref()
        }
    }

    pub(super) fn other_ty_for(&self, derive_path: &syn::TypePath) -> Option<&syn::Ident> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.other_ty.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.other_ty.as_deref()
            } else {
                None
            }
        } else {
            self.other_ty.as_deref()
        }
    }

    fn hint_for(&self, derive_path: &syn::TypePath) -> Option<&MapperHint> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return Some(&item.hint);
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                Some(&self.hint)
            } else {
                None
            }
        } else {
            Some(&self.hint)
        }
    }

    pub(super) fn build_into_for(
        &self,
        from: bool,
        is_try: bool,
        ident: &syn::Ident,
        derive_path: &syn::TypePath,
    ) -> TokenStream {
        let into = build_into_for_inner(from, is_try, ident, self.hint_for(derive_path));
        if is_try {
            quote!(#into?)
        } else {
            into
        }
    }
}

fn build_into_for_inner(from: bool, is_try: bool, ident: &syn::Ident, hint: Option<&MapperHint>) -> TokenStream {
    if let Some(hint) = hint {
        let check_with = |with: &Option<SpannedValue<syn::Expr>>| {
            if let Some(with) = with {
                let with = with.as_ref();
                if let syn::Expr::Path(with_path) = with {
                    let crate_name = match crate_name("model-mapper") {
                        Ok(FoundCrate::Itself) => quote!(::model_mapper),
                        Ok(FoundCrate::Name(name)) => {
                            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                            quote!(::#ident)
                        }
                        Err(_) => quote!(::model_mapper),
                    };
                    Some(quote!({
                        use #crate_name::private::{RefMapper, ValueMapper};
                        (&(#with_path)).map_value(#ident)
                    }))
                } else if is_try {
                    Some(quote!(Ok::<_, anyhow::Error>(#with)))
                } else {
                    Some(quote!(#with))
                }
            } else {
                None
            }
        };

        if from {
            if let Some(t) = check_with(&hint.from_with) {
                return t;
            }
        } else if let Some(t) = check_with(&hint.into_with) {
            return t;
        }

        if let Some(t) = check_with(&hint.with) {
            return t;
        } else if let Some(opt) = &hint.opt {
            let inner;
            if let Some(inner_hint) = opt.as_ref().as_ref().explicit() {
                inner = build_into_for_inner(from, is_try, ident, Some(inner_hint));
            } else {
                inner = build_into_for_inner(from, is_try, ident, None);
            }
            if is_try {
                return quote!(#ident.map(|#ident| #inner).transpose());
            } else {
                return quote!(#ident.map(|#ident| #inner));
            }
        } else if let Some(iter) = &hint.iter {
            let inner;
            if let Some(inner_hint) = iter.as_ref().as_ref().explicit() {
                inner = build_into_for_inner(from, is_try, ident, Some(inner_hint));
            } else {
                inner = build_into_for_inner(from, is_try, ident, None);
            }
            if is_try {
                return quote!(#ident.into_iter().map(|#ident| #inner).collect::<std::result::Result<_, _>>());
            } else {
                return quote!(#ident.into_iter().map(|#ident| #inner).collect());
            }
        } else if let Some(map) = &hint.map {
            let inner;
            if let Some(inner_hint) = map.as_ref().as_ref().explicit() {
                inner = build_into_for_inner(from, is_try, ident, Some(inner_hint));
            } else {
                inner = build_into_for_inner(from, is_try, ident, None);
            }
            if is_try {
                return quote!(
                    #ident.into_iter().map(|(k, #ident)| #inner.map(|v| (k, v))).collect::<std::result::Result<_, _>>()
                );
            } else {
                return quote!(#ident.into_iter().map(|(k, #ident)| (k, #inner)).collect());
            }
        }
    }
    if is_try {
        quote!(TryInto::try_into(#ident))
    } else {
        quote!(Into::into(#ident))
    }
}
