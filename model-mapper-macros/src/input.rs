use darling::{
    ast::{Data, Fields},
    util::{Flag, SpannedValue},
    FromDeriveInput, FromField, FromMeta, FromVariant,
};
use proc_macro_error::{abort_call_site, emit_error};

#[derive(FromDeriveInput)]
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
    /// Wether to ignore all extra fields/variants of the other type
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
    /// Wether to ignore some variants/fields of the other type
    #[darling(default, multiple)]
    ignore: Vec<IgnoreInput>,
    /// Whether to derive [From] the type to self
    #[darling(default)]
    from: SpannedValue<Flag>,
    /// Whether to derive [From] self to the type
    #[darling(default)]
    into: SpannedValue<Flag>,
    /// Whether to derive [TryFrom] the type to self
    #[darling(default)]
    try_from: SpannedValue<Flag>,
    /// Whether to derive [TryFrom] self to the type
    #[darling(default)]
    try_into: SpannedValue<Flag>,
}

#[derive(FromMeta, Clone)]
pub(super) struct ItemInput {
    /// Path of the struct or enum
    #[darling(rename = "ty")]
    pub(super) path: SpannedValue<syn::Path>,
    /// Wether to ignore all extra fields/variants of the other type
    #[darling(default)]
    pub(super) ignore_extra: SpannedValue<Flag>,
    /// Wether to ignore some variants/fields of the other type
    #[darling(default, multiple)]
    pub(super) ignore: Vec<IgnoreInput>,
    /// Whether to derive [From] the type to self
    #[darling(default)]
    pub(super) from: SpannedValue<Flag>,
    /// Whether to derive [From] self to the type
    #[darling(default)]
    pub(super) into: SpannedValue<Flag>,
    /// Whether to derive [TryFrom] the type to self
    #[darling(default)]
    pub(super) try_from: SpannedValue<Flag>,
    /// Whether to derive [TryFrom] self to the type
    #[darling(default)]
    pub(super) try_into: SpannedValue<Flag>,
}

#[derive(FromVariant, Clone)]
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
    /// Wether to ignore all extra fields of the other variant
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
    /// Wether to ignore some fields of the variant
    #[darling(default, multiple)]
    ignore: Vec<IgnoreInput>,
    /// To skip this variant when deriving (defaults to `false`)
    #[darling(default)]
    skip: SpannedValue<Flag>,
    /// To populate this variant when skipped (defaults to `Default::default()`)
    #[darling(default)]
    default: Option<SpannedValue<syn::Expr>>,
    /// To rename the variant
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
}
macro_field_utils::variant_info!(VariantReceiver, FieldReceiver);

#[derive(FromMeta, Clone)]
struct ItemVariantInput {
    /// Path of the struct or enum to derive
    #[darling(rename = "ty")]
    path: SpannedValue<syn::Path>,
    /// Wether to ignore all extra fields of the other variant
    #[darling(default)]
    ignore_extra: SpannedValue<Flag>,
    /// Wether to ignore some fields of the variant
    #[darling(default, multiple)]
    ignore: Vec<IgnoreInput>,
    /// To skip this variant when deriving (defaults to `false`)
    #[darling(default)]
    skip: SpannedValue<Flag>,
    /// To populate this variant when skipped (defaults to `Default::default()`)
    #[darling(default)]
    default: Option<SpannedValue<syn::Expr>>,
    /// To rename the variant
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
}

#[derive(FromField, Clone)]
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
    /// To skip this field when deriving (defaults to `false`)
    #[darling(default)]
    skip: SpannedValue<Flag>,
    /// To populate this field when skipped (defaults to `Default::default()`)
    #[darling(default)]
    default: Option<SpannedValue<syn::Expr>>,
    /// To use some function to map the values
    #[darling(default)]
    with: Option<SpannedValue<syn::Path>>,
    /// To use some function to map the values when deriving Try
    #[darling(default)]
    try_with: Option<SpannedValue<syn::Path>>,
    /// To rename the field
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
}
macro_field_utils::field_info!(FieldReceiver);

