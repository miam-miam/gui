use crate::fluent::FluentIdent;
use crate::widget::common::{Fluents, Statics, Variables};
use anyhow::{anyhow, bail};
use gui_core::parse::var::Name;
use gui_core::parse::{StateDeclaration, WidgetDeclaration};
use gui_core::widget::WidgetBuilder;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Debug, Default)]
pub struct WidgetProperties {
    pub statics: Statics,
    pub fluents: Fluents,
    pub variables: Variables,
}

impl WidgetProperties {
    fn get_intersection<'a, T: Eq + Hash + Clone + 'a>(
        iter: impl Iterator<Item = HashSet<&'a T>>,
    ) -> HashSet<T> {
        iter.reduce(|acc, w| acc.intersection(&w).copied().collect())
            .map_or_else(HashSet::default, |h| {
                h.into_iter().cloned().collect::<HashSet<T>>()
            })
    }

    /// Removes properties that are shared by all the given `widgets`.
    #[allow(clippy::mutable_key_type)]
    pub fn remove_common_properties(widgets: &mut [OverriddenWidget]) -> Self {
        let common_statics =
            Self::get_intersection(widgets.iter().map(|w| w.statics.0.iter().collect()));

        let common_fluents =
            Self::get_intersection(widgets.iter().map(|w| w.fluents.0.iter().collect()));

        let common_variables =
            Self::get_intersection(widgets.iter().map(|w| w.variables.0.iter().collect()));

        for w in widgets.iter_mut() {
            w.statics.0.retain(|s| !common_statics.contains(s));
            w.fluents.0.retain(|f| !common_fluents.contains(f));
            w.variables.0.retain(|v| !common_variables.contains(v));
        }
        // Sort by property to prevent generated code from changing on each compile
        Self {
            statics: Statics(common_statics.into_iter().sorted_by_key(|s| s.0).collect()),
            fluents: Fluents(
                common_fluents
                    .into_iter()
                    .sorted_by_key(|f| f.property)
                    .collect(),
            ),
            variables: Variables(
                common_variables
                    .into_iter()
                    .sorted_by_key(|v| v.0)
                    .collect(),
            ),
        }
    }
}

/// Struct holding the specific property overrides for each state.
#[derive(Clone, Debug, Default)]
pub struct OverriddenWidget<'a> {
    pub state_name: &'a str,
    pub statics: Statics,
    pub fluents: Fluents,
    pub state_fluent_overrides: Fluents,
    pub variables: Variables,
}

impl<'a> OverriddenWidget<'a> {
    pub fn new(
        component_name: &str,
        widget_declaration: &'a WidgetDeclaration,
        states: &'a [StateDeclaration],
    ) -> anyhow::Result<Vec<Self>> {
        let mut result = vec![];
        if let Some(widget_name) = widget_declaration.name.as_ref().map(Name::as_str) {
            for state in states {
                let state_name = &state.name;
                if let Some(state_override) = state
                    .overrides
                    .iter()
                    .filter(|w| &*w.name == widget_name)
                    .at_most_one()
                    .map_err(|_| {
                        anyhow!("Can only override widget {widget_name} once in {state_name}.")
                    })?
                {
                    let mut new_widget = state_override.widget.clone();
                    if new_widget.widgets().is_some_and(|v| !v.is_empty()) {
                        bail!("Overridden widget {widget_name} in {state_name} contains children.");
                    }
                    let widget_builder = widget_declaration.widget.as_ref();
                    let fluents = Fluents::new_state_override(
                        &*new_widget,
                        component_name,
                        widget_name,
                        state_name,
                    );
                    new_widget.combine(widget_builder);
                    result.push(Self::new_inner(
                        state_name,
                        widget_name,
                        component_name,
                        fluents,
                        &*new_widget,
                    ));
                }
            }
        }
        Ok(result)
    }

    fn new_inner<'b>(
        state_name: &'a str,
        widget_name: &str,
        component_name: &str,
        state_fluent_overrides: Fluents,
        builder: &'b (dyn WidgetBuilder + 'static),
    ) -> Self {
        let statics = Statics::new(builder);
        let fluents = Fluents(
            builder
                .get_fluents()
                .into_iter()
                .filter(|(prop, fluent)| {
                    !state_fluent_overrides
                        .0
                        .iter()
                        .map(|f| (&f.property, &f.fluent))
                        .contains(&(prop, fluent))
                })
                .map(|(prop, fluent)| {
                    FluentIdent::new(
                        prop,
                        fluent,
                        component_name,
                        Some(widget_name),
                        "", // Can keep as "" as it is only used if widget_name is None.
                    )
                })
                .collect(),
        );
        let variables = Variables(builder.get_vars());

        Self {
            state_name,
            statics,
            fluents,
            state_fluent_overrides,
            variables,
        }
    }

    pub fn gen_if_correct_state(
        &self,
        stream: &mut TokenStream,
        func: impl FnOnce(&mut TokenStream),
    ) {
        let mut func_stream = TokenStream::new();
        func(&mut func_stream);
        if func_stream.is_empty() {
            return;
        }

        let ident = format_ident!("{}", self.state_name);

        stream.extend(quote! {
            if self.state == State::#ident {
                #func_stream
            }
        })
    }
}
