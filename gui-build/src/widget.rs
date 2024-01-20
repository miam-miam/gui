use crate::fluent::FluentIdent;
use anyhow::anyhow;
use gui_core::parse::{ComponentDeclaration, WidgetDeclaration};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

#[derive(Clone, Debug)]
pub struct Widget<'a> {
    pub widget_type_name: &'static str,
    pub widget_declaration: &'a WidgetDeclaration,
    pub child_widgets: Vec<Option<Widget<'a>>>,
    pub child_type: Option<Ident>,
    pub handler: Option<Ident>,
    pub fluents: Vec<FluentIdent<'a>>,
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
                .get_widgets()
                .iter()
                .map(|w| {
                    Ok(match w {
                        Some(d) => Some(Self::new_inner(component_name, d)?),
                        None => None,
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
            child_type: None,
            handler,
            fluents,
        })
    }

    pub fn gen_widget_type(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        let comp_struct = Ident::new("CompStruct", Span::call_site());
        let child_type = match &self.child_widgets[..] {
            [Some(child)] => {
                let mut stream = TokenStream::new();
                child
                    .widget_declaration
                    .widget
                    .widget_type(None, &comp_struct, None, &mut stream);
                Some(stream)
            }
            _ => None,
        };
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
                .iter()
                .filter_map(|w| w.as_ref())
                .any(|w| w.contains_fluents())
    }

    pub fn push_fluents(&'a self, container: &mut Vec<FluentIdent<'a>>) {
        container.extend_from_slice(&self.fluents[..]);
        for child in self.child_widgets.iter().flatten() {
            child.push_fluents(container);
        }
    }

    pub fn gen_fluent_update(&self, widget_stmt: Option<&TokenStream>, stream: &mut TokenStream) {
        let mut widget_stmt = widget_stmt.map_or_else(|| quote! {&mut self.widget}, Clone::clone);
        let widget = Ident::new("widget", Span::call_site());
        let value = Ident::new("value", Span::call_site());

        for fluent in &self.fluents {
            let property_ident = &fluent.property_ident;
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
                if force_update || #property_ident {
                    let value = get_bundle_message(#fluent_name, #arg);
                    let #widget = #widget_stmt;
                    #on_property_update
                }
            });
        }

        widget_stmt.extend(quote!(.get_widget()));

        for w in self.child_widgets.iter().filter_map(|w| w.as_ref()) {
            w.gen_fluent_update(Some(&widget_stmt), stream)
        }
    }
}
