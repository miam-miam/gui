use gui_core::glazier::kurbo::Size;
use gui_core::layout::LayoutConstraints;
use gui_core::parse::fluent::Fluent;
use gui_core::parse::var::{ComponentVar, Name};
use gui_core::parse::WidgetDeclaration;
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, RuntimeID, UpdateHandle, Widget, WidgetBuilder,
    WidgetEvent, WidgetID,
};
use gui_core::{Point, SceneBuilder, ToComponent};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use serde::Deserialize;

pub struct CompHolder {
    id: WidgetID,
    child_id: Option<RuntimeID>,
}

impl CompHolder {
    pub fn new(id: WidgetID) -> Self {
        CompHolder { id, child_id: None }
    }

    pub fn set_child(&mut self, id: RuntimeID, handle: &mut UpdateHandle) {
        self.child_id = Some(id);
        handle.resize();
    }
}

impl<C: ToComponent> Widget<C> for CompHolder {
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<C>) {
        if let Some(child_id) = self.child_id {
            handle.render_component(scene, child_id);
        }
    }

    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        match self.child_id {
            Some(child_id) => handle.layout_component(Point::ZERO, child_id, constraints),
            None => Size::default(),
        }
    }

    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<C>) {
        if let Some(child_id) = self.child_id {
            handle.propagate_component_event(child_id, event);
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct CompHolderBuilder {
    pub component: Option<ComponentVar>,
}

#[typetag::deserialize(name = "CompHolder")]
impl WidgetBuilder for CompHolderBuilder {
    fn widget_type(
        &self,
        _handler: Option<&Ident>,
        _comp_struct: &Ident,
        _child: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::CompHolder));
    }

    fn name(&self) -> &'static str {
        "CompHolder"
    }
    fn combine(&mut self, _rhs: &dyn WidgetBuilder) {}

    fn create_widget(&self, id: WidgetID, _child: Option<&TokenStream>, stream: &mut TokenStream) {
        stream.extend(quote! {
            ::gui::gui_widget::CompHolder::new(#id)
        });
    }

    #[allow(clippy::single_match)]
    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        handle: &Ident,
        stream: &mut TokenStream,
    ) {
        match property {
            "component" => stream.extend(quote! {#widget.set_child(#value, #handle);}),
            _ => {}
        }
    }

    fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
        vec![]
    }

    fn get_fluents(&self) -> Vec<(&'static str, Fluent)> {
        vec![]
    }

    fn get_vars(&self) -> Vec<(&'static str, Name)> {
        vec![]
    }

    fn get_components(&self) -> Vec<(&'static str, ComponentVar)> {
        let mut result = vec![];
        if let Some(v) = &self.component {
            result.push(("component", v.clone()));
        }
        result
    }

    fn get_widgets(&mut self) -> Option<Vec<&mut WidgetDeclaration>> {
        None
    }

    fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>> {
        None
    }
}
