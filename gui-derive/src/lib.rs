mod derive;

use crate::derive::Derive;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_derive(ToComponent)]
pub fn derive_to_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Derive);
    input.to_token_stream().into()
}
