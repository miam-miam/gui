pub use crate::handles::{EventHandle, Handle, RenderHandle, ResizeHandle, UpdateHandle};
use crate::layout::LayoutConstraints;
use crate::parse::fluent::Fluent;
use crate::parse::var::{ComponentVar, Name};
use crate::parse::WidgetDeclaration;
use crate::ToComponent;
use dyn_clone::DynClone;
use glazier::kurbo::Point;
use glazier::PointerEvent;
use itertools::Either;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::any::Any;
use std::slice::IterMut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use vello::kurbo::Size;
use vello::SceneBuilder;

/// A unique ID to reference a component, each instantiation increments the ID by 1.
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

/// An ID giving to all widgets to uniquely identify them in their component's namespace.
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

    /// Increments the Widget ID based on a given component ID.
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

/// Framework events a widget could respond to.
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
    /// The id of the widget
    fn id(&self) -> WidgetID;
    /// render the widget. It is up to the widget to call the render handle with any widget children that should be rendered.
    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<T>);
    /// resize the current widget, the selected [`Size`] must be adhered to, it is up to the widget
    /// to call `resize` on its children and tell the `handle` the child's position in the widget's local position.
    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<T>) -> Size;
    /// propagate the WidgetEvent use the [`EventHandle`] to decide how the event should be propagated to children.
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

pub type MutMultiWidget<'a> = SingleOrMulti<&'a mut WidgetDeclaration>;
pub type MultiWidget<'a> = SingleOrMulti<&'a WidgetDeclaration>;

#[derive(Clone, Debug)]
pub enum SingleOrMulti<T> {
    Single(T),
    Multi(Vec<T>),
}

impl<T> SingleOrMulti<T> {
    pub fn single(&self) -> Option<&T> {
        if let SingleOrMulti::Single(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn multi(&self) -> Option<&Vec<T>> {
        if let SingleOrMulti::Multi(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        match self {
            SingleOrMulti::Single(_) => 1,
            SingleOrMulti::Multi(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.multi().is_some_and(|v| v.is_empty())
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> SingleOrMulti<U> {
        match self {
            SingleOrMulti::Single(s) => SingleOrMulti::Single(f(s)),
            SingleOrMulti::Multi(m) => SingleOrMulti::Multi(m.into_iter().map(f).collect()),
        }
    }

    pub fn try_map<U, E, F: FnMut(T) -> Result<U, E>>(
        self,
        mut f: F,
    ) -> Result<SingleOrMulti<U>, E> {
        Ok(match self {
            SingleOrMulti::Single(s) => SingleOrMulti::Single(f(s)?),
            SingleOrMulti::Multi(m) => {
                SingleOrMulti::Multi(m.into_iter().map(f).collect::<Result<Vec<_>, E>>()?)
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.single()
            .into_iter()
            .chain(self.multi().into_iter().flat_map(|m| m.iter()))
    }

    pub fn iter_mut(&mut self) -> SingleOrMultiIterMut<T> {
        let either = match self {
            SingleOrMulti::Single(s) => Either::Left(Some(s)),
            SingleOrMulti::Multi(m) => Either::Right(m.iter_mut()),
        };
        SingleOrMultiIterMut(either)
    }
}

pub struct SingleOrMultiIterMut<'a, T>(Either<Option<&'a mut T>, IterMut<'a, T>>);

impl<'a, T> Iterator for SingleOrMultiIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Either::Left(l) => l.take(),
            Either::Right(r) => r.next(),
        }
    }
}

/// Trait used to create and define widgets. All the implemented functions will only be run once
/// so must only depend on the state of the WidgetBuilder.
#[typetag::deserialize(tag = "widget", content = "properties", deny_unknown_fields)]
pub trait WidgetBuilder: std::fmt::Debug + AsAny + DynClone {
    /// The type of the widget.
    fn widget_type(
        &self,
        handler: Option<&Ident>,
        component: &Ident,
        child: Option<&TokenStream>,
        stream: &mut TokenStream,
    );
    /// The name of the widget, used in diagnostics. Normally matches the tag name.
    fn name(&self) -> &'static str;
    /// Used to combine style properties and state properties into a single component.
    /// Will always match the type of the current WidgetBuilder.
    fn combine(&mut self, rhs: &dyn WidgetBuilder);
    /// [`TokenStream`] to create the widget, passing in the id and children.
    /// The `children` [`TokenStream`] is of the form `<child1>, <child2>, <child3>`.
    fn create_widget(&self, id: WidgetID, stream: &mut TokenStream);
    /// Function to be called when a `property` is updated.
    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        handle: &Ident,
        stream: &mut TokenStream,
    );
    /// The value of a given static and the property it relates to.
    fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
        vec![]
    }
    /// The fluent and the property it is attached to.
    fn get_fluents(&self) -> Vec<(&'static str, Fluent)> {
        vec![]
    }
    /// The variables and property they are attached to.
    fn get_vars(&self) -> Vec<(&'static str, Name)> {
        vec![]
    }
    /// The components and property the widget holds.
    fn get_components(&self) -> Vec<(&'static str, ComponentVar)> {
        vec![]
    }
    /// Indicates whether this widget has a handler.
    fn has_handler(&self) -> bool {
        false
    }
    /// Return [`WidgetDeclaration`]s for each child stored in the widget.
    /// None indicates that this widget does not normally store children
    fn get_widgets(&mut self) -> Option<Vec<MutMultiWidget>> {
        None
    }
    /// Return the [`TokenStream`] needed to access the given child widgets from runtime widget.
    /// None indicates that this widget does not normally store children
    fn widgets(&self) -> Option<Vec<(TokenStream, MultiWidget)>> {
        None
    }
}

// Allows the WidgetBuilder trait objects to be cloned into boxes.
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
