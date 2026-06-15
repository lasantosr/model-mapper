//! Zero-Sized typestate markers, mapping shapes, direction/codegen modes, and fallibility modes.

use super::*;

/// A marker trait mapping a structural shape to its concrete mapping representation.
pub(crate) trait MappingShape {
    /// Concrete mapping representation associated with the shape.
    type Config<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode>;
}

/// Marker for struct mappings.
pub(crate) struct StructShape;
impl MappingShape for StructShape {
    type Config<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> = StructMapping<F, D, I>;
}

/// Marker for enum mappings.
pub(crate) struct EnumShape;
impl MappingShape for EnumShape {
    type Config<F: FallibilityMode, D: DirectionMode<I>, I: ImplMode> = EnumMapping<F, D, I>;
}

/// Trait to configure fallibility-specific resolved error handler representation.
pub(crate) trait FallibilityMode {
    /// Resolved error handler details.
    type ErrorHandler;
}

/// Infallible conversion marker.
pub(crate) struct InfallibleMode;
impl FallibilityMode for InfallibleMode {
    type ErrorHandler = NoErrorHandler;
}

/// Fallible conversion mappings.
pub(crate) struct FallibleMode {
    /// The base error type for individual field conversions.
    pub(super) base_error_ty: syn::Type,
    /// The compilation mode for error collection.
    pub(super) mode: FallibilityModeDetails,
}
impl FallibilityMode for FallibleMode {
    type ErrorHandler = Option<ResolvedErrorHandler>;
}

impl FallibleMode {
    /// Gets the base error type of individual field conversions.
    pub(crate) fn base_error_ty_direct(&self) -> &syn::Type {
        &self.base_error_ty
    }

    /// Gets the compilation mode for error collection.
    pub(crate) fn mode_direct(&self) -> &FallibilityModeDetails {
        &self.mode
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "Accumulate variant is larger but is part of the core typestate design"
)]
/// Details of the error handling strategy.
pub(crate) enum FallibilityModeDetails {
    /// Return the error immediately on first failure.
    ShortCircuit,
    /// Accumulate errors into a collection.
    Accumulate {
        /// Type of the error collection accumulator (e.g. Vec<Err>).
        accumulator_ty: syn::Type,
    },
}

/// Trait to configure direction-specific skipped forms, type-level deconstruct rules, and variant-level deconstruct
/// rules.
pub(crate) trait DirectionMode<I: ImplMode> {
    /// Skip details wrapper.
    type Skip;
    /// Struct target deconstruct rules.
    type StructExtra;
    /// Enum target deconstruct rules.
    type EnumExtra;
    /// Variant target deconstruct rules.
    type VariantExtraFields;
}

/// Derived -> Base conversion direction.
pub(crate) struct FromDirection;
impl<I: ImplMode> DirectionMode<I> for FromDirection {
    type EnumExtra = EnumDeconstructRules;
    type Skip = I::SkipFrom;
    type StructExtra = StructDeconstructRules;
    type VariantExtraFields = VariantDeconstructRules;
}

/// Base -> Derived conversion direction.
pub(crate) struct IntoDirection;
impl<I: ImplMode> DirectionMode<I> for IntoDirection {
    type EnumExtra = ();
    type Skip = SkippedInto;
    type StructExtra = I::StructIntoAddedFields;
    type VariantExtraFields = I::VariantIntoAddedFields;
}

/// Implementation mode trait.
pub(crate) trait ImplMode: 'static {
    /// Skip details wrapper for `FromDirection`.
    type SkipFrom;
    /// Struct target synthesized fields for `IntoDirection`.
    type StructIntoAddedFields;
    /// Variant target synthesized fields for `IntoDirection`.
    type VariantIntoAddedFields;
}

/// Standard trait implementation.
pub(crate) struct TraitMode;
impl ImplMode for TraitMode {
    type SkipFrom = ResolvedSkipTrait;
    type StructIntoAddedFields = TraitSynthesizedFields;
    type VariantIntoAddedFields = TraitSynthesizedFields;
}

/// Custom named function implementation.
pub(crate) struct CustomFnMode {
    /// The custom mapping function name.
    pub(super) fn_name: Option<syn::Ident>,
}
impl ImplMode for CustomFnMode {
    type SkipFrom = CustomSkippedFrom;
    type StructIntoAddedFields = CustomSynthesizedFields;
    type VariantIntoAddedFields = CustomSynthesizedFields;
}

impl CustomFnMode {
    /// Gets the custom mapping function name if specified.
    pub(crate) fn fn_name_direct(&self) -> Option<&syn::Ident> {
        self.fn_name.as_ref()
    }
}
