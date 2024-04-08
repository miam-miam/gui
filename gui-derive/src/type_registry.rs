use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::env;
use syn::parse::{Nothing, Parse, ParseStream};

pub struct TypeRegistry;

impl Parse for TypeRegistry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: Nothing = input.parse()?;
        Ok(TypeRegistry)
    }
}

impl ToTokens for TypeRegistry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let env = env::var("GUI_COMPONENTS").unwrap();
        let component_idents = env.split(',').map(|s| format_ident!("{s}"));
        tokens.extend(quote! {
            #[doc(hidden)]
            pub(crate) mod __gui_private {
                #(pub(crate) struct #component_idents;)*
            }
        })
    }
}
