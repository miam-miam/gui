mod common;
mod iter;
mod overridden_widget;
mod widget_set;

use crate::fluent::FluentIdent;
use crate::widget::common::{Components, Fluents, Statics, Variables};
use crate::widget::overridden_widget::WidgetProperties;
use anyhow::anyhow;
use gui_core::parse::{
    ComponentDeclaration, NormalVariableDeclaration, StateDeclaration, WidgetDeclaration,
};
use gui_core::widget::WidgetID;
use iter::WidgetIter;
use itertools::Itertools;
use overridden_widget::OverriddenWidget;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicU32, Ordering};
use widget_set::WidgetSet;

#[derive(Clone, Debug)]
pub struct Widget<'a> {
    pub widget_type_name: &'static str,
    pub widget_declaration: &'a WidgetDeclaration,
    pub state_overrides: Vec<OverriddenWidget<'a>>,
    pub fully_state_overridden: bool,
    pub shared_overrides: WidgetProperties,
    pub fallback: WidgetProperties,
    pub child_widgets: Option<WidgetSet<'a>>,
    pub child_type: Option<Ident>,
    pub handler: Option<Ident>,
    pub components: Components,
    pub id: WidgetID,
}

impl<'a> Widget<'a> {
    pub fn new(component: &'a ComponentDeclaration) -> anyhow::Result<Self> {
        static COMPONENT_COUNTER: AtomicU32 = AtomicU32::new(0);
        Self::new_inner(
            component.name.as_str(),
            &component.child,
            &component.states[..],
            COMPONENT_COUNTER.fetch_add(1, Ordering::Relaxed),
        )
    }
    pub fn new_inner(
        component_name: &str,
        widget_declaration: &'a WidgetDeclaration,
        states: &'a [StateDeclaration],
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
        let fluents = Fluents::new(
            widget,
            component_name,
            widget_declaration.name.as_deref(),
            widget_type_name,
        );

        let id = WidgetID::next(component_id);
        let mut state_overrides =
            OverriddenWidget::new(component_name, widget_declaration, states)?;
        let shared_overrides = WidgetProperties::remove_common_properties(&mut state_overrides[..]);
        Ok(Self {
            widget_type_name,
            widget_declaration,
            components: Components::new(widget),
            child_widgets: widget
                .widgets()
                .map(|ws| WidgetSet::new(component_name, ws, states, component_id))
                .transpose()?,
            child_type: None,
            fully_state_overridden: !state_overrides.is_empty()
                && state_overrides.len() == states.len(),
            handler,
            state_overrides,
            fallback: WidgetProperties {
                statics: Statics::new(widget),
                fluents,
                variables: Variables(widget.get_vars()),
            },
            id,
            shared_overrides,
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

    pub fn push_fluents(&'a self, container: &mut Vec<FluentIdent>) {
        container.extend_from_slice(&self.fallback.fluents.0[..]);
        for ow in &self.state_overrides {
            container.extend_from_slice(&ow.state_fluent_overrides.0[..]);
        }
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
        self.gen_if_correct_state(stream, |var_stream| {
            self.fallback.variables.gen_variables(
                &*self.widget_declaration.widget,
                widget_stmt,
                &var.name,
                var_stream,
            );
            self.fallback
                .fluents
                .gen_fluent_arg_update(&var.name, var_stream);
        });

        self.gen_shared_overrides(stream, |var_stream| {
            self.shared_overrides.variables.gen_variables(
                &*self.widget_declaration.widget,
                widget_stmt,
                &var.name,
                var_stream,
            );
            self.shared_overrides
                .fluents
                .gen_fluent_arg_update(&var.name, var_stream);
        });

        for widget in &self.state_overrides {
            widget.gen_if_correct_state(stream, |var_stream| {
                widget.variables.gen_variables(
                    &*self.widget_declaration.widget,
                    widget_stmt,
                    &var.name,
                    var_stream,
                );
                widget.fluents.gen_fluent_arg_update(&var.name, var_stream);
                widget
                    .state_fluent_overrides
                    .gen_fluent_arg_update(&var.name, var_stream);
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

        self.gen_if_correct_state(stream, |fluent_stream| {
            self.fallback.fluents.gen_fluents(
                &*self.widget_declaration.widget,
                &widget_stmt,
                fluent_stream,
            )
        });

        self.gen_shared_overrides(stream, |fluent_stream| {
            self.shared_overrides.fluents.gen_fluents(
                &*self.widget_declaration.widget,
                &widget_stmt,
                fluent_stream,
            )
        });

        for widget in &self.state_overrides {
            widget.gen_if_correct_state(stream, |state_stream| {
                widget.fluents.gen_fluents(
                    &*self.widget_declaration.widget,
                    &widget_stmt,
                    state_stream,
                );
                widget.state_fluent_overrides.gen_fluents(
                    &*self.widget_declaration.widget,
                    &widget_stmt,
                    state_stream,
                )
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

    fn gen_shared_overrides(&self, stream: &mut TokenStream, func: impl FnOnce(&mut TokenStream)) {
        if self.fully_state_overridden || self.state_overrides.is_empty() {
            func(stream)
        } else {
            let idents = self
                .state_overrides
                .iter()
                .map(|o| format_ident!("{}", o.state_name));
            let mut overrides = TokenStream::new();

            func(&mut overrides);

            if overrides.is_empty() {
                return;
            }

            stream.extend(quote! {
                if #(self.state == State::#idents)||* {
                    #overrides
                }
            });
        }
    }

    fn gen_if_correct_state(&self, stream: &mut TokenStream, func: impl FnOnce(&mut TokenStream)) {
        if self.fully_state_overridden {
            return;
        }

        let mut func_stream = TokenStream::new();
        func(&mut func_stream);
        if func_stream.is_empty() {
            return;
        }

        if self.state_overrides.is_empty() {
            stream.extend(func_stream);
            return;
        }

        let idents = self
            .state_overrides
            .iter()
            .map(|o| format_ident!("{}", o.state_name));

        stream.extend(quote! {
            if #(self.state != State::#idents)&&* {
                #func_stream
            }
        })
    }

    pub fn gen_statics(&self, widget_stmt: Option<&TokenStream>, stream: &mut TokenStream) {
        let widget_stmt = widget_stmt.map_or_else(|| quote! {&mut self.widget}, Clone::clone);

        self.gen_if_correct_state(stream, |static_stream| {
            self.fallback.statics.gen_statics(
                &*self.widget_declaration.widget,
                &widget_stmt,
                static_stream,
            )
        });

        self.gen_shared_overrides(stream, |static_stream| {
            self.shared_overrides.statics.gen_statics(
                &*self.widget_declaration.widget,
                &widget_stmt,
                static_stream,
            );
        });

        self.components
            .gen_components(&*self.widget_declaration.widget, &widget_stmt, stream);

        for widget in &self.state_overrides {
            widget.gen_if_correct_state(stream, |static_stream| {
                widget.statics.gen_statics(
                    &*self.widget_declaration.widget,
                    &widget_stmt,
                    static_stream,
                )
            });
        }

        if let Some(ws) = &self.child_widgets {
            for (get_stmt, w) in ws.gen_widget_gets(&widget_stmt) {
                w.gen_statics(Some(&get_stmt), stream);
            }
        }
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

    pub fn iter<'b>(&'b self) -> WidgetIter<'a, 'b> {
        WidgetIter::new(self)
    }
}
