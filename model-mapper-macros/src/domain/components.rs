//! Granular mapping components like mapped variants, mapped fields, resolved transforms, skipped forms, and synthesized
//! fields.

use super::*;

/// Mapping for a single enum variant.
pub(crate) struct MappedVariant<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> {
    /// The identifier of the base variant.
    pub(super) ident: syn::Ident,
    /// Mapped fields within this variant.
    pub(super) fields: Vec<MappedField<F, D, I>>,
    /// Optional rename identifier on the derived enum variant.
    pub(super) rename: Option<syn::Ident>,
    /// Optional skip details if the variant is omitted.
    pub(super) skip: Option<Located<D::Skip>>,
    /// Extra rules specific to the direction.
    pub(super) extra: D::VariantExtraFields,
}

impl<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> MappedVariant<F, D, I> {
    /// Gets the base variant identifier.
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    /// Gets the fields within this variant.
    pub(crate) fn fields(&self) -> &[MappedField<F, D, I>] {
        &self.fields
    }

    /// Gets the rename identifier if any.
    pub(crate) fn rename(&self) -> Option<&syn::Ident> {
        self.rename.as_ref()
    }

    /// Gets the variant skip configuration if specified.
    pub(crate) fn skip(&self) -> Option<&Located<D::Skip>> {
        self.skip.as_ref()
    }

    /// Gets the direction-specific extra rules for this variant.
    pub(crate) fn extra(&self) -> &D::VariantExtraFields {
        &self.extra
    }
}

/// Mapping for a struct/variant field.
pub(crate) struct MappedField<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> {
    /// Base field identifier (None for tuple struct/variant fields).
    pub(super) base_ident: Option<syn::Ident>,
    /// The base type of the field.
    pub(super) base_ty: syn::Type,
    /// Optional rename identifier on the derived field.
    pub(super) rename: Option<syn::Ident>,
    /// The type of this field in the derived type, if it differs from the base type's field.
    pub(super) other_ty: Option<syn::Ident>,
    /// Type transformation applied to this field.
    pub(super) transform: ResolvedTransform,
    /// Error mapping strategy (only active for fallible mappings).
    pub(super) error_handler: F::ErrorHandler,
    /// Optional skip details if the field is omitted.
    pub(super) skip: Option<Located<D::Skip>>,
}

impl<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> MappedField<F, D, I> {
    /// Gets the identifier of the base field.
    pub(crate) fn base_ident(&self) -> Option<&syn::Ident> {
        self.base_ident.as_ref()
    }

    /// Gets the base type of the field.
    pub(crate) fn base_ty(&self) -> &syn::Type {
        &self.base_ty
    }

    /// Gets the rename identifier if any.
    pub(crate) fn rename(&self) -> Option<&syn::Ident> {
        self.rename.as_ref()
    }

    /// Gets the `other_ty` identifier if any.
    pub(crate) fn other_ty(&self) -> Option<&syn::Ident> {
        self.other_ty.as_ref()
    }

    /// Gets the type transformation hints.
    pub(crate) fn transform(&self) -> &ResolvedTransform {
        &self.transform
    }

    /// Gets the error mapping handler.
    pub(crate) fn error_handler(&self) -> &F::ErrorHandler {
        &self.error_handler
    }

    /// Gets the skipped field mapping if specified.
    pub(crate) fn skip(&self) -> Option<&Located<D::Skip>> {
        self.skip.as_ref()
    }
}

/// Action-oriented type transformation applied to field mappings.
pub(crate) enum ResolvedTransform {
    /// Standard conversion using standard trait mapping.
    Default,
    /// Custom mapping expression (resolved from `with`, `from_with`, or `into_with`).
    With(Located<syn::Expr>),
    /// Maps over an Option type.
    Option(Box<ResolvedTransform>),
    /// Maps over an iterator/collection.
    Iterator(Box<ResolvedTransform>),
    /// Maps over a hashmap-like collection.
    Map(Box<ResolvedTransform>),
    /// Both source and target are boxed (Deref input, `Box::new` output).
    Boxed(Box<ResolvedTransform>),
    /// Only the source type is boxed (Deref input, output unmodified).
    BoxSource(Box<ResolvedTransform>),
    /// Only the target type is boxed (Input unmodified, `Box::new` output).
    BoxTarget(Box<ResolvedTransform>),
}

