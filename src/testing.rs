use gui_core::glazier::kurbo::{Affine, Rect};
use gui_core::glazier::{
    Application, Cursor, FileDialogToken, FileInfo, IdleToken, KeyEvent, PointerEvent, Region,
    TimerToken, WinHandler, WindowBuilder, WindowHandle,
};
use gui_core::vello::peniko::Color;
use gui_core::vello::util::{RenderContext, RenderSurface};
use gui_core::vello::{RenderParams, Renderer, RendererOptions, Scene, SceneFragment};
use gui_core::widget::{Handle, WidgetEvent, WidgetID};
use gui_core::{Component, LayoutConstraints, SceneBuilder, Size, ToComponent};
use itertools::Itertools;
use std::any::Any;
use std::marker::PhantomData;
use tracing_subscriber::EnvFilter;
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d,
    ImageCopyBuffer, Maintain, MapMode, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

pub struct TestHarness<T: ToComponent> {
    handle: Handle,
    renderer: Option<Renderer>,
    render: RenderContext,
    scene: Scene,
    size: Size,
    global_positions: Vec<Rect>,
    active_widget: Option<WidgetID>,
    hovered_widgets: Vec<WidgetID>,
    component: T::Component,
    phantom: PhantomData<T>,
}

impl<T: ToComponent> TestHarness<T> {
    pub fn new(component: T) -> Self {
        let render = RenderContext::new().unwrap();
        let mut harness = Self {
            handle: Default::default(),
            renderer: None,
            render,
            scene: Default::default(),
            active_widget: None,
            hovered_widgets: vec![],
            global_positions: vec![
                Rect::default();
                component.largest_id().widget_id() as usize + 1
            ],
            component: component.to_component_holder(),
            size: Size::new(512.0, 512.0),
            phantom: PhantomData,
        };
        harness.init();
        harness
    }

    fn init(&mut self) {
        self.component
            .update_vars(true, &mut self.handle, &self.global_positions[..]);
        self.resize();
    }
    pub fn resize(&mut self) {
        let mut local_positions =
            vec![Rect::default(); self.component.largest_id().widget_id() as usize + 1];
        let size = self.component.resize(
            LayoutConstraints::new_max(self.size),
            &mut self.handle,
            &mut local_positions[..],
        );

        self.global_positions[0] =
            Rect::from_center_size((self.size.width / 2.0, self.size.height / 2.0), size);

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

    pub fn render(&mut self) {
        let (width, height) = (self.size.width as u32, self.size.height as u32);
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // TODO Move out of here
        let dev_id = pollster::block_on(self.render.device(None)).unwrap();
        let device = &self.render.devices[dev_id].device;
        let queue = &self.render.devices[dev_id].queue;
        let texture_desc = TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::STORAGE_BINDING,
            label: None,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        };
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());
        let u32_size = std::mem::size_of::<u32>() as u32;

        let output_buffer_size = (u32_size * width * height) as BufferAddress;
        let output_buffer_desc = BufferDescriptor {
            size: output_buffer_size,
            usage: BufferUsages::COPY_DST
                    // this tells wpgu that we want to read this buffer from the cpu
                    | BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);
        let renderer_options = RendererOptions {
            surface_format: Some(TextureFormat::Rgba8UnormSrgb),
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
            .render_to_texture(device, queue, &self.scene, &texture_view, &render_params)
            .unwrap();
        device.poll(Maintain::Wait);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Copy out buffer"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(u32_size * width),
                    rows_per_image: Some(height),
                },
            },
            texture_desc.size,
        );

        queue.submit(Some(encoder.finish()));
        {
            let buffer_slice = output_buffer.slice(..);

            // NOTE: We have to create the mapping THEN device.poll() before await
            // the future. Otherwise the application will freeze.
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            device.poll(Maintain::Wait);
            pollster::block_on(rx.receive()).unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).unwrap();
            buffer.save("image.png").unwrap();
        }
        output_buffer.unmap();
    }

    fn send_component_event(&mut self, id: WidgetID, event: WidgetEvent) -> bool {
        self.component.event(
            id,
            event,
            &mut self.handle,
            &self.global_positions[..],
            &mut self.active_widget,
            &mut self.hovered_widgets,
        )
    }

    fn propagate_component_event(&mut self, event: WidgetEvent) -> bool {
        self.component.propagate_event(
            event,
            &mut self.handle,
            &self.global_positions[..],
            &mut self.active_widget,
            &mut self.hovered_widgets,
        )
    }
}
//
// impl<C: Component + 'static> WinHandler for TestHarness<C> {
//     fn connect(&mut self, handle: &WindowHandle) {
//         self.handle.window = handle.clone();
//         self.component
//             .update_vars(true, &mut self.handle, &self.global_positions[..]);
//         self.resize();
//         self.render();
//     }
//
//     fn prepare_paint(&mut self) {
//         if self
//             .component
//             .update_vars(false, &mut self.handle, &self.global_positions[..])
//         {
//             self.resize();
//         }
//     }
//
//     fn paint(&mut self, _: &Region) {
//         self.render();
//     }
//
//     fn pointer_move(&mut self, event: &PointerEvent) {
//         self.handle.window.set_cursor(&Cursor::Arrow);
//         let mouse_point = event.pos;
//         let un_hovered_widgets = self
//             .hovered_widgets
//             .iter()
//             .filter(|i| !self.global_positions[i.widget_id() as usize].contains(mouse_point))
//             .copied()
//             .collect_vec();
//
//         self.hovered_widgets = self
//             .hovered_widgets
//             .iter()
//             .copied()
//             .filter(|i| self.global_positions[i.widget_id() as usize].contains(mouse_point))
//             .collect_vec();
//
//         let mut resize = false;
//         for id in un_hovered_widgets.into_iter() {
//             if self.send_component_event(id, WidgetEvent::HoverChange) {
//                 resize = true;
//             }
//         }
//
//         let event_resize = if let Some(id) = self.active_widget {
//             self.send_component_event(id, WidgetEvent::PointerMove(event))
//         } else {
//             self.propagate_component_event(WidgetEvent::PointerMove(event))
//         };
//         let var_resize =
//             self.component
//                 .update_vars(false, &mut self.handle, &self.global_positions[..]);
//
//         if event_resize || var_resize || resize {
//             self.resize();
//         }
//     }
//
//     fn pointer_down(&mut self, event: &PointerEvent) {
//         let event_resize = self.propagate_component_event(WidgetEvent::PointerDown(event));
//         let var_resize =
//             self.component
//                 .update_vars(false, &mut self.handle, &self.global_positions[..]);
//         if event_resize || var_resize {
//             self.resize();
//         }
//     }
//
//     fn pointer_up(&mut self, event: &PointerEvent) {
//         let event_resize = if let Some(id) = self.active_widget {
//             self.send_component_event(id, WidgetEvent::PointerUp(event))
//         } else {
//             self.propagate_component_event(WidgetEvent::PointerUp(event))
//         };
//         let var_resize =
//             self.component
//                 .update_vars(false, &mut self.handle, &self.global_positions[..]);
//         if event_resize || var_resize {
//             self.resize();
//         }
//     }
//     fn as_any(&mut self) -> &mut dyn Any {
//         todo!()
//     }
// }
