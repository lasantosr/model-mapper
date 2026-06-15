//! AST representation and structures for mapping specifications.
//!
//! This module defines the types that represent a verified mapping specification
//! (e.g. [`MappingSpec`], [`BaseData`], and [`DeriveConfig`]). These types model
//! the base struct/enum being mapped, its constituent fields/variants, and the
//! derived configuration rules parsed from the macro attributes.

use core::{fmt, ops};

/// Specification for generating model mappings.
pub(crate) struct MappingSpec {
    /// Identifier of the base struct/enum.
    pub ident: syn::Ident,
    /// Generics of the base struct/enum.
    pub generics: syn::Generics,
    /// Structural data (fields or variants) of the base item.
    pub data: BaseData,
    /// Derived target configurations.
    pub derives: Vec<DeriveConfig>,
}

/// Data payload structure of the base item.
pub(crate) enum BaseData {
    /// Named or unnamed struct fields.
    Struct(Vec<BaseField>),
    /// Enum variants.
    Enum(Vec<BaseVariant>),
}

/// A struct or variant field definition.
pub(crate) struct BaseField {
    /// Identifier of the field (None for tuple fields).
    pub ident: Option<syn::Ident>,
    /// Type of the field.
    pub ty: syn::Type,
    /// Attributes and overrides for this field.
    pub mappings: FieldMappings,
}

/// Field attributes configuration, potentially derived type-specific.
pub(crate) enum FieldMappings {
    /// Default mapping configuration applied to all derived types.
    Default(Box<Located<FieldMapping>>),
    /// Derived type-specific override mapping configurations.
    Overrides(Vec<(syn::TypePath, Located<FieldMapping>)>),
}

/// Configuration options for mapping a single field.
#[derive(Clone, Default)]
pub(crate) struct FieldMapping {
    /// To rename this field on the derived type.
    pub rename: Option<syn::Ident>,
    /// Whether to skip this field because the derived type doesn't have it.
    pub skip: Option<Located<SkippedField>>,
    /// The type of this field in the derived type, if it differs from the base type's field.
    pub other_ty: Option<syn::Ident>,
    /// Hint to navigate nesting or provide custom mapping function.
    pub hint: TransformHint,
    /// Map conversion failures for this field to a specific error value.
    pub err: Option<syn::Expr>,
    /// Map conversion failures for this field using a helper function/callable.
    pub err_with: Option<syn::Expr>,
}

/// An enum variant definition.
pub(crate) struct BaseVariant {
    /// Identifier of the variant.
    pub ident: syn::Ident,
    /// Fields contained within the variant.
    pub fields: Vec<BaseField>,
    /// Attributes and overrides for this variant.
    pub mappings: VariantMappings,
}

/// Variant attributes configuration, potentially derived type-specific.
pub(crate) enum VariantMappings {
    /// Default mapping configuration applied to all derived types.
    Default(Box<VariantMapping>),
    /// Derived type-specific override mapping configurations.
    Overrides(Vec<(syn::TypePath, VariantMapping)>),
}

/// Configuration options for mapping an enum variant.
#[derive(Clone, Default)]
pub(crate) struct VariantMapping {
    /// To rename this variant on the derived enum.
    pub rename: Option<syn::Ident>,
    /// Additional fields of the variant that the derived variant has and the base variant doesn't.
    pub add: Vec<AddedField>,
    /// Whether to skip this variant because the derived enum doesn't have it.
    pub skip: Option<Located<SkippedField>>,
    /// Whether to ignore all extra fields of the derived variant (only valid for `from` and `try_from`).
    pub ignore_extra: bool,
}

/// Configuration for deriving mappings to a specific derived type.
#[derive(Clone)]
pub(crate) struct DeriveConfig {
    /// The derived type to map to/from.
    pub path: Located<syn::TypePath>,
    /// Whether to derive `From` the derived type for the base type.
    pub from: Option<ConversionOptions>,
    /// Whether to derive `From` the base type for the derived type.
    pub into: Option<ConversionOptions>,
    /// Whether to derive `TryFrom` the derived type for the base type.
    pub try_from: Option<ConversionOptions>,
    /// Whether to derive `TryFrom` the base type for the derived type.
    pub try_into: Option<ConversionOptions>,
    /// Additional fields (for structs with named fields) or variants (for enums) the derived type has and the base
    /// type doesn't.
    pub add: Vec<AddedField>,
    /// Whether to ignore all extra fields (for structs) or variants (for enums) of the derived type.
    pub ignore_extra: Option<proc_macro2::Span>,
}

/// Settings for a trait derive operation.
#[derive(Clone)]
pub(crate) struct ConversionOptions {
    /// Use a custom function instead of generating trait impl.
    pub custom: Option<Located<Option<syn::Ident>>>,
    /// Explicit error type for fallible conversions.
    pub err: Option<syn::TypePath>,
    /// Collect mapping errors rather than early-exiting.
    pub accumulate: Option<Located<Option<syn::TypePath>>>,
}

/// Configuration for fields added at the variant/struct level.
#[derive(Clone)]
pub(crate) struct AddedField {
    /// Name of the field.
    pub field: Located<syn::Ident>,
    /// Type of the field.
    pub ty: Option<Located<syn::TypePath>>,
    /// Default value expression of the field.
    pub default: Option<Located<syn::Expr>>,
}

/// Configuration for a skipped field.
#[derive(Clone)]
pub(crate) struct SkippedField {
    /// Default value expression of the field.
    pub default: Option<Located<syn::Expr>>,
}

/// Hints that configure field type transformation or custom conversions.
#[derive(Clone, Default)]
pub(crate) struct TransformHint {
    /// If the field type doesn't implement Into/TryInto, this property allows customizing the behavior by providing a
    /// conversion function.
    pub with: Option<Located<syn::Expr>>,
    /// The same as with but only for the `into` or `try_into` derives.
    pub into_with: Option<Located<syn::Expr>>,
    /// The same as with but only for the `from` or `try_from` derives.
    pub from_with: Option<Located<syn::Expr>>,
    /// The field is an Option and the inner value shall be mapped.
    pub opt: Option<Located<Box<TransformHint>>>,
    /// The field is an iterator/collection and the inner value shall be mapped.
    pub iter: Option<Located<Box<TransformHint>>>,
    /// The field is a hashmap-like iterator and the inner value shall be mapped.
    pub map: Option<Located<Box<TransformHint>>>,
    /// The field is a Box and the inner value shall be mapped.
    pub boxed: Option<Located<Box<TransformHint>>>,
    /// The derived field is a Box while the base field is not.
    pub r#box: Option<Located<Box<TransformHint>>>,
    /// The base field is a Box while the derived field is not.
    pub unbox: Option<Located<Box<TransformHint>>>,
}

/// Pairs a value with its source-location span.
#[derive(Clone)]
pub(crate) struct Located<T>(T, proc_macro2::Span);

impl<T> Located<T> {
    pub(crate) fn new(value: T, span: proc_macro2::Span) -> Self {
        Self(value, span)
    }

    pub(crate) fn span(&self) -> proc_macro2::Span {
        self.1
    }

    pub(crate) fn error(&self, message: impl fmt::Display) -> syn::Error {
        syn::Error::new(self.1, message)
    }

    pub(crate) fn value(&self) -> &T {
        &self.0
    }
}

impl<T> ops::Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for Located<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: quote::ToTokens> quote::ToTokens for Located<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}