/// Skipped behavior in From mapping direction under `TraitMode`.
pub(crate) struct ResolvedSkipTrait {
    /// Statically guaranteed custom fallback expression.
    pub(super) default_expr: Located<syn::Expr>,
}

impl ResolvedSkipTrait {
    /// Gets the statically guaranteed custom fallback expression.
    pub(crate) fn default_expr(&self) -> &Located<syn::Expr> {
        &self.default_expr
    }
}

/// Skipped behavior in From mapping direction under `CustomFnMode`.
pub(crate) struct CustomSkippedFrom {
    /// Optional custom default/fallback expression.
    pub(super) default_expr: Option<Located<syn::Expr>>,
}

impl CustomSkippedFrom {
    /// Gets the custom default/fallback expression if specified.
    pub(crate) fn default_expr(&self) -> Option<&Located<syn::Expr>> {
        self.default_expr.as_ref()
    }
}

/// Skipped behavior in Into mapping direction.
pub(crate) struct SkippedInto {
    /// Optional custom default/fallback expression.
    pub(super) default: Option<Located<syn::Expr>>,
}

impl SkippedInto {
    /// Gets the custom default/fallback expression if specified.
    pub(crate) fn default_expr(&self) -> Option<&Located<syn::Expr>> {
        self.default.as_ref()
    }
}

/// Struct type-level deconstruct rules only valid for the From direction.
pub(crate) struct StructDeconstructRules {
    /// The fields present on the derived type that are bound during deconstruction.
    pub(super) captured_fields: CapturedFields,
    /// Whether extra fields on the derived type are ignored.
    pub(super) ignore_extra: bool,
}

impl StructDeconstructRules {
    /// Gets the fields present on the derived type that are bound during deconstruction.
    pub(crate) fn captured_fields(&self) -> &CapturedFields {
        &self.captured_fields
    }

    /// Returns true if extra fields on the derived type are ignored.
    pub(crate) fn ignore_extra(&self) -> bool {
        self.ignore_extra
    }
}

/// Enum type-level deconstruct rules only valid for the From direction.
pub(crate) struct EnumDeconstructRules {
    /// The variants present on the derived enum that are bound during deconstruction fallback.
    pub(super) captured_variants: CapturedVariants,
    /// Whether extra variants on the derived enum are ignored.
    pub(super) ignore_extra: bool,
}

impl EnumDeconstructRules {
    /// Gets the variants present on the derived enum that are bound during deconstruction fallback.
    pub(crate) fn captured_variants(&self) -> &CapturedVariants {
        &self.captured_variants
    }

    /// Returns true if extra variants on the derived type are ignored.
    pub(crate) fn ignore_extra(&self) -> bool {
        self.ignore_extra
    }
}

/// Variant type-level deconstruct rules only valid for From direction.
pub(crate) struct VariantDeconstructRules {
    /// Whether extra fields on this variant are ignored.
    pub(super) ignore_extra: bool,
    /// Fields present on the derived type variant that are bound during deconstruction.
    pub(super) captured_fields: CapturedFields,
}

impl VariantDeconstructRules {
    /// Returns true if extra fields on this variant are ignored.
    pub(crate) fn ignore_extra(&self) -> bool {
        self.ignore_extra
    }

    /// Gets the fields present on the derived type that are bound during deconstruction.
    pub(crate) fn captured_fields(&self) -> &CapturedFields {
        &self.captured_fields
    }
}

