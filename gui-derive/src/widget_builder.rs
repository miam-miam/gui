use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{Data, DeriveInput};
use syn::parse::{Parse, ParseStream};

use field_attributes::FieldAttributes;

mod field_attributes;
mod interpolated_path;

#[derive(Clone)]
pub struct WidgetBuilder {
    ident: Ident,
    widget_name: Ident,
    type_path: syn::Path,
    init_path: syn::Path,
    fields: Vec<FieldAttributes>,
}

impl Parse for WidgetBuilder {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = stream.parse()?;

        match input.data {
            Data::Struct(s) => {
                todo!()
            }
            Data::Enum(e) => Err(syn::Error::new(
                e.enum_token.span,
                "The derive macro does not currently support enums.",
            )),
            Data::Union(u) => Err(syn::Error::new(
                u.union_token.span,
                "The derive macro does not currently support unions.",
            )),
        }
    }
}

impl ToTokens for WidgetBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        todo!()
    }
}
