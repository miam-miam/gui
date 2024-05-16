use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;

use gui_core::{
    Children, LayoutConstraints, MutWidgetChildren, Point, SceneBuilder, Size, ToComponent,
    Var, WidgetChildren,
};
use gui_core::glazier::kurbo::Rect;
use gui_core::parse::var::Name;
use gui_core::parse::WidgetDeclaration;
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetBuilder, WidgetEvent,
    WidgetID,
};

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

pub struct HVStack<W> {
    id: WidgetID,
    axis: Axis,
    spacing: f32,
    children: Vec<W>,
}

impl<W> HVStack<W> {
    pub fn new_horizontal(id: WidgetID) -> Self {
        Self {
            id,
            axis: Axis::Horizontal,
            spacing: Default::default(),
            children: vec![],
        }
    }

    pub fn new_vertical(id: WidgetID) -> Self {
        Self {
            id,
            axis: Axis::Vertical,
            spacing: Default::default(),
            children: vec![],
        }
    }

    pub fn set_spacing(&mut self, spacing: f32, handle: &mut UpdateHandle) {
        self.spacing = spacing;
        handle.invalidate_id(self.id);
    }

    pub fn widgets(&mut self) -> &mut Vec<W> {
        &mut self.children
    }
}

impl<C: ToComponent, W: Widget<C>> Widget<C> for HVStack<W> {
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
        _comp_struct: &Ident,
        children: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::HVStack<#children>));
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

    fn create_widget(&self, id: WidgetID, stream: &mut TokenStream) {
        stream.extend(quote! {
            ::gui::gui_widget::HVStack::new_horizontal(#id)
        });
    }

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        handle: &Ident,
        stream: &mut TokenStream,
    ) {
        #[allow(clippy::single_match)]
        match property {
            "spacing" => stream.extend(quote! {#widget.set_spacing(#value, #handle);}),
            _ => {}
        }
    }

    fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
        let mut array = vec![];
        match &self.spacing {
            Some(Var::Value(v)) => array.push(("spacing", v.to_token_stream())),
            None => array.push(("spacing", 0_10f32.to_token_stream())),
            _ => {}
        };
        array
    }

    fn get_vars(&self) -> Vec<(&'static str, Name)> {
        let mut array = vec![];
        if let Some(Var::Variable(v)) = &self.spacing {
            array.push(("spacing", v.clone()));
        }
        array
    }

    fn get_widgets(&mut self) -> Option<Vec<MutWidgetChildren>> {
        let mut array = vec![];
        if !self.children.is_empty() {
            array.push(Children::Many(self.children.iter_mut().collect()));
        }
        Some(array)
    }

    fn widgets(&self) -> Option<Vec<(TokenStream, WidgetChildren)>> {
        let mut array = vec![];
        if !self.children.is_empty() {
            array.push((
                quote!(.widgets()),
                Children::Many(self.children.iter().collect()),
            ));
        }
        Some(array)
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
        _comp_struct: &Ident,
        children: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::HVStack<#children>));
    }

    fn name(&self) -> &'static str {
        "VStack"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.spacing {
                self.spacing.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, id: WidgetID, stream: &mut TokenStream) {
        stream.extend(quote! {
            ::gui::gui_widget::HVStack::new_vertical(#id)
        });
    }

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        handle: &Ident,
        stream: &mut TokenStream,
    ) {
        #[allow(clippy::single_match)]
        match property {
            "spacing" => stream.extend(quote! {#widget.set_spacing(#value, #handle);}),
            _ => {}
        }
    }

    fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
        let mut array = vec![];
        match &self.spacing {
            Some(Var::Value(v)) => array.push(("spacing", v.to_token_stream())),
            None => array.push(("spacing", 0_10f32.to_token_stream())),
            _ => {}
        };
        array
    }

    fn get_vars(&self) -> Vec<(&'static str, Name)> {
        let mut array = vec![];
        if let Some(Var::Variable(v)) = &self.spacing {
            array.push(("spacing", v.clone()));
        }
        array
    }

    fn get_widgets(&mut self) -> Option<Vec<MutWidgetChildren>> {
        let mut array = vec![];
        if !self.children.is_empty() {
            array.push(Children::Many(self.children.iter_mut().collect()));
        }
        Some(array)
    }

    fn widgets(&self) -> Option<Vec<(TokenStream, WidgetChildren)>> {
        let mut array = vec![];
        if !self.children.is_empty() {
            array.push((
                quote!(.widgets()),
                Children::Many(self.children.iter().collect()),
            ));
        }
        Some(array)
    }
}
