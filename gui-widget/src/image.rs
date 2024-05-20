use std::path::{Path, PathBuf};

use image::io::Reader as ImageReader;
use serde::Deserialize;

use gui_custom::glazier::kurbo::Size;
use gui_custom::layout::LayoutConstraints;
use gui_custom::vello::kurbo::Affine;
use gui_custom::vello::peniko::{Blob, Format, Image};
use gui_custom::widget::{
    EventHandle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetEvent, WidgetID,
};
use gui_custom::WidgetBuilder;
use gui_custom::{SceneBuilder, ToComponent, Var};

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

    pub fn set_image_from_file<P: AsRef<Path>>(&mut self, path: P, handle: &mut UpdateHandle) {
        let path_buf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(path);
        inner(&mut self.image, path_buf, handle);
        fn inner(image: &mut Option<Image>, path_buf: PathBuf, handle: &mut UpdateHandle) {
            let dyn_img = ImageReader::open(path_buf).unwrap().decode().unwrap();
            let img = dyn_img.to_rgba8();
            let new_image = Image {
                data: Blob::from(img.to_vec()),
                format: Format::Rgba8,
                width: img.width(),
                height: img.height(),
                extend: Default::default(),
            };
            *image = Some(new_image);
            handle.resize();
        }
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
#[widget(
    name = "Image",
    type_path = "::gui::gui_widget::ImageWidget",
    init_path = "new"
)]
pub struct ImageBuilder {
    #[widget(
        static_only = "set_image_from_file",
        var_only = "set_image",
        static_bound = "P: AsRef<Path>"
    )]
    pub image: Option<Var<String>>,
}
