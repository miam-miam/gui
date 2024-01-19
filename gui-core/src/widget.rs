use crate::parse::fluent::Fluent;
use dyn_clone::DynClone;
use parley::FontContext;
use proc_macro2::{Ident, TokenStream};
use std::any::Any;
use vello::SceneBuilder;

pub trait Widget {
    fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext);
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
    fn widget_type(&self, stream: &mut TokenStream); 
    fn name(&self) -> &'static str;
    fn combine(&mut self, rhs: &dyn WidgetBuilder);
    fn create_widget(&self, stream: &mut TokenStream);

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        stream: &mut TokenStream,
    );
    fn get_fluents(&self) -> Vec<(&'static str, &Fluent)>;
    fn get_vars(&self) -> Vec<(&'static str, &str)>;
}

dyn_clone::clone_trait_object!(WidgetBuilder);
