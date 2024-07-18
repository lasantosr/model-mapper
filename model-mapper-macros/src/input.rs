#![allow(clippy::manual_unwrap_or_default)] // darling macros

use darling::{
    ast::{Data, Fields},
    util::{Flag, Override, SpannedValue},
    FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro_error::{abort_call_site, emit_error};

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
    path: Option<SpannedValue<syn::Path>>,
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
    /// Wether the other type have some additional variants/fields
    #[darling(default, multiple)]
    add: Vec<AddInput>,
    /// Wether to ignore all extra fields/variants of the other type
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct ItemInput {
    /// Path of the struct or enum
    #[darling(rename = "ty")]
    pub(super) path: SpannedValue<syn::Path>,
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
    /// Wether the other type have some additional variants/fields
    #[darling(default, multiple)]
    pub(super) add: Vec<AddInput>,
    /// Wether to ignore all extra fields/variants of the other type
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
    path: Option<SpannedValue<syn::Path>>,
    /// To rename the variant
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// Wether the other type have some additional fields on the variant
    #[darling(default, multiple)]
    add: Vec<AddInput>,
    /// To skip this variant when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// Wether to ignore all extra fields of the other variant
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
}
macro_field_utils::variant_info!(VariantReceiver, FieldReceiver);

#[derive(Debug, FromMeta, Clone)]
struct ItemVariantInput {
    /// Path of the struct or enum to derive
    #[darling(rename = "ty")]
    path: SpannedValue<syn::Path>,
    /// To rename the variant
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// Wether the other type have some additional fields on the variant
    #[darling(default, multiple)]
    add: Vec<AddInput>,
    /// To skip this variant when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// Wether to ignore all extra fields of the other variant
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
    path: Option<SpannedValue<syn::Path>>,
    /// To rename the field
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// To skip this field when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// To use some function to map the values
    #[darling(default)]
    with: Option<SpannedValue<syn::Path>>,
    /// To use some function to map the values
    #[darling(default)]
    into_with: Option<SpannedValue<syn::Path>>,
    /// To use some function to map the values
    #[darling(default)]
    from_with: Option<SpannedValue<syn::Path>>,
}
macro_field_utils::field_info!(FieldReceiver);

#[derive(Debug, FromMeta, Clone)]
struct ItemFieldInput {
    /// Path of the struct or enum to derive
    #[darling(rename = "ty")]
    path: SpannedValue<syn::Path>,
    /// To rename the field
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
    /// To skip this field when deriving (defaults to `false`)
    #[darling(default)]
    skip: Option<SpannedValue<Override<SkipInput>>>,
    /// To use some function to map the values
    #[darling(default)]
    with: Option<SpannedValue<syn::Path>>,
    /// To use some function to map the values
    #[darling(default)]
    into_with: Option<SpannedValue<syn::Path>>,
    /// To use some function to map the values
    #[darling(default)]
    from_with: Option<SpannedValue<syn::Path>>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct DeriveInput {
    /// Wether the derive has external properties or not (name of the custom function if populated)
    #[darling(default)]
    pub(super) custom: Option<SpannedValue<Override<syn::Ident>>>,
}

#[derive(Debug, FromMeta, Clone)]
pub(super) struct AddInput {
    /// Name of the ignored field or variant
    pub(super) field: SpannedValue<syn::Ident>,
    /// Type of the field
    #[darling(default)]
    pub(super) ty: Option<SpannedValue<syn::Path>>,
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
            if let Some(skip) = self.skip.as_ref() {
                if Override::as_ref(skip)
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

    pub(super) fn rename_for(&self, derive_path: &syn::Path) -> Option<&syn::Ident> {
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

    pub(super) fn additional_for(&self, derive_path: &syn::Path) -> Option<&Vec<AddInput>> {
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

    pub(super) fn skip_for(&self, derive_path: &syn::Path) -> Option<&Override<SkipInput>> {
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

    pub(super) fn ignore_extra_for(&self, derive_path: &syn::Path) -> bool {
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
    fn validate(&self, derives: &[ItemInput]) {
        let derive = derives.iter().find(|d| d.path.as_ref() == self.path.as_ref());
        if let Some(derive) = derive {
            // If there are skipped fields without a default value
            if let Some(skip) = self.skip.as_ref() {
                if Override::as_ref(skip)
                    .explicit()
                    .map(|e| e.default.is_none())
                    .unwrap_or(true)
                {
                    // from and try_from must have the custom flag if enabled
                    let mut missing_custom = false;
                    if let Some(from) = derive.from.as_deref() {
                        if from.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true) {
                            missing_custom = true;
                        }
                    }
                    if let Some(try_from) = derive.try_from.as_deref() {
                        if try_from.as_ref().explicit().map(|e| e.custom.is_none()).unwrap_or(true) {
                            missing_custom = true;
                        }
                    }
                    if missing_custom {
                        emit_error!(
                            skip.span(),
                            "Enable `default` here or include `custom` on `from` and `try_from` derives"
                        );
                    }
                }
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
            if let Some(with) = self.with.as_ref() {
                emit_error!(with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(into_with) = self.into_with.as_ref() {
                emit_error!(into_with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(from_with) = self.from_with.as_ref() {
                emit_error!(from_with.span(), "Illegal attribute if 'when' is set")
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
            ItemFieldInput {
                path: path.clone(),
                rename: self.rename.clone(),
                skip: self.skip.clone(),
                with: self.with.clone(),
                into_with: self.into_with.clone(),
                from_with: self.from_with.clone(),
            }
            .validate(derives);
        } else {
            for d in derives {
                ItemFieldInput {
                    path: d.path.clone(),
                    rename: self.rename.clone(),
                    skip: self.skip.clone(),
                    with: self.with.clone(),
                    into_with: self.into_with.clone(),
                    from_with: self.from_with.clone(),
                }
                .validate(derives);
            }
        }
        for item in self.items.iter() {
            item.validate(derives);
        }
    }

    pub(super) fn rename_for(&self, derive_path: &syn::Path) -> Option<&syn::Ident> {
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

    pub(super) fn skip_for(&self, derive_path: &syn::Path) -> Option<&Override<SkipInput>> {
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

    pub(super) fn with_into_for(&self, derive_path: &syn::Path) -> Option<&syn::Path> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.into_with.as_deref().or(item.with.as_deref());
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.into_with.as_deref().or(self.with.as_deref())
            } else {
                None
            }
        } else {
            self.into_with.as_deref().or(self.with.as_deref())
        }
    }

    pub(super) fn with_from_for(&self, derive_path: &syn::Path) -> Option<&syn::Path> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.from_with.as_deref().or(item.with.as_deref());
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.from_with.as_deref().or(self.with.as_deref())
            } else {
                None
            }
        } else {
            self.from_with.as_deref().or(self.with.as_deref())
        }
    }
}
