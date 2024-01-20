use crate::parse::fluent::Fluent;
use crate::parse::WidgetDeclaration;
use dyn_clone::DynClone;
use parley::FontContext;
use proc_macro2::{Ident, TokenStream};
use std::any::Any;
use vello::SceneBuilder;

pub trait Widget<H> {
    fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext);
    fn on_press(&mut self, _handler: &mut H) {}
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

#[typetag::deserialize(tag = "widget", content = "properties")]
pub trait WidgetBuilder: std::fmt::Debug + AsAny + DynClone {
    fn widget_type(
        &self,
        handler: Option<&Ident>,
        comp_struct: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    );
    fn name(&self) -> &'static str;
    fn combine(&mut self, rhs: &dyn WidgetBuilder);
    fn create_widget(&self, widget: Option<&TokenStream>, stream: &mut TokenStream);

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        stream: &mut TokenStream,
    );
    fn get_fluents(&self) -> Vec<(&'static str, &Fluent)>;
    fn get_vars(&self) -> Vec<(&'static str, &str)>;
    fn has_handler(&self) -> bool;
    fn get_widgets(&self) -> Vec<&Option<WidgetDeclaration>>;
}

dyn_clone::clone_trait_object!(WidgetBuilder);
