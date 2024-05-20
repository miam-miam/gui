use std::sync::atomic::{AtomicU32, Ordering};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Path, PredicateType};

use crate::widget_builder::field_attributes::FieldAttributes;
use crate::widget_builder::WidgetBuilder;

fn generate_prop_assert(
    assert_path: &TokenStream,
    widget_type: &TokenStream,
    widget_turbo: &TokenStream,
    span: Span,
    bound: &Option<PredicateType>,
    property: &Path,
) -> TokenStream {
    /// Ensure generated functions are unique
    static FUNC_COUNTER: AtomicU32 = AtomicU32::new(0);

    match bound {
        None => quote_spanned!(span=>
            #assert_path property_path::<#widget_type, _>(#widget_turbo :: #property);
        ),
        Some(static_bound) => {
            let func_name = format_ident!(
                "assert_type_{}",
                FUNC_COUNTER.fetch_add(1, Ordering::Relaxed)
            );
            let type_param = &static_bound.bounded_ty;
            quote! {
                {
                    use super::*;
                    fn #func_name<#type_param>() where #static_bound {
                        #assert_path property_path::<#widget_type, #type_param>(#widget_turbo :: #property);
                    }
                }
            }
        }
    }
}

impl FieldAttributes {
    pub fn assert(&self, widget_type: &TokenStream, widget_turbo: &TokenStream) -> TokenStream {
        let mut stream = TokenStream::new();

        let assert_path = quote!(::gui_custom::__private::assertions::);
        let widget_id = quote!(::gui_custom::widget::WidgetID);
        let span = self.field.ident.span();

        if let Some(static_prop) = &self.static_prop {
            stream.extend(generate_prop_assert(
                &assert_path,
                widget_type,
                widget_turbo,
                span,
                &self.static_bound,
                static_prop,
            ));
        }
        if let Some(var_prop) = &self.var_prop {
            stream.extend(generate_prop_assert(
                &assert_path,
                widget_type,
                widget_turbo,
                span,
                &self.var_bound,
                var_prop,
            ));
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
            #[doc(hidden)]
            mod #module {
                #[allow(unused)]
                fn assert_types() {
                    #field_assertions
                }
            }
        }
    }
}
