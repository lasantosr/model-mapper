use core::ops;
use std::collections::{HashMap, HashSet};

use darling::FromMeta;
use syn::{
    TypePath,
    fold::{Fold, fold_type_path},
    visit::{Visit, visit_type_path},
};

#[derive(Debug, Clone)]
pub(crate) struct TypePathWrapper(TypePath);

impl TypePathWrapper {
    pub(crate) fn into_inner(self) -> TypePath {
        self.0
    }
}

impl From<TypePathWrapper> for TypePath {
    #[inline]
    fn from(wrapper: TypePathWrapper) -> Self {
        wrapper.0
    }
}

impl FromMeta for TypePathWrapper {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(str) = value {
            let tp: TypePath = str.parse().map_err(darling::Error::custom)?;
            Ok(TypePathWrapper(tp))
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Path(path_expr) => Ok(TypePathWrapper(TypePath {
                qself: path_expr.qself.clone(),
                path: path_expr.path.clone(),
            })),
            syn::Expr::Lit(lit_expr) => Self::from_value(&lit_expr.lit),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
    }
}

impl AsRef<TypePath> for TypePathWrapper {
    fn as_ref(&self) -> &TypePath {
        &self.0
    }
}

impl ops::Deref for TypePathWrapper {
    type Target = TypePath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl quote::ToTokens for TypePathWrapper {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl PartialEq<TypePath> for TypePathWrapper {
    fn eq(&self, other: &TypePath) -> bool {
        &self.0 == other
    }
}

impl PartialEq<TypePathWrapper> for TypePath {
    #[inline]
    fn eq(&self, other: &TypePathWrapper) -> bool {
        self == &other.0
    }
}

impl PartialEq for TypePathWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub(crate) struct TypePathCollector {
    pub idents: HashSet<syn::Ident>,
}

impl<'ast> Visit<'ast> for TypePathCollector {
    fn visit_type_path(&mut self, i: &'ast TypePath) {
        if i.qself.is_none()
            && i.path.leading_colon.is_none()
            && i.path.segments.len() == 1
            && let Some(first_segment) = i.path.segments.first()
            && first_segment.arguments.is_empty()
        {
            self.idents.insert(first_segment.ident.clone());
        }
        visit_type_path(self, i);
    }
}

pub(crate) struct TypePathReplacer<'a> {
    pub map: &'a HashMap<syn::Ident, syn::Ident>,
}

impl Fold for TypePathReplacer<'_> {
    fn fold_type_path(&mut self, i: TypePath) -> TypePath {
        if i.qself.is_none()
            && i.path.leading_colon.is_none()
            && i.path.segments.len() == 1
            && let Some(first_segment) = i.path.segments.first()
            && first_segment.arguments.is_empty()
            && let Some(new_ident) = self.map.get(&first_segment.ident)
        {
            let mut new_path = i.clone();
            if let Some(seg) = new_path.path.segments.first_mut() {
                seg.ident = new_ident.clone();
            }
            return new_path;
        }
        fold_type_path(self, i)
    }
}
