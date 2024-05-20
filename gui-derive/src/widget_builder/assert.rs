use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::widget_builder::field_attributes::FieldAttributes;
use crate::widget_builder::WidgetBuilder;

impl FieldAttributes {
    pub fn assert(&self, widget_type: &TokenStream, widget_turbo: &TokenStream) -> TokenStream {
        let mut stream = TokenStream::new();

        let assert_path = quote!(::gui_custom::__private::assertions::);
        let widget_id = quote!(::gui_custom::widget::WidgetID);

        if let Some(static_prop) = &self.static_prop {
            stream.extend(quote!(
                #assert_path property_path::<#widget_type, _>(#widget_turbo :: #static_prop);
            ))
        }
        if let Some(var_prop) = &self.var_prop {
            stream.extend(quote!(
                #assert_path property_path::<#widget_type, _>(#widget_turbo :: #var_prop);
            ))
        }
        if let Some(fluent) = &self.fluent {
            stream.extend(
                quote!( #assert_path fluent_path::<#widget_type>(#widget_turbo :: #fluent); ),
            )
        }
        if let Some(component) = &self.component {
            stream.extend(
                quote!( #assert_path component_path::<#widget_type>(#widget_turbo :: #component); ),
            )
        }
        if let Some(child) = &self.child {
            stream.extend(quote!( #assert_path child_path::<#widget_type, #widget_id>(#widget_turbo :: #child); ))
        }
        if let Some(children) = &self.children {
            stream.extend(quote!( #assert_path children_path::<#widget_type, #widget_id>(#widget_turbo :: #children); ))
        }
        stream
    }
}

impl WidgetBuilder {
    pub fn generate_assertions(&self) -> TokenStream {
        let module = format_ident!("__assertions_{}", &self.name);

        let mut widget_type = self.attributes.type_path.clone();
        if let Some(i) = widget_type.segments.first_mut() {
            *i = format_ident!("crate");
        }
        if let Some(generics) = &mut widget_type.generics {
            let fake_path = quote!(::gui_custom::__private::fakes::);
            generics
                .args
                .iter_mut()
                .for_each(|t| t.convert_to_fakes(&fake_path))
        }

        let mut widget_turbo = widget_type.clone();
        if widget_turbo.generics.is_some() {
            widget_turbo.segments.push_punct(Default::default());
        }

        let widget_type = widget_type.to_token_stream();
        let widget_turbo = widget_turbo.to_token_stream();
        let field_assertions: TokenStream = self
            .fields
            .iter()
            .map(|a| a.assert(&widget_type, &widget_turbo))
            .collect();

        quote! {
            #[allow(non_snake_case)]
            mod #module {
                #[allow(unused)]
                fn assert_types() {
                    #field_assertions
                }
            }
        }
    }
}