#[derive(FromMeta, Clone)]
struct ItemFieldInput {
    /// Path of the struct or enum to derive
    #[darling(rename = "ty")]
    path: SpannedValue<syn::Path>,
    /// To skip this field when deriving (defaults to `false`)
    #[darling(default)]
    skip: SpannedValue<Flag>,
    /// To populate this field when skipped (defaults to `Default::default()`)
    #[darling(default)]
    default: Option<SpannedValue<syn::Expr>>,
    /// To use some function to map the values
    #[darling(default)]
    with: Option<SpannedValue<syn::Path>>,
    /// To use some function to map the values when deriving Try
    #[darling(default)]
    try_with: Option<SpannedValue<syn::Path>>,
    /// To rename the field
    #[darling(default)]
    rename: Option<SpannedValue<syn::Ident>>,
}

#[derive(FromMeta, Clone)]
pub(super) struct IgnoreInput {
    /// Name of the ignored field or variant
    pub(super) field: SpannedValue<syn::Ident>,
    /// Default value for the field
    #[darling(default)]
    pub(super) default: Option<syn::Expr>,
}

impl MapperOpts {
    pub(super) fn items(&self) -> Vec<ItemInput> {
        if !self.items.is_empty() {
            if let Some(path) = self.path.as_ref() {
                emit_error!(path.span(), "Illegal attribute when 'derive' is set")
            }
            if self.ignore_extra.is_present() {
                emit_error!(self.ignore_extra.span(), "Illegal attribute when 'derive' is set")
            }
            if !self.ignore.is_empty() {
                for i in &self.ignore {
                    emit_error!(i.field.span(), "Illegal attribute when 'derive' is set")
                }
            }
            if self.from.is_present() {
                emit_error!(self.from.span(), "Illegal attribute when 'derive' is set")
            }
            if self.into.is_present() {
                emit_error!(self.into.span(), "Illegal attribute when 'derive' is set")
            }
            if self.try_from.is_present() {
                emit_error!(self.try_from.span(), "Illegal attribute when 'derive' is set")
            }
            if self.try_into.is_present() {
                emit_error!(self.try_into.span(), "Illegal attribute when 'derive' is set")
            }
            self.items.to_vec()
        } else if let Some(path) = self.path.as_ref() {
            vec![ItemInput {
                path: path.clone(),
                ignore_extra: self.ignore_extra,
                ignore: self.ignore.clone(),
                from: self.from,
                into: self.into,
                try_from: self.try_from,
                try_into: self.try_into,
            }]
        } else {
            abort_call_site!("One of 'ty' or 'derive' must be set")
        }
    }
}

impl ItemInput {
    pub(super) fn validate(&self) {
        if !self.from.is_present()
            && !self.into.is_present()
            && !self.try_from.is_present()
            && !self.try_into.is_present()
        {
            emit_error!(
                self.path.span(),
                "One of 'from', 'into', 'try_from' or 'try_into' must be set"
            );
        }
    }
}