/// Fields present on the derived type that are bound during deconstruction.
pub(crate) struct CapturedFields {
    /// The list of captured field identifiers.
    pub(super) fields: Vec<syn::Ident>,
}

impl CapturedFields {
    /// Gets a slice of the captured field identifiers.
    pub(crate) fn fields(&self) -> &[syn::Ident] {
        &self.fields
    }
}

/// A single variant present on the derived enum that is bound during deconstruction fallback.
pub(crate) struct CapturedVariant {
    pub(super) ident: syn::Ident,
    pub(super) default: Option<syn::Expr>,
}

impl CapturedVariant {
    /// Gets the identifier of the captured variant.
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    /// Gets the custom fallback expression if specified.
    pub(crate) fn default_expr(&self) -> Option<&syn::Expr> {
        self.default.as_ref()
    }
}

/// Variants present on the derived enum that are bound during deconstruction fallback.
pub(crate) struct CapturedVariants {
    /// The list of captured variant identifiers.
    pub(super) variants: Vec<CapturedVariant>,
}

impl CapturedVariants {
    /// Gets a slice of the captured variants.
    pub(crate) fn variants(&self) -> &[CapturedVariant] {
        &self.variants
    }
}

/// Synthesized fields on the derived type populated during construction in `TraitMode`.
pub(crate) struct TraitSynthesizedFields {
    /// The list of synthesized field mappings.
    pub(super) fields: Vec<TraitAddedMappedField>,
    pub(super) ignore_extra: bool,
}

impl TraitSynthesizedFields {
    /// Gets a slice of the synthesized field mappings.
    pub(crate) fn fields(&self) -> &[TraitAddedMappedField] {
        &self.fields
    }

    /// Returns true if extra fields on the derived type are ignored.
    pub(crate) fn ignore_extra(&self) -> bool {
        self.ignore_extra
    }
}

/// Details of a synthesized field in `TraitMode`.
pub(crate) struct TraitAddedMappedField {
    /// The name of the added field.
    pub(super) field: Located<syn::Ident>,
    /// Custom construction default expression.
    pub(super) default: Located<syn::Expr>, // Statically guaranteed default expression
}

impl TraitAddedMappedField {
    /// Gets the name of the added field.
    pub(crate) fn field(&self) -> &Located<syn::Ident> {
        &self.field
    }

    /// Gets the mandatory custom default expression.
    pub(crate) fn mandatory_default(&self) -> &Located<syn::Expr> {
        &self.default
    }
}

/// Synthesized fields on the derived type populated during construction in `CustomFnMode`.
pub(crate) struct CustomSynthesizedFields {
    /// The list of synthesized field mappings.
    pub(super) fields: Vec<CustomAddedMappedField>,
    pub(super) ignore_extra: bool,
}

impl CustomSynthesizedFields {
    /// Gets a slice of the synthesized field mappings.
    pub(crate) fn fields(&self) -> &[CustomAddedMappedField] {
        &self.fields
    }

    /// Returns true if extra fields on the derived type are ignored.
    pub(crate) fn ignore_extra(&self) -> bool {
        self.ignore_extra
    }
}

/// Details of a synthesized field in `CustomFnMode`.
pub(crate) enum CustomAddedMappedField {
    /// Added field with a custom construction default expression.
    WithDefault {
        /// The name of the added field.
        field: Located<syn::Ident>,
        /// Custom construction default expression.
        default: Located<syn::Expr>,
    },
    /// Added field with only a type path.
    WithType {
        /// The name of the added field.
        field: Located<syn::Ident>,
        /// The type path of the added field.
        ty: Located<syn::TypePath>,
    },
}

pub(crate) struct NoErrorHandler;

pub(crate) enum ResolvedErrorHandler {
    /// Map conversion failures to a specific error expression: `err = DerivedError`.
    Value(Located<syn::Expr>),
    /// Map using a helper closure/callable: `err_with = MapFn`.
    With(Located<syn::Expr>),
}
