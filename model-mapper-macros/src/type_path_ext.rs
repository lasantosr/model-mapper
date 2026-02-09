use std::collections::{HashMap, HashSet};

use darling::FromMeta;
use syn::{fold::Fold, visit::Visit, TypePath};

#[derive(Debug, Clone)]
pub(crate) struct TypePathWrapper(pub(crate) syn::TypePath);

impl FromMeta for TypePathWrapper {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(s) = value {
            let tp: syn::TypePath = s.parse().map_err(darling::Error::custom)?;
            Ok(TypePathWrapper(tp))
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Path(path_expr) => Ok(TypePathWrapper(syn::TypePath {
                qself: path_expr.qself.clone(),
                path: path_expr.path.clone(),
            })),
            syn::Expr::Lit(lit_expr) => Self::from_value(&lit_expr.lit),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
    }
}

impl AsRef<syn::TypePath> for TypePathWrapper {
    fn as_ref(&self) -> &syn::TypePath {
        &self.0
    }
}

impl std::ops::Deref for TypePathWrapper {
    type Target = syn::TypePath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl quote::ToTokens for TypePathWrapper {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl PartialEq<syn::TypePath> for TypePathWrapper {
    fn eq(&self, other: &syn::TypePath) -> bool {
        &self.0 == other
    }
}

impl PartialEq<TypePathWrapper> for syn::TypePath {
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
    pub(crate) idents: HashSet<syn::Ident>,
}

impl<'ast> Visit<'ast> for TypePathCollector {
    fn visit_type_path(&mut self, i: &'ast TypePath) {
        if i.qself.is_none()
            && i.path.leading_colon.is_none()
            && i.path.segments.len() == 1
            && i.path.segments[0].arguments.is_empty()
        {
            self.idents.insert(i.path.segments[0].ident.clone());
        }
        syn::visit::visit_type_path(self, i);
    }
}

pub(crate) struct TypePathReplacer<'a> {
    pub(crate) map: &'a HashMap<syn::Ident, syn::Ident>,
}

impl<'a> Fold for TypePathReplacer<'a> {
    fn fold_type_path(&mut self, i: TypePath) -> TypePath {
        if i.qself.is_none()
            && i.path.leading_colon.is_none()
            && i.path.segments.len() == 1
            && i.path.segments[0].arguments.is_empty()
            && let Some(new_ident) = self.map.get(&i.path.segments[0].ident)
        {
            let mut new_path = i.clone();
            new_path.path.segments[0].ident = new_ident.clone();
            return new_path;
        }
        syn::fold::fold_type_path(self, i)
    }
}
