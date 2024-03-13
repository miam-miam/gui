mod derive;
mod type_registery;

use crate::derive::Derive;
use crate::type_registery::TypeRegistry;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_derive(ToComponent)]
pub fn derive_to_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Derive);
    input.to_token_stream().into()
}

/// Put this in the crate root.
#[proc_macro]
pub fn type_registry(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as TypeRegistry);
    input.to_token_stream().into()
}
