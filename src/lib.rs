mod testing;
mod update;

use gui_core::glazier::kurbo::{Affine, Rect, Size};
use gui_core::glazier::{
    Application, Cursor, FileDialogToken, FileInfo, IdleToken, KeyEvent, PointerEvent, Region,
    Scalable, TimerToken, WinHandler, WindowBuilder, WindowHandle,
};
use gui_core::vello::peniko::Color;
use gui_core::vello::util::{RenderContext, RenderSurface};
use gui_core::vello::{RenderParams, Renderer, RendererOptions, Scene, SceneFragment};
pub use gui_core::CompHolder;
use gui_core::{Component, SceneBuilder, ToComponent};
use std::any::Any;
use tracing_subscriber::EnvFilter;

pub use fluent_bundle::concurrent::FluentBundle;
pub use fluent_bundle::{FluentArgs, FluentMessage, FluentResource};
#[doc(hidden)]
pub use gui_core;
use gui_core::layout::LayoutConstraints;
pub use unic_langid::langid;

pub use gui_derive::{type_registry, ToComponent};

pub use gui_widget;

pub use gui_core::glazier::PointerButton;

use gui_core::widget::{Handle, RuntimeID, WidgetEvent, WidgetID};
pub use gui_core::Update;
pub use testing::TestHarness;
pub use update::Updateable;
use wgpu::Maintain;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

pub fn run<T: ToComponent>(component: T)
where
    <T as ToComponent>::Component: 'static,
{
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let app = Application::new().unwrap();
    let window = WindowBuilder::new(app.clone())
        .size((WIDTH as f64, HEIGHT as f64).into())
        .handler(Box::new(WindowState::new(component.to_component_holder())))
        .build()
        .unwrap();
    window.show();
    app.run(None);
}

struct WindowState<C: Component + 'static> {
    handle: Handle,
    renderer: Option<Renderer>,
    render: RenderContext,
    surface: Option<RenderSurface>,
    scene: Scene,
    size: Size,
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
            component,
            size: Size::new(WIDTH as f64, HEIGHT as f64),
        }
    }

    fn resize(&mut self) {
        let (max_width, max_height) = self.surface_size();
        self.handle.info.reset_positions();
        let size = self.component.resize(
            LayoutConstraints::new_max(Size::new(max_width as f64, max_height as f64)),
            &mut self.handle,
        );

        self.handle.info.convert_to_global_positions(
            Rect::from_center_size((max_width as f64 / 2.0, max_height as f64 / 2.0), size),
            &self.component,
        );
    }

    fn surface_size(&self) -> (u32, u32) {
        let window = &self.handle.window;
        if window == &WindowHandle::default() {
            return (self.size.width as u32, self.size.height as u32);
        }
        let scale = window.get_scale().unwrap_or_default();
        let insets = window.content_insets().to_px(scale);
        let mut size = window.get_size().to_px(scale);
        size.width -= insets.x_value();
        size.height -= insets.y_value();
        (size.width as u32, size.height as u32)
    }

    // Code mostly adapted from https://github.com/linebender/glazier/blob/main/examples/shello.rs
    fn render(&mut self) {
        let (width, height) = self.surface_size();
        if self.surface.is_none() {
            self.surface = Some(
                pollster::block_on(
                    self.render
                        .create_surface(&self.handle.window, width, height),
                )
                .expect("failed to create surface"),
            );
        }

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
                base_color: Color::WHITE,
                width,
                height,
            };

            let mut sb = SceneBuilder::for_scene(&mut self.scene);
            let mut fragment = SceneFragment::new();
            let mut component = SceneBuilder::for_fragment(&mut fragment);
            self.component.render(&mut component, &mut self.handle);
            sb.append(
                &fragment,
                Some(Affine::translate(
                    self.handle
                        .info
                        .get_rect(RuntimeID::new(0), WidgetID::new(0))
                        .origin()
                        .to_vec2(),
                )),
            );

            self.renderer
                .get_or_insert_with(|| Renderer::new(device, &renderer_options).unwrap())
                .render_to_surface(device, queue, &self.scene, &surface_texture, &render_params)
                .unwrap();
            surface_texture.present();
            device.poll(Maintain::Wait);
        }
    }

    fn send_component_event(
        &mut self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
        event: WidgetEvent,
    ) -> bool {
        self.component
            .event(runtime_id, widget_id, event, &mut self.handle)
    }

    fn propagate_component_event(&mut self, event: WidgetEvent) -> bool {
        self.component.propagate_event(event, &mut self.handle)
    }
}

impl<C: Component + 'static> WinHandler for WindowState<C> {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle.window = handle.clone();
        self.component.update_vars(true, &mut self.handle);
        self.resize();
        self.render();
    }

    fn size(&mut self, size: Size) {
        if self.size != size {
            self.size = size;
            self.resize();
        }
    }

    fn prepare_paint(&mut self) {
        if self.component.update_vars(false, &mut self.handle) {
            self.resize();
        }
    }

    fn paint(&mut self, _: &Region) {
        self.render();
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

    fn pointer_move(&mut self, event: &PointerEvent) {
        if self.handle.window != WindowHandle::default() {
            self.handle.window.set_cursor(&Cursor::Arrow);
        }
        let mouse_point = event.pos;
        let un_hovered_widgets = self.handle.info.remove_un_hovered(mouse_point);

        let mut resize = false;
        for id in un_hovered_widgets.into_iter() {
            if self.send_component_event(id.0, id.1, WidgetEvent::HoverChange) {
                resize = true;
            }
        }

        let event_resize = if let Some(id) = self.handle.info.get_active_widget() {
            self.send_component_event(id.0, id.1, WidgetEvent::PointerMove(event))
        } else {
            self.propagate_component_event(WidgetEvent::PointerMove(event))
        };
        let var_resize = self.component.update_vars(false, &mut self.handle);

        if event_resize || var_resize || resize {
            self.resize();
        }
    }

    fn pointer_down(&mut self, event: &PointerEvent) {
        let event_resize = self.propagate_component_event(WidgetEvent::PointerDown(event));
        let var_resize = self.component.update_vars(false, &mut self.handle);
        if event_resize || var_resize {
            self.resize();
        }
    }

    fn pointer_up(&mut self, event: &PointerEvent) {
        let event_resize = if let Some(id) = self.handle.info.get_active_widget() {
            self.send_component_event(id.0, id.1, WidgetEvent::PointerUp(event))
        } else {
            self.propagate_component_event(WidgetEvent::PointerUp(event))
        };
        let var_resize = self.component.update_vars(false, &mut self.handle);
        if event_resize || var_resize {
            self.resize();
        }
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
        self.handle.window.close();
    }

    fn destroy(&mut self) {
        Application::global().quit();
    }

    fn idle(&mut self, _: IdleToken) {}

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
