use crate::fluent::FluentIdent;
use crate::widget_set::WidgetSet;
use anyhow::anyhow;
use gui_core::parse::{ComponentDeclaration, NormalVariableDeclaration, WidgetDeclaration};
use gui_core::widget::WidgetID;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::cmp::max_by_key;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Debug)]
pub struct Widget<'a> {
    pub widget_type_name: &'static str,
    pub widget_declaration: &'a WidgetDeclaration,
    pub child_widgets: Option<WidgetSet<'a>>,
    pub child_type: Option<Ident>,
    pub handler: Option<Ident>,
    pub fluents: Vec<FluentIdent<'a>>,
    pub variables: Vec<(&'static str, &'a str)>,
    pub id: WidgetID,
}

impl<'a> Widget<'a> {
    pub fn new(component: &'a ComponentDeclaration) -> anyhow::Result<Self> {
        static COMPONENT_COUNTER: AtomicU32 = AtomicU32::new(0);
        Self::new_inner(
            component.name.as_str(),
            &component.child,
            COMPONENT_COUNTER.fetch_add(1, Ordering::Relaxed),
        )
    }

    pub fn new_inner(
        component_name: &str,
        widget_declaration: &'a WidgetDeclaration,
        component_id: u32,
    ) -> anyhow::Result<Self> {
        let widget = widget_declaration.widget.as_ref();
        let widget_type_name = widget.name();
        let handler = if widget.has_handler() {
            Some(Ident::new(
                widget_declaration
                    .name
                    .as_ref()
                    .ok_or_else(|| anyhow!("Widgets with handlers must be named."))?,
                Span::call_site(),
            ))
        } else {
            None
        };
        let fluents = widget
            .get_fluents()
            .into_iter()
            .map(|(prop, fluent)| {
                FluentIdent::new(
                    prop,
                    fluent,
                    component_name,
                    widget_declaration.name.as_deref(),
                    widget_type_name,
                )
            })
            .collect();

        static WIDGET_COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = WidgetID::new(component_id, WIDGET_COUNTER.fetch_add(1, Ordering::Relaxed));
        Ok(Self {
            widget_type_name,
            widget_declaration,
            child_widgets: widget
                .widgets()
                .map(|ws| WidgetSet::new(component_name, ws, component_id))
                .transpose()?,
            child_type: None,
            handler,
            fluents,
            variables: widget.get_vars(),
            id,
        })
    }

    pub fn gen_widget_type(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        let child_type = self.child_widgets.as_ref().map(|s| s.gen_widget_type());
        self.widget_declaration.widget.widget_type(
            self.handler.as_ref(),
            &format_ident!("CompStruct"),
            child_type.as_ref(),
            &mut stream,
        );
        stream
    }

    pub fn contains_fluents(&self) -> bool {
        !self.fluents.is_empty()
            || self
                .child_widgets
                .as_ref()
                .is_some_and(|s| s.widgets.iter().any(|(_, w)| w.contains_fluents()))
    }

    pub fn push_fluents(&'a self, container: &mut Vec<FluentIdent<'a>>) {
        container.extend_from_slice(&self.fluents[..]);
        for (_, child) in self.child_widgets.iter().flat_map(|s| &s.widgets) {
            child.push_fluents(container);
        }
    }

