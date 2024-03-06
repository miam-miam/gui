pub use crate::handles::{EventHandle, Handle, RenderHandle, ResizeHandle, UpdateHandle};
use crate::layout::LayoutConstraints;
use crate::parse::fluent::Fluent;
use crate::parse::var::Name;
use crate::parse::WidgetDeclaration;
use crate::ToComponent;
use dyn_clone::DynClone;
use glazier::kurbo::Point;
use glazier::PointerEvent;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::any::Any;
use vello::kurbo::Size;
use vello::SceneBuilder;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct WidgetID {
    component_id: u32,
    widget_id: u32,
}

impl ToTokens for WidgetID {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let component_id = self.component_id;
        let widget_id = self.widget_id;
        tokens.extend(quote!(WidgetID::new(#component_id, #widget_id)))
    }
}

impl WidgetID {
    pub const fn new(component_id: u32, widget_id: u32) -> Self {
        WidgetID {
            component_id,
            widget_id,
        }
    }

    pub fn widget_id(&self) -> u32 {
        self.widget_id
    }

    pub fn component_id(&self) -> u32 {
        self.component_id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetEvent<'a> {
    PointerUp(&'a PointerEvent),
    PointerDown(&'a PointerEvent),
    PointerMove(&'a PointerEvent),
    HoverChange,
    ActiveChange,
}

impl<'a> WidgetEvent<'a> {
    pub fn get_point(&self) -> Option<Point> {
        match self {
            WidgetEvent::PointerUp(e)
            | WidgetEvent::PointerDown(e)
            | WidgetEvent::PointerMove(e) => Some(e.pos),
            _ => None,
        }
    }
}

pub trait Widget<T: ToComponent> {
    fn id(&self) -> WidgetID;
    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<T>);
    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<T>) -> Size;
    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<T>);
}

impl<T: ToComponent> Widget<T> for WidgetID {
    fn id(&self) -> WidgetID {
        *self
    }
    fn render(&mut self, _scene: &mut SceneBuilder, _handle: &mut RenderHandle<T>) {}
    fn resize(&mut self, constraints: LayoutConstraints, _handle: &mut ResizeHandle<T>) -> Size {
        constraints.get_min()
    }
    fn event(&mut self, _event: WidgetEvent, _handle: &mut EventHandle<T>) {}
}

/// Helper trait to enable trait upcasting, since upcasting is not stable.
pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &(dyn Any) {
        self as &dyn Any
    }
}

#[typetag::deserialize(tag = "widget", content = "properties", deny_unknown_fields)]
pub trait WidgetBuilder: std::fmt::Debug + AsAny + DynClone {
    fn widget_type(
        &self,
        handler: Option<&Ident>,
        component: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    );
    fn name(&self) -> &'static str;
    fn combine(&mut self, rhs: &dyn WidgetBuilder);
    fn create_widget(&self, id: WidgetID, widget: Option<&TokenStream>, stream: &mut TokenStream);
    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        handle: &Ident,
        stream: &mut TokenStream,
    );
    fn get_statics(&self) -> Vec<(&'static str, TokenStream)>;
    fn get_fluents(&self) -> Vec<(&'static str, Fluent)>;
    fn get_vars(&self) -> Vec<(&'static str, Name)>;
    fn has_handler(&self) -> bool;
    fn get_widgets(&mut self) -> Option<Vec<&mut WidgetDeclaration>>;
    fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>>;
}

dyn_clone::clone_trait_object!(WidgetBuilder);
