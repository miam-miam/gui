mod messages;
mod render;

use crate::WindowState;
use gui_core::glazier::kurbo::Rect;
use gui_core::glazier::{PointerButton, PointerEvent, WinHandler};
use gui_core::widget::{RuntimeID, WidgetID};
use gui_core::{Component, Point, Size, ToComponent};
use image::io::Reader as ImageReader;
use image::{ImageBuffer, Pixel, Rgba};
use std::default::Default;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::thread;

/// Render a screenshot and add it to the `wip` folder if it does not match the currently stored
/// image. The `GUI_SCREENSHOT_RUNNER` environment variable can be used to control what screenshot group gets used.
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

fn get_screenshot_environment() -> String {
    option_env!("CI").map_or_else(
        || option_env!("GUI_SCREENSHOT_RUNNER").map_or(String::new(), |s| String::from('_') + s),
        |_| "_ci".into(),
    )
}

#[derive(Debug, Default, Copy, Clone)]
struct TestReport {
    total_tests: u32,
    failed_tests: u32,
    wip_tests: u32,
}

/// Create a [`TestHarness`] to tests a component.
/// The [`assert_screenshot`] macro can be used to check is visually identical between runs.
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

impl<T: ToComponent> TestHarness<T> {
    pub fn new<S: Into<Size>>(component: T, size: S) -> Self {
        let mut harness = Self {
            window_state: WindowState::new(component.to_component_holder(RuntimeID::next())),
            report: TestReport::default(),
            image_buffer: vec![],
            last_mouse_pos: None,
            phantom: PhantomData,
        };
        harness.init(size.into());
        harness
    }

    fn init(&mut self, size: Size) {
        self.window_state.size = size;
        self.window_state
            .component
            .update_vars(true, &mut self.window_state.handle);
        self.window_state.resize();
    }

    /// Returns the size of the first widget (not the component)
    pub fn set_size<S: Into<Size>>(&mut self, size: S) -> Size {
        self.window_state.size = size.into();
        self.window_state.resize();
        self.window_state
            .handle
            .info
            .get_rect(self.window_state.component.id(), Default::default())
            .size()
    }

    fn create_screenshot_paths(&self, source_file_name: &str, message: &str) -> (PathBuf, PathBuf) {
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

        let screenshot_runner = get_screenshot_environment();

        let count = self.report.total_tests - 1;
        let file_name = format!("{source_file_name}_{count:03}_{message}{screenshot_runner}.png");
        let reference_path = screenshot_dir.join(&file_name);
        let new_path = wip_dir.join(&file_name);

        (reference_path, new_path)
    }

    /// Prefer to use [`assert_screenshot`] to take screenshots.
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

        let (reference_path, new_path) = self.create_screenshot_paths(file_name, message);

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

    pub fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
        self.window_state.component.get_id(name)
    }

    pub fn get_local_rect(&self, runtime_id: RuntimeID, widget_id: WidgetID) -> Rect {
        Rect::from_origin_size(
            (0.0, 0.0),
            self.window_state
                .handle
                .info
                .get_rect(runtime_id, widget_id)
                .size(),
        )
    }

    fn get_global_point(
        &self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
        local_pos: Point,
    ) -> Point {
        (self
            .window_state
            .handle
            .info
            .get_rect(runtime_id, widget_id)
            .origin()
            .to_vec2()
            + local_pos.to_vec2())
        .to_point()
    }

    pub fn simulate_pointer_down_up(
        &mut self,
        button: PointerButton,
        local_pos: Option<(RuntimeID, WidgetID)>,
    ) {
        self.simulate_pointer_down(button, local_pos);
        self.simulate_pointer_up(button, local_pos);
    }

    pub fn simulate_pointer_down(
        &mut self,
        button: PointerButton,
        local_pos: Option<(RuntimeID, WidgetID)>,
    ) {
        let pos = local_pos.map_or_else(
            || {
                self.last_mouse_pos
                    .unwrap_or_else(|| self.window_state.size.to_rect().center())
            },
            |id| self.window_state.handle.info.get_rect(id.0, id.1).center(),
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

    pub fn simulate_pointer_up(
        &mut self,
        button: PointerButton,
        local_pos: Option<(RuntimeID, WidgetID)>,
    ) {
        let pos = local_pos.map_or_else(
            || {
                self.last_mouse_pos
                    .unwrap_or_else(|| self.window_state.size.to_rect().center())
            },
            |id| self.window_state.handle.info.get_rect(id.0, id.1).center(),
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

    pub fn simulate_pointer_move(
        &mut self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
        local_pos: Option<Point>,
    ) {
        let pos = local_pos.unwrap_or_else(|| self.get_local_rect(runtime_id, widget_id).center());
        let pointer_event = PointerEvent {
            pos: self.get_global_point(runtime_id, widget_id, pos),
            ..PointerEvent::default()
        };
        self.last_mouse_pos = Some(pointer_event.pos);
        self.window_state.pointer_move(&pointer_event);
    }
}

impl<T: ToComponent + 'static> TestHarness<T>
where
    <T as ToComponent>::Component: 'static,
{
    pub fn get_component(&mut self) -> &mut T {
        self.window_state
            .component
            .get_comp_struct()
            .downcast_mut::<T>()
            .unwrap()
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

#[cfg(test)]
mod tests {
    use super::compare_images;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn compare_images_identical() {
        let img1 = ImageBuffer::from_fn(10, 10, |x, y| Rgba([x as u8, y as u8, 0, 0]));
        let img2 = ImageBuffer::from_fn(10, 10, |x, y| Rgba([x as u8, y as u8, 0, 0]));
        assert!(compare_images(&img1, &img2));
    }

    #[test]
    fn compare_images_different_dimensions() {
        let img1 = ImageBuffer::from_fn(10, 10, |x, y| Rgba([x as u8, y as u8, 0, 0]));
        let img2 = ImageBuffer::from_fn(10, 11, |x, y| Rgba([x as u8, y as u8, 0, 0]));
        assert!(!compare_images(&img1, &img2));
    }

    #[test]
    fn compare_images_same_dimensions_different_data() {
        let img1 = ImageBuffer::from_fn(10, 10, |x, y| Rgba([x as u8, y as u8, 0, 0]));
        let img2 = ImageBuffer::from_fn(10, 10, |x, y| Rgba([x as u8, y as u8, 255, 255]));
        assert!(!compare_images(&img1, &img2));
    }
}
