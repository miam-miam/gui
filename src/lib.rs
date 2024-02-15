mod update;

use gui_core::glazier::kurbo::{Affine, Rect, Size};
use gui_core::glazier::{
    Application, FileDialogToken, FileInfo, IdleToken, KeyEvent, PointerEvent, Region, Scalable,
    TimerToken, WinHandler, WindowBuilder, WindowHandle,
};
use gui_core::vello::peniko::Color;
use gui_core::vello::util::{RenderContext, RenderSurface};
use gui_core::vello::{RenderParams, Renderer, RendererOptions, Scene, SceneFragment};
use gui_core::{Component, SceneBuilder, ToComponent};
use std::any::Any;
use tracing_subscriber::EnvFilter;

pub use fluent_bundle::concurrent::FluentBundle;
pub use fluent_bundle::{FluentArgs, FluentMessage, FluentResource};
pub use gui_core;
use gui_core::layout::LayoutConstraints;
pub use unic_langid::langid;

pub use gui_derive::ToComponent;

pub use gui_widget;

use gui_core::widget::{Handle, WidgetEvent, WidgetID};
pub use gui_core::Update;
use itertools::Itertools;
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
    handle: Handle,
    renderer: Option<Renderer>,
    render: RenderContext,
    surface: Option<RenderSurface>,
    scene: Scene,
    size: Size,
    global_positions: Vec<Rect>,
    active_widget: Option<WidgetID>,
    hovered_widgets: Vec<WidgetID>,
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
            active_widget: None,
            hovered_widgets: vec![],
            global_positions: vec![Rect::default(); component.largest_id().widget_id() as usize],
            component,
            size: Size::new(800.0, 600.0),
        }
    }

    fn schedule_render(&self) {
        self.handle.window.invalidate();
    }

    fn resize(&mut self) {
        let (max_width, max_height) = self.surface_size();
        let mut local_positions =
            vec![Rect::default(); self.component.largest_id().widget_id() as usize];
        let size = self.component.resize(
            LayoutConstraints::new_max(Size::new(max_width as f64, max_height as f64)),
            &mut self.handle,
            &mut local_positions[..],
        );

        self.global_positions[0] =
            Rect::from_center_size((max_width as f64 / 2.0, max_height as f64 / 2.0), size);

        for (i, rect) in local_positions.into_iter().enumerate() {
            if let Some(parent) = self.component.get_parent(WidgetID::new(
                self.component.largest_id().component_id(),
                i as u32,
            )) {
                let parent_rect = self.global_positions[parent.widget_id() as usize];
                self.global_positions[i] = rect + parent_rect.origin().to_vec2();
            }
        }
    }

    fn surface_size(&self) -> (u32, u32) {
        let window = &self.handle.window;
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
            let component = SceneBuilder::for_fragment(&mut fragment);
            self.component.render(
                component,
                &mut self.handle,
                &mut self.global_positions[..],
                &mut self.active_widget,
                &self.hovered_widgets[..],
            );
            sb.append(
                &fragment,
                Some(Affine::translate(
                    self.global_positions[0].origin().to_vec2(),
                )),
            );

            self.renderer
                .get_or_insert_with(|| Renderer::new(device, &renderer_options).unwrap())
                .render_to_surface(device, queue, &self.scene, &surface_texture, &render_params)
                .unwrap();
            surface_texture.present();
            device.poll(wgpu_types::Maintain::Wait);
        }
    }
}

impl<C: Component + 'static> WinHandler for WindowState<C> {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle.window = handle.clone();
        self.component.update_vars(true);
        self.resize();
        self.render();
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn prepare_paint(&mut self) {
        self.component.update_vars(false);
        self.resize();
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

    fn pointer_move(&mut self, event: &PointerEvent) {
        let mouse_point = event.pos;
        let un_hovered_widgets = self
            .hovered_widgets
            .iter()
            .filter(|i| !self.global_positions[i.widget_id() as usize].contains(mouse_point))
            .copied()
            .collect_vec();

        self.hovered_widgets = self
            .hovered_widgets
            .iter()
            .copied()
            .filter(|i| self.global_positions[i.widget_id() as usize].contains(mouse_point))
            .collect_vec();

        let mut resize = false;
        for id in un_hovered_widgets.into_iter() {
            if self.component.event(
                id,
                WidgetEvent::HoverChange,
                &mut self.handle,
                &self.global_positions[..],
                &mut self.active_widget,
                &mut self.hovered_widgets,
            ) {
                resize = true;
            }
        }

        if self.component.propagate_event(
            WidgetEvent::PointerMove(event),
            &mut self.handle,
            &self.global_positions[..],
            &mut self.active_widget,
            &mut self.hovered_widgets,
        ) || resize
        {
            self.resize();
        }

        self.component.update_vars(false);
    }

    fn pointer_down(&mut self, event: &PointerEvent) {
        if self.component.propagate_event(
            WidgetEvent::PointerDown(event),
            &mut self.handle,
            &self.global_positions[..],
            &mut self.active_widget,
            &mut self.hovered_widgets,
        ) {
            self.resize();
        }

        self.component.update_vars(false);
    }

    fn pointer_up(&mut self, event: &PointerEvent) {
        if self.component.propagate_event(
            WidgetEvent::PointerUp(event),
            &mut self.handle,
            &self.global_positions[..],
            &mut self.active_widget,
            &mut self.hovered_widgets,
        ) {
            self.resize();
        }

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
