use std::path::PathBuf;

use image::io::Reader as ImageReader;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;

use gui_core::{SceneBuilder, ToComponent, Var};
use gui_core::glazier::kurbo::Size;
use gui_core::layout::LayoutConstraints;
use gui_core::parse::var::Name;
use gui_core::vello::kurbo::Affine;
use gui_core::vello::peniko::{Blob, Format, Image};
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetBuilder, WidgetEvent,
    WidgetID,
};
use gui_derive::WidgetBuilder;

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

#[derive(Deserialize, WidgetBuilder, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
#[widget(name = "Image", type_path = "::gui::gui_widget::ImageWidget", init_path = "new")]
pub struct ImageBuilder {
    #[widget(static_only = "set_image_from_file", var_only = "set_image")]
    pub image: Option<Var<String>>,
}