    fn gen_var_update2(
        &self,
        var: &NormalVariableDeclaration,
        widget_stmt: &TokenStream,
        stream: &mut TokenStream,
    ) {
        let widget_ident = Ident::new("widget", Span::call_site());
        let value_ident = Ident::new("value", Span::call_site());
        let handle_ident = Ident::new("handle_ref", Span::call_site());
        let string_var_name = &var.name;

        let mut update_stream = TokenStream::new();

        for (prop, _var) in self.variables.iter().filter(|(_p, v)| v == &var.name) {
            self.widget_declaration.widget.on_property_update(
                prop,
                &widget_ident,
                &value_ident,
                &handle_ident,
                &mut update_stream,
            );
        }

        if !update_stream.is_empty() {
            stream.extend(quote!(let widget = #widget_stmt; #update_stream));
        }

        for fluent in self
            .fluents
            .iter()
            .filter(|f| f.fluent.vars.contains(&var.name))
        {
            let fluent_ident = &fluent.ident;
            let prop = Ident::new(fluent.property, Span::call_site());
            stream.extend(quote! {
                #prop = true;
                self.#fluent_ident.set(#string_var_name, #value_ident);
            });
        }

        if let Some(ws) = &self.child_widgets {
            for (get_stmt, w) in ws.gen_widget_gets(widget_stmt) {
                w.gen_var_update2(var, &get_stmt, stream);
            }
        }
    }

    pub fn gen_var_update(&self, var: &NormalVariableDeclaration) -> TokenStream {
        let var_name = Ident::new(&var.name, Span::call_site());
        let mut stream = TokenStream::new();
        let widget = quote!(&mut self.widget);
        self.gen_var_update2(var, &widget, &mut stream);
        quote! {
            if force_update || <CompStruct as Update<#var_name>>::is_updated(&self.comp_struct) {
                let value = <CompStruct as Update<#var_name>>::value(&self.comp_struct);
                #stream
            }
        }
    }

    pub fn gen_fluent_update(&self, widget_stmt: Option<&TokenStream>, stream: &mut TokenStream) {
        let widget_stmt = widget_stmt.map_or_else(|| quote! {&mut self.widget}, Clone::clone);
        let widget = Ident::new("widget", Span::call_site());
        let value = Ident::new("value", Span::call_site());
        let handle_ident = Ident::new("handle_ref", Span::call_site());

        for fluent in &self.fluents {
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
            self.widget_declaration.widget.on_property_update(
                fluent.property,
                &widget,
                &value,
                &handle_ident,
                &mut on_property_update,
            );
            stream.extend(quote! {
                if force_update #(|| #property_iter)* {
                    let value = get_bundle_message(#fluent_name, #arg);
                    let #widget = #widget_stmt;
                    #on_property_update
                }
            });
        }

        if let Some(ws) = &self.child_widgets {
            for (get_stmt, w) in ws.gen_widget_gets(&widget_stmt) {
                w.gen_fluent_update(Some(&get_stmt), stream)
            }
        }
    }

    pub fn gen_widget_init(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        let child_init = self.child_widgets.as_ref().map(WidgetSet::gen_widget_init);

        self.widget_declaration
            .widget
            .create_widget(self.id, child_init.as_ref(), &mut stream);
        stream
    }

    pub fn gen_widget_set(&self, stream: &mut TokenStream) {
        if let Some(set) = &self.child_widgets {
            set.gen_widget_set(stream)
        }
    }

    pub fn get_largest_id(&self) -> WidgetID {
        max_by_key(
            self.id,
            self.child_widgets
                .as_ref()
                .and_then(|s| s.largest_id())
                .unwrap_or_default(),
            |i| i.widget_id(),
        )
    }

    pub fn gen_widget_id_to_widget(
        &self,
        widget_stmt: Option<&TokenStream>,
        acc: &mut Vec<(WidgetID, TokenStream)>,
    ) {
        let widget_stmt = widget_stmt.map_or_else(|| quote! {self.widget}, Clone::clone);

        if let Some(set) = &self.child_widgets {
            for (get_stmt, w) in set.gen_widget_gets(&widget_stmt) {
                w.gen_widget_id_to_widget(Some(&get_stmt), acc);
            }
        }

        acc.push((self.id, widget_stmt));
    }

    pub fn get_parent_ids(&self, acc: &mut Vec<(WidgetID, Vec<WidgetID>)>) {
        if let Some(set) = &self.child_widgets {
            let child_ids = set
                .widgets
                .iter()
                .map(|(_, w)| {
                    w.get_parent_ids(acc);
                    w.id
                })
                .collect_vec();
            acc.push((self.id, child_ids));
        }
    }

    pub fn gen_handler_structs(&self, stream: &mut TokenStream) -> anyhow::Result<()> {
        if let Some(name) = &self.handler {
            stream.extend(quote! {
                pub(crate) struct #name;

                impl ToHandler for #name {
                    type BaseHandler = CompStruct;
                }
            });
        }

        if let Some(ws) = &self.child_widgets {
            for (_, w) in &ws.widgets {
                w.gen_handler_structs(stream)?;
            }
        }

        Ok(())
    }
}
