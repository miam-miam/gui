use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{Data, DeriveInput, Error, Fields, Generics};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use field_attributes::FieldAttributes;

use crate::widget_builder::attributes::StructAttributes;

mod field_attributes;
mod interpolated_path;
mod attributes;

#[derive(Clone)]
pub struct WidgetBuilder {
    ident: Ident,
    generics: Generics,
    attributes: StructAttributes,
    fields: Vec<FieldAttributes>,
}

impl Parse for WidgetBuilder {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = stream.parse()?;

        match input.data {
            Data::Struct(s) => {
                if let Fields::Named(fields) = s.fields {
                    let attributes = StructAttributes::new(&input.attrs)?;
                    let fields = fields.named.into_iter().map(FieldAttributes::new).collect::<syn::Result<_>>()?;
                    Ok(WidgetBuilder {
                        ident: input.ident,
                        generics: input.generics,
                        attributes,
                        fields,
                    })
                } else {
                    Err(Error::new(s.fields.span(), "The derive macro only supports named properties"))
                }
            },
            Data::Enum(e) => Err(Error::new(
                e.enum_token.span,
                "The derive macro does not currently support enums.",
            )),
            Data::Union(u) => Err(Error::new(
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


#[cfg(test)]
mod test {
    use syn::parse_quote;

    use crate::widget_builder::WidgetBuilder;

    #[test]
    pub fn parse_attributes() {
        let builder: WidgetBuilder = parse_quote! {
            #[widget(name = "test", init_path = "new", type_path = "::gui::Test")]
            struct TestBuilder {
                #[widget(property = "set_prop_a", default = 5)]
                prop_a: Option<Var<u8>>
            }
        };
        let multi_attributes: WidgetBuilder = parse_quote! {
            #[widget(name = "test", type_path = "::gui::Test")]
            #[widget(init_path = "new")]
            struct TestBuilder {
                #[widget(property = "set_prop_a")]
                #[widget(default = 5)]
                prop_a: Option<Var<u8>>
            }
        };

        assert_eq!(builder.attributes, multi_attributes.attributes)
    }
}
