use gui_core::glazier::kurbo::Rect;
use gui_core::parse::fluent::Fluent;
use gui_core::parse::WidgetDeclaration;
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, Widget, WidgetBuilder, WidgetEvent, WidgetID,
};
use gui_core::{LayoutConstraints, Point, SceneBuilder, Size, ToComponent, Var};
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::marker::PhantomData;

enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn to_size(&self, value: f64) -> Size {
        match self {
            Axis::Horizontal => Size::new(value, 0.0),
            Axis::Vertical => Size::new(0.0, value),
        }
    }

    fn get_axis(&self, size: Size) -> f64 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn invert(&self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

pub struct HVStack<C: ToComponent, W: Widget<C>> {
    id: WidgetID,
    axis: Axis,
    spacing: f32,
    children: Vec<W>,
    phantom: PhantomData<C>,
}

impl<C: ToComponent, W: Widget<C>> HVStack<C, W> {
    pub fn new_horizontal(id: WidgetID, spacing: f32, children: Vec<W>) -> Self {
        Self {
            id,
            axis: Axis::Horizontal,
            spacing,
            children,
            phantom: PhantomData,
        }
    }

    pub fn new_vertical(id: WidgetID, spacing: f32, children: Vec<W>) -> Self {
        Self {
            id,
            axis: Axis::Vertical,
            spacing,
            children,
            phantom: PhantomData,
        }
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }

    pub fn widgets(&mut self, i: usize) -> &mut W {
        self.children.get_mut(i).unwrap()
    }
}

impl<C: ToComponent, W: Widget<C>> Widget<C> for HVStack<C, W> {
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<C>) {
        handle.render_widgets(scene, self.children.iter_mut());
    }
    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        let child_length = self.children.len();
        let total_spacing = self.spacing as f64 * (child_length - 1) as f64;
        let mut remaining = constraints.map(|s| s - self.axis.to_size(total_spacing));

        let layouts =
            self.children
                .iter_mut()
                .enumerate()
                .map(|(i, child)| {
                    let allocated_space = match self.axis {
                        Axis::Horizontal => remaining
                            .map(|s| Size::new(s.width / (child_length - i) as f64, s.height)),
                        Axis::Vertical => remaining
                            .map(|s| Size::new(s.width, s.height / (child_length - i) as f64)),
                    };
                    let size = child.resize(allocated_space, handle);
                    remaining = remaining.map(|s| s - self.axis.to_size(self.axis.get_axis(size)));
                    (size, child.id())
                })
                .collect_vec();

        let max_length = layouts
            .iter()
            .map(|(s, _)| self.axis.invert().get_axis(*s))
            .reduce(f64::max)
            .unwrap_or_default();
        let mut acc = 0.0;

        for (s, id) in layouts.iter().copied() {
            let pos = match self.axis {
                Axis::Horizontal => {
                    Point::new(acc, (max_length - self.axis.invert().get_axis(s)) / 2.0)
                }
                Axis::Vertical => {
                    Point::new((max_length - self.axis.invert().get_axis(s)) / 2.0, acc)
                }
            };
            acc += self.axis.get_axis(s) + self.spacing as f64;
            handle.position_widget(Rect::from_origin_size(pos, s), id)
        }

        match self.axis {
            Axis::Horizontal => Size::new(
                Itertools::intersperse(layouts.iter().map(|(s, _)| s.width), self.spacing as f64)
                    .sum(),
                max_length,
            ),
            Axis::Vertical => Size::new(
                max_length,
                Itertools::intersperse(layouts.iter().map(|(s, _)| s.height), self.spacing as f64)
                    .sum(),
            ),
        }
    }

    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<C>) {
        handle.propagate_event(event, self.children.iter_mut())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HStackBuilder {
    spacing: Option<Var<f32>>,
    #[serde(default)]
    children: Vec<WidgetDeclaration>,
}

#[typetag::deserialize(name = "HStack")]
impl WidgetBuilder for HStackBuilder {
    fn widget_type(
        &self,
        _handler: Option<&Ident>,
        comp_struct: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::HVStack<#comp_struct, #widget>));
    }

    fn name(&self) -> &'static str {
        "HStack"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.spacing {
                self.spacing.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, id: WidgetID, widget: Option<&TokenStream>, stream: &mut TokenStream) {
        let spacing = match &self.spacing {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => 0_10f32.to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::HVStack::new_horizontal(#id, #spacing, vec![#widget])
        });
    }

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        stream: &mut TokenStream,
    ) {
        #[allow(clippy::single_match)]
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

    fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>> {
        Some(
            self.children
                .iter()
                .enumerate()
                .map(|(i, c)| (quote!(.widgets(#i)), c))
                .collect(),
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct VStackBuilder {
    spacing: Option<Var<f32>>,
    #[serde(default)]
    children: Vec<WidgetDeclaration>,
}

#[typetag::deserialize(name = "VStack")]
impl WidgetBuilder for VStackBuilder {
    fn widget_type(
        &self,
        _handler: Option<&Ident>,
        comp_struct: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::HVStack<#comp_struct, #widget>));
    }

    fn name(&self) -> &'static str {
        "HStack"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.spacing {
                self.spacing.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, id: WidgetID, widget: Option<&TokenStream>, stream: &mut TokenStream) {
        let spacing = match &self.spacing {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => 0_10f32.to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::HVStack::new_vertical(#id, #spacing, vec![#widget])
        });
    }

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        stream: &mut TokenStream,
    ) {
        #[allow(clippy::single_match)]
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

    fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>> {
        Some(
            self.children
                .iter()
                .enumerate()
                .map(|(i, c)| (quote!(.widgets(#i)), c))
                .collect(),
        )
    }
}