impl VariantReceiver {
    pub(super) fn validate(&self, derives: &[ItemInput]) {
        if !self.items.is_empty() {
            if let Some(path) = self.path.as_ref() {
                emit_error!(path.span(), "Illegal attribute if 'when' is set")
            }
            if self.ignore_extra.is_present() {
                emit_error!(self.ignore_extra.span(), "Illegal attribute if 'when' is set")
            }
            if !self.ignore.is_empty() {
                for i in &self.ignore {
                    emit_error!(i.field.span(), "Illegal attribute if 'when' is set")
                }
            }
            if self.skip.is_present() {
                emit_error!(self.skip.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(default) = self.default.as_ref() {
                emit_error!(default.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(rename) = self.rename.as_ref() {
                emit_error!(rename.span(), "Illegal attribute if 'when' is set")
            }
        }

        if let Some(path) = &self.path {
            if !derives.iter().any(|d| d.path.as_ref() == path.as_ref()) {
                emit_error!(path.span(), "There is no derive defined for this type");
            }
        }
        if !self.skip.is_present() {
            if let Some(default) = &self.default {
                emit_error!(default.span(), "Illegal attribute if no 'skip' is set")
            }
        }
        for item in self.items.iter() {
            if !derives.iter().any(|d| d.path.as_ref() == item.path.as_ref()) {
                emit_error!(item.path.span(), "There is no derive defined for this type");
            }
            if !item.skip.is_present() {
                if let Some(default) = &item.default {
                    emit_error!(default.span(), "Illegal attribute if no 'skip' is set")
                }
            }
        }

        self.fields.iter().for_each(|f| f.validate(derives));
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

    pub(super) fn ignore_for(&self, derive_path: &syn::Path) -> Option<&Vec<IgnoreInput>> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                if item.ignore.is_empty() {
                    return None;
                } else {
                    return Some(item.ignore.as_ref());
                }
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                if self.ignore.is_empty() {
                    None
                } else {
                    Some(self.ignore.as_ref())
                }
            } else {
                None
            }
        } else if self.ignore.is_empty() {
            None
        } else {
            Some(self.ignore.as_ref())
        }
    }

    pub(super) fn skip_for(&self, derive_path: &syn::Path) -> Option<Option<&syn::Expr>> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return if item.skip.is_present() {
                    Some(item.default.as_deref())
                } else {
                    None
                };
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                if self.skip.is_present() {
                    Some(self.default.as_deref())
                } else {
                    None
                }
            } else {
                None
            }
        } else if self.skip.is_present() {
            Some(self.default.as_deref())
        } else {
            None
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
}

impl FieldReceiver {
    pub(super) fn validate(&self, derives: &[ItemInput]) {
        if !self.items.is_empty() {
            if let Some(path) = self.path.as_ref() {
                emit_error!(path.span(), "Illegal attribute if 'when' is set")
            }
            if self.skip.is_present() {
                emit_error!(self.skip.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(default) = self.default.as_ref() {
                emit_error!(default.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(with) = self.with.as_ref() {
                emit_error!(with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(try_with) = self.try_with.as_ref() {
                emit_error!(try_with.span(), "Illegal attribute if 'when' is set")
            }
            if let Some(rename) = self.rename.as_ref() {
                emit_error!(rename.span(), "Illegal attribute if 'when' is set")
            }
        }

        if let Some(path) = &self.path {
            if !derives.iter().any(|d| d.path.as_ref() == path.as_ref()) {
                emit_error!(path.span(), "There is no derive defined for this type");
            }
        }
        if !self.skip.is_present() {
            if let Some(default) = &self.default {
                emit_error!(default.span(), "Illegal attribute if no 'skip' is set")
            }
        }
        for item in self.items.iter() {
            if !derives.iter().any(|d| d.path.as_ref() == item.path.as_ref()) {
                emit_error!(item.path.span(), "There is no derive defined for this type");
            }
            if !item.skip.is_present() {
                if let Some(default) = &item.default {
                    emit_error!(default.span(), "Illegal attribute if no 'skip' is set")
                }
            }
        }
    }

    pub(super) fn skip_for(&self, derive_path: &syn::Path) -> Option<Option<&syn::Expr>> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return if item.skip.is_present() {
                    Some(item.default.as_deref())
                } else {
                    None
                };
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                if self.skip.is_present() {
                    Some(self.default.as_deref())
                } else {
                    None
                }
            } else {
                None
            }
        } else if self.skip.is_present() {
            Some(self.default.as_deref())
        } else {
            None
        }
    }

    pub(super) fn with_for(&self, derive_path: &syn::Path) -> Option<&syn::Path> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.with.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.with.as_deref()
            } else {
                None
            }
        } else {
            self.with.as_deref()
        }
    }

    pub(super) fn try_with_for(&self, derive_path: &syn::Path) -> Option<&syn::Path> {
        for item in &self.items {
            if item.path.as_ref() == derive_path {
                return item.try_with.as_deref();
            }
        }
        if let Some(path) = &self.path {
            if path.as_ref() == derive_path {
                self.try_with.as_deref()
            } else {
                None
            }
        } else {
            self.try_with.as_deref()
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
}
