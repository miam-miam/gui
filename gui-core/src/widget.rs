use parley::FontContext;
use proc_macro2::{Ident, TokenStream};
use std::any::Any;
use vello::SceneBuilder;
use crate::parse::fluent::Fluent;

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
pub trait WidgetBuilder: std::fmt::Debug + AsAny {
    fn name(&self) -> &'static str;
    fn combine(&mut self, rhs: &dyn WidgetBuilder);
    fn create_widget(&self, stream: &mut TokenStream);
    fn on_var_update(&self, widget: &Ident, var: &str, value: &Ident, stream: &mut TokenStream);
    
    // (property_name, fluent)
    fn get_fluents(&self) -> Vec<(&'static str, &Fluent)>;
}
