use crate::fluent::FluentIdent;
use crate::tokenstream::EqTokenStream;
use gui_core::parse::var::Name;
use gui_core::widget::WidgetBuilder;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::hash::Hash;

fn gen_idents() -> (Ident, Ident, Ident) {
    let widget_ident = Ident::new("widget", Span::call_site());
    let value_ident = Ident::new("value", Span::call_site());
    let handle_ident = Ident::new("handle_ref", Span::call_site());
    (widget_ident, value_ident, handle_ident)
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct Statics(pub Vec<(&'static str, EqTokenStream)>);

impl Statics {
    pub fn new(builder: &dyn WidgetBuilder) -> Self {
        Statics(
            builder
                .get_statics()
                .into_iter()
                .map(|(p, s)| (p, s.into()))
                .collect_vec(),
        )
    }

    pub fn gen_statics(
        &self,
        widget_builder: &dyn WidgetBuilder,
        widget_stmt: &TokenStream,
        static_stream: &mut TokenStream,
    ) {
        let (widget_ident, value_ident, handle_ident) = gen_idents();
        if !self.0.is_empty() {
            static_stream.extend(quote! {
                let widget = #widget_stmt;
            })
        }

        for (prop, value) in &self.0 {
            static_stream.extend(quote! {
                let value = #value;
            });
            widget_builder.on_property_update(
                prop,
                &widget_ident,
                &value_ident,
                &handle_ident,
                static_stream,
            );
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Fluents(pub Vec<FluentIdent>);

impl Fluents {
    pub fn new(
        builder: &dyn WidgetBuilder,
        component_name: &str,
        widget_name: Option<&str>,
        widget_type_name: &str,
    ) -> Self {
        Self(
            builder
                .get_fluents()
                .into_iter()
                .map(|(prop, fluent)| {
                    FluentIdent::new(prop, fluent, component_name, widget_name, widget_type_name)
                })
                .collect(),
        )
    }

    pub fn new_state_override(
        builder: &dyn WidgetBuilder,
        component_name: &str,
        widget_name: &str,
        state_name: &str,
    ) -> Self {
        Self(
            builder
                .get_fluents()
                .into_iter()
                .map(|(prop, fluent)| {
                    FluentIdent::new_state_override(
                        prop,
                        fluent,
                        component_name,
                        widget_name,
                        state_name,
                    )
                })
                .collect(),
        )
    }

    pub fn gen_fluents(
        &self,
        widget_builder: &dyn WidgetBuilder,
        widget_stmt: &TokenStream,
        fluent_stream: &mut TokenStream,
    ) {
        let (widget_ident, value_ident, handle_ident) = gen_idents();

        for fluent in &self.0 {
            let property_ident = (!fluent.fluent.vars.is_empty()).then_some(&fluent.property_ident);
            let property_iter = property_ident.iter();
            let fluent_name = &fluent.name;
            let fluent_arg = &fluent.ident;
            let mut on_property_update = TokenStream::new();

            let arg = if fluent.fluent.vars.is_empty() {
                quote! {None}
            } else {
                quote! {Some(&self.#fluent_arg)}
            };

            widget_builder.on_property_update(
                fluent.property,
                &widget_ident,
                &value_ident,
                &handle_ident,
                &mut on_property_update,
            );

            fluent_stream.extend(quote! {
                if force_update #(|| #property_iter)* {
                    let value = get_bundle_message(#fluent_name, #arg);
                    let #widget_ident = #widget_stmt;
                    #on_property_update
                }
            });
        }
    }

    pub fn gen_fluent_arg_update(&self, var_name: &Name, fluent_stream: &mut TokenStream) {
        let (_, value_ident, _) = gen_idents();

        for fluent in self.0.iter().filter(|f| f.fluent.vars.contains(var_name)) {
            let fluent_ident = &fluent.ident;
            let prop = Ident::new(fluent.property, Span::call_site());
            let string_var_name: &str = var_name;
            fluent_stream.extend(quote! {
                #prop = true;
                self.#fluent_ident.set(#string_var_name, #value_ident);
            });
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Variables(pub Vec<(&'static str, Name)>);

impl Variables {
    pub fn gen_variables(
        &self,
        widget_builder: &dyn WidgetBuilder,
        widget_stmt: &TokenStream,
        var_name: &Name,
        var_stream: &mut TokenStream,
    ) {
        let (widget_ident, value_ident, handle_ident) = gen_idents();
        let mut update_stream = TokenStream::new();

        for (prop, _var) in self.0.iter().filter(|(_p, v)| v == var_name) {
            widget_builder.on_property_update(
                prop,
                &widget_ident,
                &value_ident,
                &handle_ident,
                &mut update_stream,
            );
        }

        if !update_stream.is_empty() {
            var_stream.extend(quote!(let widget = #widget_stmt; #update_stream));
        }
    }
}
