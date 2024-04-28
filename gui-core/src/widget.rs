pub use crate::handles::{EventHandle, Handle, RenderHandle, ResizeHandle, UpdateHandle};
use crate::layout::LayoutConstraints;
use crate::parse::fluent::Fluent;
use crate::parse::var::{ComponentVar, Name};
use crate::parse::WidgetDeclaration;
use crate::ToComponent;
use dyn_clone::DynClone;
use glazier::kurbo::Point;
use glazier::PointerEvent;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::any::Any;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use vello::kurbo::Size;
use vello::SceneBuilder;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Hash)]
pub struct RuntimeID(u32);

impl ToTokens for RuntimeID {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let runtime_id = self.0;
        tokens.extend(quote!(RuntimeID::new(#runtime_id)))
    }
}

impl RuntimeID {
    pub const fn new(runtime_id: u32) -> Self {
        RuntimeID(runtime_id)
    }
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn next() -> Self {
        static WIDGET_COUNTER: AtomicU32 = AtomicU32::new(0);
        RuntimeID(WIDGET_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Hash)]
pub struct WidgetID(u32);

impl ToTokens for WidgetID {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let widget_id = self.0;
        tokens.extend(quote!(WidgetID::new(#widget_id)))
    }
}

impl WidgetID {
    pub const fn new(widget_id: u32) -> Self {
        WidgetID(widget_id)
    }
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn next(component_id: u32) -> Self {
        static WIDGET_COUNTER: Mutex<Vec<u32>> = Mutex::new(vec![]);
        let mut array = WIDGET_COUNTER.lock().expect("Mutex is not poisoned");
        if array.len() <= component_id as usize {
            array.resize(component_id as usize + 1, 0);
        }
        let id = array[component_id as usize];
        array[component_id as usize] += 1;
        WidgetID(id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetEvent<'a> {
    PointerUp(&'a PointerEvent),
    PointerDown(&'a PointerEvent),
    PointerMove(&'a PointerEvent),
    /// Sent to all widgets that are no longer being hovered over
    HoverChange,
    /// Sent to the active widget if a new widget is now active
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
        child: Option<&TokenStream>,
        stream: &mut TokenStream,
    );
    fn name(&self) -> &'static str;
    fn combine(&mut self, rhs: &dyn WidgetBuilder);
    fn create_widget(&self, id: WidgetID, children: Option<&TokenStream>, stream: &mut TokenStream);
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
    fn get_components(&self) -> Vec<(&'static str, ComponentVar)> {
        vec![]
    }
    fn has_handler(&self) -> bool {
        false
    }
    fn get_widgets(&mut self) -> Option<Vec<&mut WidgetDeclaration>>;
    fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>>;
}

dyn_clone::clone_trait_object!(WidgetBuilder);

#[cfg(test)]
mod test {
    use crate::widget::{RuntimeID, WidgetID};

    #[test]
    pub fn test_runtime_id() {
        let first = RuntimeID::next();
        let second = RuntimeID::next();
        assert!(first.0 < second.0);
    }

    #[test]
    pub fn test_widget_id() {
        fn compare_ids(component_id: u32) {
            let first = WidgetID::next(component_id);
            let second = WidgetID::next(component_id);
            assert!(first.0 < second.0);
        }

        compare_ids(0);
        compare_ids(1);
        compare_ids(2);
    }
}
