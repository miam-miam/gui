mod messages;

use gui_core::glazier::kurbo::{Affine, Rect};
use gui_core::vello::peniko::Color;
use gui_core::vello::util::RenderContext;
use gui_core::vello::{RenderParams, Renderer, RendererOptions, Scene, SceneFragment};
use gui_core::widget::{Handle, WidgetEvent, WidgetID};
use gui_core::{Component, LayoutConstraints, SceneBuilder, Size, ToComponent};
use image::io::Reader as ImageReader;
use image::{ImageBuffer, Pixel, Rgba};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;
use std::thread;
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, Maintain,
    MapMode, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

#[macro_export]
macro_rules! assert_screenshot {
    ($harness:expr, $($arg:tt)*) => {$harness.take_screenshot(file!(), &format!($($arg)*))};
}

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn compare_images<
    P: Pixel,
    LC: Deref<Target = [P::Subpixel]>,
    RC: Deref<Target = [P::Subpixel]>,
>(
    lhs: &ImageBuffer<P, LC>,
    rhs: &ImageBuffer<P, RC>,
) -> bool {
    if lhs.width() != rhs.width() || lhs.height() != rhs.height() {
        return false;
    }
    let length = (lhs.width() * lhs.height() * P::CHANNEL_COUNT as u32) as usize;
    lhs.as_raw()[..length] == rhs.as_raw()[..length]
}

#[derive(Debug, Default, Copy, Clone)]
struct TestReport {
    total_tests: u32,
    failed_tests: u32,
    wip_tests: u32,
}

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
    image_buffer: Vec<u8>,
    report: TestReport,
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
            report: TestReport::default(),
            image_buffer: vec![],
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

        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise, the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(Maintain::Wait);
        pollster::block_on(rx.receive()).unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        self.image_buffer.extend_from_slice(&data);
        drop(data);
        output_buffer.unmap();
    }

    fn get_screenshot_environment() -> String {
        option_env!("CI").map_or_else(
            || {
                option_env!("GUI_SCREENSHOT_RUNNER")
                    .map_or(String::new(), |s| String::from('_') + s)
            },
            |_| "_ci".into(),
        )
    }

    pub fn take_screenshot(&mut self, file_path: &str, message: &str) {
        if message.contains(char::is_whitespace) {
            panic!("Message should not contain any whitespace: {message}")
        }
        let file_name = Path::new(file_path)
            .file_stem()
            .expect("filepath points to a rust file")
            .to_str()
            .expect("filename to be valid UTF-8");
        self.report.total_tests += 1;
        messages::start_test(file_name, message).unwrap();
        self.render();
        let new_image = ImageBuffer::<Rgba<u8>, _>::from_raw(
            self.size.width as u32,
            self.size.height as u32,
            &self.image_buffer[..],
        )
        .expect("generated image is valid");
        let cargo_dir_env =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");
        let cargo_dir = Path::new(&cargo_dir_env);
        let screenshot_dir = cargo_dir.join("screenshots");
        create_dir_all(&screenshot_dir).expect("screenshots folder can be created");
        let gitignore = screenshot_dir.join(".gitignore");
        let _ = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(gitignore)
            .and_then(|mut f| writeln!(f, "*.new.png\n.gitignore"));
        let screenshot_runner = Self::get_screenshot_environment();
        let reference_path =
            screenshot_dir.join(format!("{file_name}_{message}{screenshot_runner}.png"));
        let new_path =
            screenshot_dir.join(format!("{file_name}_{message}{screenshot_runner}.new.png"));
        if let Ok(reference_file) = ImageReader::open(&reference_path) {
            let reference_img = reference_file.decode().unwrap().to_rgba8();
            if !compare_images(&reference_img, &new_image) {
                let _ = std::fs::remove_file(&new_path);
                new_image.save(&new_path).unwrap();
                self.report.failed_tests += 1;
                messages::print_fail_test(&reference_path, &new_path).unwrap()
            } else {
                messages::print_pass_test().unwrap()
            }
        } else {
            let _ = std::fs::remove_file(&new_path);
            new_image.save(&new_path).unwrap();
            self.report.wip_tests += 1;
            messages::print_wip_test(&reference_path, &new_path).unwrap()
        }
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

impl<T: ToComponent> Drop for TestHarness<T> {
    fn drop(&mut self) {
        if !thread::panicking() {
            if self.report.failed_tests > 0 {
                panic!(
                    "{} of {} tests failed",
                    self.report.failed_tests, self.report.total_tests
                );
            } else if self.report.wip_tests > 0 {
                panic!("created {} new images to check", self.report.wip_tests)
            }
        }
    }
}
