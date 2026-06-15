use darling::FromDeriveInput as _;
use proc_macro2::TokenStream;
use quote::ToTokens as _;

use crate::{domain, expand, input::MappingSpec, parse::MapperOpts};

pub(crate) fn r#impl(input: syn::DeriveInput) -> TokenStream {
    // Parse input derive tokens using darling
    let opts = match MapperOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(err) => {
            return err.write_errors();
        }
    };

    // Convert the input into the mapping spec
    let spec = match MappingSpec::try_from(opts) {
        Ok(item) => item,
        Err(err) => return err.into_compile_error(),
    };

    // Parse the input into the domain model
    let mappings = match domain::parse_mappings(&spec) {
        Ok(map) => map,
        Err(err) => return err.into_compile_error(),
    };

    // Generate the output tokens for each mapping
    let mut output = TokenStream::new();
    for item in mappings.values() {
        expand::item_mapping(item).to_tokens(&mut output);
    }

    output
}
