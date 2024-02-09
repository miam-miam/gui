use gui_core::parse::fluent::Fluent;
use gui_core::parse::WidgetDeclaration;
use gui_core::widget::{Widget, WidgetBuilder};
use gui_core::Var;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::marker::PhantomData;

pub struct HBox<C, W: Widget<C>> {
    spacing: f32,
    children: Vec<W>,
    phantom: PhantomData<C>,
}

impl<C, W: Widget<C>> HBox<C, W> {
    pub fn new(spacing: f32, children: Vec<W>) -> Self {
        Self {
            spacing,
            children,
            phantom: PhantomData,
        }
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }

    pub fn get_widgets(&mut self, i: usize) -> &mut W {
        self.children.get_mut(i).unwrap()
    }

    pub fn widgets(&self, i: usize) -> &W {
        self.children.get(i).unwrap()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HBoxBuilder {
    spacing: Option<Var<f32>>,
    #[serde(default)]
    children: Vec<WidgetDeclaration>,
}

#[typetag::deserialize(name = "HBox")]
impl WidgetBuilder for HBoxBuilder {
    fn widget_type(
        &self,
        _handler: Option<&Ident>,
        comp_struct: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::HBox<#comp_struct, #widget>));
    }

    fn name(&self) -> &'static str {
        "HBox"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.spacing {
                self.spacing.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, widget: Option<&TokenStream>, stream: &mut TokenStream) {
        let spacing = match &self.spacing {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => 0_10f32.to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::HBox::new(#spacing, vec![#widget])
        });
    }

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        stream: &mut TokenStream,
    ) {
        match property {
            "spacing" => stream.extend(quote! {#widget.set_disabled(#value);}),
            _ => {}
        }
    }

    fn get_fluents(&self) -> Vec<(&'static str, &Fluent)> {
        vec![]
    }

    fn get_vars(&self) -> Vec<(&'static str, &str)> {
        let mut array = vec![];
        if let Some(Var::Variable(v)) = &self.spacing {
            array.push(("spacing", v.as_str()));
        }
        array
    }

    fn has_handler(&self) -> bool {
        false
    }

    fn get_widgets(&mut self) -> Option<Vec<&mut WidgetDeclaration>> {
        Some(self.children.iter_mut().collect())
    }

    fn widgets(&self, widget: &Ident) -> Option<Vec<(TokenStream, &WidgetDeclaration)>> {
        Some(
            self.children
                .iter()
                .enumerate()
                .map(|(i, c)| (quote!(#widget.widgets(#i)), c))
                .collect(),
        )
    }
}
