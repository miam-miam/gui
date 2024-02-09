use crate::fluent::FluentIdent;
use anyhow::anyhow;
use gui_core::parse::{ComponentDeclaration, NormalVariableDeclaration, WidgetDeclaration};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Debug)]
pub struct WidgetSet<'a> {
    pub widgets: Vec<(TokenStream, Widget<'a>)>,
    pub count: Option<u32>,
}

impl<'a> WidgetSet<'a> {
    pub fn new(
        component_name: &str,
        widgets: Vec<(TokenStream, &'a WidgetDeclaration)>,
    ) -> anyhow::Result<Self> {
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let widgets = widgets
            .into_iter()
            .map(|(s, w)| Ok((s, Widget::new_inner(component_name, w)?)))
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            count: (widgets.len() > 1).then(|| COUNTER.fetch_add(1, Ordering::Relaxed)),
            widgets,
        })
    }

    pub fn gen_widget_type(&self) -> TokenStream {
        let comp_struct = Ident::new("CompStruct", Span::call_site());
        match &self.widgets[..] {
            [(_, child)] => {
                let mut stream = TokenStream::new();
                child
                    .widget_declaration
                    .widget
                    .widget_type(None, &comp_struct, None, &mut stream);
                stream
            }
            [] => quote!(()),
            _ => {
                let count = self.count.expect("widget set should be created.");
                let ident = format_ident!("WidgetSet_{count}");
                quote!(#ident)
            }
        }
    }

    pub fn gen_widget_init(&self) -> TokenStream {
        match &self.widgets[..] {
            [(_, child)] => {
                let mut stream = TokenStream::new();
                child
                    .widget_declaration
                    .widget
                    .create_widget(None, &mut stream);
                stream
            }
            [] => quote!(()),
            _ => {
                let count = self.count.expect("widget set should be created.");
                let widget_set = format_ident!("WidgetSet_{count}");

                let inits = self.widgets.iter().map(|(_, child)| {
                    let mut stream = TokenStream::new();
                    child
                        .widget_declaration
                        .widget
                        .create_widget(None, &mut stream);
                    stream
                });

                let variants = self.widgets.iter().enumerate().map(|(i, _)| {
                    let ident = format_ident!("{i}");
                    quote!(#widget_set::#ident)
                });

                quote! {
                    #(#variants(#inits)),*
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Widget<'a> {
    pub widget_type_name: &'static str,
    pub widget_declaration: &'a WidgetDeclaration,
    pub child_widgets: Option<WidgetSet<'a>>,
    pub child_type: Option<Ident>,
    pub handler: Option<Ident>,
    pub fluents: Vec<FluentIdent<'a>>,
    pub variables: Vec<(&'static str, &'a str)>,
}

impl<'a> Widget<'a> {
    pub fn new(component: &'a ComponentDeclaration) -> anyhow::Result<Self> {
        Self::new_inner(component.name.as_str(), &component.child)
    }

    fn new_inner(
        component_name: &str,
        widget_declaration: &'a WidgetDeclaration,
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
        Ok(Self {
            widget_type_name,
            widget_declaration,
            child_widgets: widget
                .widgets(&Ident::new("widget", Span::call_site()))
                .map(|ws| WidgetSet::new(component_name, ws))
                .transpose()?,
            child_type: None,
            handler,
            fluents,
            variables: widget.get_vars(),
        })
    }

    pub fn gen_widget_type(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        let comp_struct = Ident::new("CompStruct", Span::call_site());
        let child_type = self.child_widgets.as_ref().map(WidgetSet::gen_widget_type);
        self.widget_declaration.widget.widget_type(
            self.handler.as_ref(),
            &comp_struct,
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
        let string_var_name = &var.name;

        stream.extend(quote!(let widget = #widget_stmt;));

        for (prop, _var) in self.variables.iter().filter(|(_p, v)| v == &var.name) {
            self.widget_declaration.widget.on_property_update(
                prop,
                &widget_ident,
                &value_ident,
                stream,
            );
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

        let mut widget_stmt = widget_stmt.clone();

        widget_stmt.extend(quote!(.get_widget()));

        for (_, w) in self.child_widgets.iter().flat_map(|s| &s.widgets) {
            w.gen_var_update2(var, &widget_stmt, stream);
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
        let mut widget_stmt = widget_stmt.map_or_else(|| quote! {&mut self.widget}, Clone::clone);
        let widget = Ident::new("widget", Span::call_site());
        let value = Ident::new("value", Span::call_site());

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

        widget_stmt.extend(quote!(.get_widget()));

        for (_, w) in self.child_widgets.iter().flat_map(|s| &s.widgets) {
            w.gen_fluent_update(Some(&widget_stmt), stream)
        }
    }

    pub fn gen_widget_init(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        let child_init = self.child_widgets.as_ref().map(WidgetSet::gen_widget_init);

        self.widget_declaration
            .widget
            .create_widget(child_init.as_ref(), &mut stream);
        stream
    }
}
