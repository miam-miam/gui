mod messages;

use crate::WindowState;
use gui_core::glazier::kurbo::{Affine, Rect};
use gui_core::glazier::{PointerButton, PointerEvent, WinHandler};
use gui_core::vello::peniko::Color;
use gui_core::vello::{RenderParams, Renderer, RendererOptions, SceneFragment};
use gui_core::widget::WidgetID;
use gui_core::{Component, Point, SceneBuilder, Size, ToComponent};
use image::io::Reader as ImageReader;
use image::{ImageBuffer, Pixel, Rgba};
use std::default::Default;
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

pub struct TestHarness<T: ToComponent>
where
    <T as ToComponent>::Component: 'static,
{
    window_state: WindowState<T::Component>,
    image_buffer: Vec<u8>,
    report: TestReport,
    last_mouse_pos: Option<Point>,
    phantom: PhantomData<T>,
}

impl<T: ToComponent + 'static> TestHarness<T>
where
    <T as ToComponent>::Component: 'static,
{
    pub fn new(component: T) -> Self {
        let mut harness = Self {
            window_state: WindowState::new(component.to_component_holder()),
            report: TestReport::default(),
            image_buffer: vec![],
            last_mouse_pos: None,
            phantom: PhantomData,
        };
        harness.init();
        harness
    }

    pub fn get_component(&mut self) -> &mut T {
        self.window_state
            .component
            .get_comp_struct()
            .downcast_mut::<T>()
            .unwrap()
    }

    fn init(&mut self) {
        self.window_state.size = Size::new(512.0, 512.0);
        self.window_state.component.update_vars(
            true,
            &mut self.window_state.handle,
            &self.window_state.global_positions[..],
        );
        self.window_state.resize();
    }

    fn render(&mut self) {
        let (width, height) = (
            self.window_state.size.width as u32,
            self.window_state.size.height as u32,
        );
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // TODO Move out of here
        let dev_id = pollster::block_on(self.window_state.render.device(None)).unwrap();
        let device = &self.window_state.render.devices[dev_id].device;
        let queue = &self.window_state.render.devices[dev_id].queue;
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

        let mut sb = SceneBuilder::for_scene(&mut self.window_state.scene);
        let mut fragment = SceneFragment::new();
        let component = SceneBuilder::for_fragment(&mut fragment);
        self.window_state.component.render(
            component,
            &mut self.window_state.handle,
            &mut self.window_state.global_positions[..],
            &mut self.window_state.active_widget,
            &self.window_state.hovered_widgets[..],
        );
        sb.append(
            &fragment,
            Some(Affine::translate(
                self.window_state.global_positions[0].origin().to_vec2(),
            )),
        );

        self.window_state
            .renderer
            .get_or_insert_with(|| Renderer::new(device, &renderer_options).unwrap())
            .render_to_texture(
                device,
                queue,
                &self.window_state.scene,
                &texture_view,
                &render_params,
            )
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

    /// Returns the size of the widget (not the component)
    pub fn set_size<S: Into<Size>>(&mut self, size: S) -> Size {
        self.window_state.size = size.into();
        self.window_state.resize();
        self.window_state.global_positions[0].size()
    }

    pub fn take_screenshot(&mut self, file_path: &str, message: &str) {
        if message.contains(char::is_whitespace) {
            panic!("Message should not contain any whitespace: {message}")
        }

        self.report.total_tests += 1;
        let file_name = Path::new(file_path)
            .file_stem()
            .expect("filepath points to a rust file")
            .to_str()
            .expect("filename to be valid UTF-8");
        messages::start_test(file_name, message).unwrap();

        self.window_state.prepare_paint();
        self.image_buffer.clear();
        self.render();

        let new_image = ImageBuffer::<Rgba<u8>, _>::from_raw(
            self.window_state.size.width as u32,
            self.window_state.size.height as u32,
            &self.image_buffer[..],
        )
        .expect("generated image is valid");

        let cargo_dir_env =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by cargo");
        let cargo_dir = Path::new(&cargo_dir_env);
        let screenshot_dir = cargo_dir.join("screenshots");
        let wip_dir = screenshot_dir.join("wip");
        create_dir_all(&wip_dir).expect("screenshots folder can be created");
        let gitignore = screenshot_dir.join(".gitignore");

        let _ = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(gitignore)
            .and_then(|mut f| writeln!(f, "wip\n.gitignore"));

        let screenshot_runner = Self::get_screenshot_environment();
        let count = self.report.total_tests - 1;
        let reference_path = screenshot_dir.join(format!(
            "{file_name}_{count:03}_{message}{screenshot_runner}.png"
        ));
        let new_path = wip_dir.join(format!(
            "{file_name}_{count:03}_{message}{screenshot_runner}.png"
        ));

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

    pub fn get_id(&self, name: &str) -> Option<WidgetID> {
        self.window_state.component.get_id(name)
    }

    pub fn get_local_rect(&self, id: WidgetID) -> Rect {
        Rect::from_origin_size(
            (0.0, 0.0),
            self.window_state.global_positions[id.widget_id() as usize].size(),
        )
    }

    fn get_global_point(&self, id: WidgetID, local_pos: Point) -> Point {
        (self.window_state.global_positions[id.widget_id() as usize]
            .origin()
            .to_vec2()
            + local_pos.to_vec2())
        .to_point()
    }

    pub fn simulate_pointer_down_up(&mut self, button: PointerButton, local_pos: Option<WidgetID>) {
        self.simulate_pointer_down(button, local_pos);
        self.simulate_pointer_up(button, local_pos);
    }

    pub fn simulate_pointer_down(&mut self, button: PointerButton, local_pos: Option<WidgetID>) {
        let pos = local_pos.map_or_else(
            || {
                self.last_mouse_pos
                    .unwrap_or_else(|| self.window_state.size.to_rect().center())
            },
            |id| self.window_state.global_positions[id.widget_id() as usize].center(),
        );
        let mut pointer_event = PointerEvent {
            pos,
            ..PointerEvent::default()
        };

        if Some(pointer_event.pos) != self.last_mouse_pos {
            self.last_mouse_pos = Some(pointer_event.pos);
            self.window_state.pointer_move(&pointer_event);
        }
        pointer_event.button = button;

        self.window_state.pointer_down(&pointer_event);
    }

    pub fn simulate_pointer_up(&mut self, button: PointerButton, local_pos: Option<WidgetID>) {
        let pos = local_pos.map_or_else(
            || {
                self.last_mouse_pos
                    .unwrap_or_else(|| self.window_state.size.to_rect().center())
            },
            |id| self.window_state.global_positions[id.widget_id() as usize].center(),
        );
        let mut pointer_event = PointerEvent {
            pos,
            ..PointerEvent::default()
        };

        if Some(pointer_event.pos) != self.last_mouse_pos {
            self.last_mouse_pos = Some(pointer_event.pos);
            self.window_state.pointer_move(&pointer_event);
        }
        pointer_event.button = button;

        self.window_state.pointer_up(&pointer_event);
    }

    pub fn simulate_pointer_move(&mut self, id: WidgetID, local_pos: Option<Point>) {
        let pos = local_pos.unwrap_or_else(|| self.get_local_rect(id).center());
        let pointer_event = PointerEvent {
            pos: self.get_global_point(id, pos),
            ..PointerEvent::default()
        };
        self.last_mouse_pos = Some(pointer_event.pos);
        self.window_state.pointer_move(&pointer_event);
    }
}

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
