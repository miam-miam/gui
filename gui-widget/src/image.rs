use gui_core::glazier::kurbo::Size;
use gui_core::layout::LayoutConstraints;
use gui_core::parse::var::Name;
use gui_core::vello::kurbo::Affine;
use gui_core::vello::peniko::{Blob, Format, Image};
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetBuilder, WidgetEvent,
    WidgetID,
};
use gui_core::{SceneBuilder, ToComponent, Var};
use image::io::Reader as ImageReader;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Default)]
pub struct ImageWidget {
    id: WidgetID,
    image: Option<Image>,
}

impl ImageWidget {
    pub fn new(id: WidgetID) -> Self {
        Self { id, image: None }
    }

    pub fn set_image(&mut self, image: Image, handle: &mut UpdateHandle) {
        self.image = Some(image);
        handle.resize();
    }

    pub fn set_image_from_file(&mut self, path: &str, handle: &mut UpdateHandle) {
        let path_buf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(path);
        let dyn_img = ImageReader::open(path_buf).unwrap().decode().unwrap();
        let img = dyn_img.to_rgba8();
        let image = Image {
            data: Blob::from(img.to_vec()),
            format: Format::Rgba8,
            width: img.width(),
            height: img.height(),
            extend: Default::default(),
        };
        self.image = Some(image);
        handle.resize();
    }
}

impl<C: ToComponent> Widget<C> for ImageWidget {
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, _handle: &mut RenderHandle<C>) {
        if let Some(image) = &self.image {
            scene.draw_image(image, Affine::IDENTITY)
        }
    }

    fn resize(&mut self, _constraints: LayoutConstraints, _handle: &mut ResizeHandle<C>) -> Size {
        match &self.image {
            Some(image) => Size::new(image.width as f64, image.height as f64),
            None => Size::default(),
        }
    }

    fn event(&mut self, _event: WidgetEvent, _handle: &mut EventHandle<C>) {}
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ImageBuilder {
    pub image: Option<Var<String>>,
}

#[typetag::deserialize(name = "Image")]
impl WidgetBuilder for ImageBuilder {
    fn widget_type(
        &self,
        _handler: Option<&Ident>,
        _comp_struct: &Ident,
        _child: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::ImageWidget));
    }

    fn name(&self) -> &'static str {
        "Image"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.image {
                self.image.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, id: WidgetID, stream: &mut TokenStream) {
        stream.extend(quote! {
            ::gui::gui_widget::ImageWidget::new(#id)
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
            "image" => stream.extend(quote! {#widget.set_image(#value, #handle);}),
            "image_file" => stream.extend(quote! {#widget.set_image_from_file(#value, #handle);}),
            _ => {}
        }
    }

    #[allow(clippy::single_match)]
    fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
        let mut array = vec![];
        match &self.image {
            Some(Var::Value(v)) => array.push(("image_file", v.to_token_stream())),
            _ => {}
        }
        array
    }

    fn get_vars(&self) -> Vec<(&'static str, Name)> {
        let mut array = vec![];
        if let Some(Var::Variable(v)) = &self.image {
            array.push(("image", v.clone()));
        }
        array
    }
}
