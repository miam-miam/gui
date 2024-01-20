mod update;

use gui_core::glazier::kurbo::Size;
use gui_core::glazier::{
    Application, FileDialogToken, FileInfo, IdleToken, KeyEvent, PointerEvent, Region, Scalable,
    TimerToken, WinHandler, WindowBuilder, WindowHandle,
};
use gui_core::vello::peniko::Color;
use gui_core::vello::util::{RenderContext, RenderSurface};
use gui_core::vello::{RenderParams, Renderer, RendererOptions, Scene};
use gui_core::{Component, FontContext, SceneBuilder, ToComponent};
use std::any::Any;
use tracing_subscriber::EnvFilter;

pub use fluent_bundle::concurrent::FluentBundle;
pub use fluent_bundle::{FluentArgs, FluentMessage, FluentResource};
pub use gui_core;
pub use unic_langid::langid;

pub use gui_derive::ToComponent;

pub use gui_widget;

pub use gui_core::Update;
pub use update::Updateable;

const WIDTH: usize = 2048;
const HEIGHT: usize = 1536;

pub fn run<T: ToComponent>(component: T)
where
    <T as ToComponent>::Component: 'static,
{
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let app = Application::new().unwrap();
    let window = WindowBuilder::new(app.clone())
        .size((WIDTH as f64 / 2., HEIGHT as f64 / 2.).into())
        .handler(Box::new(WindowState::new(component.to_component_holder())))
        .build()
        .unwrap();
    window.show();
    app.run(None);
}

struct WindowState<C: Component + 'static> {
    handle: WindowHandle,
    renderer: Option<Renderer>,
    render: RenderContext,
    surface: Option<RenderSurface>,
    scene: Scene,
    size: Size,
    font_context: FontContext,
    component: C,
}

impl<C: Component> WindowState<C> {
    pub fn new(component: C) -> Self {
        let render = RenderContext::new().unwrap();
        Self {
            handle: Default::default(),
            surface: None,
            renderer: None,
            render,
            scene: Default::default(),
            font_context: FontContext::new(),
            component,
            size: Size::new(800.0, 600.0),
        }
    }

    fn schedule_render(&self) {
        self.handle.invalidate();
    }

    fn surface_size(&self) -> (u32, u32) {
        let handle = &self.handle;
        let scale = handle.get_scale().unwrap_or_default();
        let insets = handle.content_insets().to_px(scale);
        let mut size = handle.get_size().to_px(scale);
        size.width -= insets.x_value();
        size.height -= insets.y_value();
        (size.width as u32, size.height as u32)
    }

    // Code mostly adapted from https://github.com/linebender/glazier/blob/main/examples/shello.rs
    fn render(&mut self) {
        let (width, height) = self.surface_size();
        if self.surface.is_none() {
            self.surface = Some(
                pollster::block_on(self.render.create_surface(&self.handle, width, height))
                    .expect("failed to create surface"),
            );
        }

        let sb = SceneBuilder::for_scene(&mut self.scene);
        self.component.render(sb, &mut self.font_context);

        if let Some(surface) = self.surface.as_mut() {
            if surface.config.width != width || surface.config.height != height {
                self.render.resize_surface(surface, width, height);
            }
            let surface_texture = surface.surface.get_current_texture().unwrap();
            let dev_id = surface.dev_id;
            let device = &self.render.devices[dev_id].device;
            let queue = &self.render.devices[dev_id].queue;
            let renderer_options = RendererOptions {
                surface_format: Some(surface.format),
                timestamp_period: queue.get_timestamp_period(),
            };
            let render_params = RenderParams {
                base_color: Color::rgb8(245, 246, 250),
                width,
                height,
            };
            self.renderer
                .get_or_insert_with(|| Renderer::new(device, &renderer_options).unwrap())
                .render_to_surface(device, queue, &self.scene, &surface_texture, &render_params)
                .unwrap();
            surface_texture.present();
        }
    }
}

impl<C: Component + 'static> WinHandler for WindowState<C> {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
        self.component.update_vars(true);
        self.schedule_render();
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn prepare_paint(&mut self) {
        self.component.update_vars(false);
    }

    fn paint(&mut self, _: &Region) {
        self.render();
        self.schedule_render();
    }

    fn command(&mut self, _id: u32) {}

    fn save_as(&mut self, _token: FileDialogToken, file: Option<FileInfo>) {
        println!("save file result: {file:?}");
    }

    fn open_file(&mut self, _token: FileDialogToken, file_info: Option<FileInfo>) {
        println!("open file result: {file_info:?}");
    }

    fn key_down(&mut self, event: &KeyEvent) -> bool {
        println!("keydown: {event:?}");
        false
    }

    fn key_up(&mut self, event: &KeyEvent) {
        println!("keyup: {event:?}");
    }

    fn wheel(&mut self, event: &PointerEvent) {
        println!("wheel {event:?}");
    }

    fn pointer_move(&mut self, _event: &PointerEvent) {
        // self.handle.set_cursor(&Cursor::Arrow);
        // println!("pointer_move {event:?}");
    }

    fn pointer_down(&mut self, event: &PointerEvent) {
        println!("pointer_down {event:?}");
    }

    fn pointer_up(&mut self, event: &PointerEvent) {
        self.component.pointer_up(event);
        self.component.update_vars(false);
    }

    fn timer(&mut self, id: TimerToken) {
        println!("timer fired: {id:?}");
    }

    fn got_focus(&mut self) {
        println!("Got focus");
    }

    fn lost_focus(&mut self) {
        println!("Lost focus");
    }

    fn request_close(&mut self) {
        self.handle.close();
    }

    fn destroy(&mut self) {
        Application::global().quit();
    }

    fn idle(&mut self, _: IdleToken) {}

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
