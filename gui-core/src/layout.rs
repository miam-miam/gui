use vello::kurbo::common::FloatExt;
use vello::kurbo::Size;

#[derive(Copy, Clone, Debug, PartialEq)]
#[must_use]
pub struct LayoutConstraints {
    min: Size,
    max: Size,
}

fn expand_safe(val: f64) -> f64 {
    val.expand().max(0.0)
}

impl LayoutConstraints {
    pub const UNBOUNDED: Self = LayoutConstraints {
        min: Size::ZERO,
        max: Size::new(f64::INFINITY, f64::INFINITY),
    };

    pub fn new(min: Size, max: Size) -> LayoutConstraints {
        assert!(min.width <= max.width, "min width greater than max width");
        assert!(
            min.height <= max.height,
            "min height greater than max height"
        );
        LayoutConstraints {
            min: Size::new(expand_safe(min.width), expand_safe(min.height)),
            max: Size::new(expand_safe(max.width), expand_safe(max.height)),
        }
    }

    pub fn new_max(max: Size) -> LayoutConstraints {
        LayoutConstraints::new(Size::ZERO, max)
    }

    pub fn tight(size: Size) -> LayoutConstraints {
        LayoutConstraints::new(size, size)
    }

    pub fn get_min(&self) -> Size {
        self.min
    }
    pub fn get_max(&self) -> Size {
        self.max
    }

    pub fn deset(&self, size: Size) -> LayoutConstraints {
        LayoutConstraints::new(self.min - size * 2.0, self.max - size * 2.0)
    }

    pub fn inset(&self, size: Size) -> LayoutConstraints {
        LayoutConstraints::new(self.min + size * 2.0, self.max + size * 2.0)
    }

    pub fn max_advance(&self) -> Option<f32> {
        self.max.width.is_finite().then_some(self.max.width as f32)
    }

    pub fn min_clamp(&self, min: Size) -> LayoutConstraints {
        LayoutConstraints::new(
            self.min.clamp(min, Size::new(f64::INFINITY, f64::INFINITY)),
            self.max.clamp(min, Size::new(f64::INFINITY, f64::INFINITY)),
        )
    }

    pub fn max_clamp(&self, max: Size) -> LayoutConstraints {
        LayoutConstraints::new(
            self.min.clamp(Size::new(0.0, 0.0), max),
            self.max.clamp(Size::new(0.0, 0.0), max),
        )
    }

    pub fn map<F>(&self, f: F) -> LayoutConstraints
    where
        F: Fn(Size) -> Size,
    {
        LayoutConstraints::new(f(self.min), f(self.max))
    }
}

#[cfg(test)]
mod layout_constraints_tests {
    use super::*;
    use vello::kurbo::Size;

    #[test]
    fn layout_constraints_new() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        assert_eq!(constraints.get_min(), Size::new(10.0, 20.0));
        assert_eq!(constraints.get_max(), Size::new(30.0, 40.0));
    }

    #[test]
    #[should_panic(expected = "min width greater than max width")]
    fn layout_constraints_new_invalid_width() {
        let _ = LayoutConstraints::new(Size::new(50.0, 20.0), Size::new(30.0, 40.0));
    }

    #[test]
    #[should_panic(expected = "min height greater than max height")]
    fn layout_constraints_new_invalid_height() {
        let _ = LayoutConstraints::new(Size::new(10.0, 50.0), Size::new(30.0, 40.0));
    }

    #[test]
    fn layout_constraints_new_max() {
        let constraints = LayoutConstraints::new_max(Size::new(30.0, 40.0));
        assert_eq!(constraints.get_min(), Size::ZERO);
        assert_eq!(constraints.get_max(), Size::new(30.0, 40.0));
    }

    #[test]
    fn layout_constraints_tight() {
        let constraints = LayoutConstraints::tight(Size::new(30.0, 40.0));
        assert_eq!(constraints.get_min(), Size::new(30.0, 40.0));
        assert_eq!(constraints.get_max(), Size::new(30.0, 40.0));
    }

    #[test]
    fn layout_constraints_deset() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        let deset_constraints = constraints.deset(Size::new(5.0, 5.0));
        assert_eq!(deset_constraints.get_min(), Size::new(0.0, 10.0));
        assert_eq!(deset_constraints.get_max(), Size::new(20.0, 30.0));
    }

    #[test]
    fn layout_constraints_inset() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        let inset_constraints = constraints.inset(Size::new(5.0, 5.0));
        assert_eq!(inset_constraints.get_min(), Size::new(20.0, 30.0));
        assert_eq!(inset_constraints.get_max(), Size::new(40.0, 50.0));
    }

    #[test]
    fn layout_constraints_max_advance() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        assert_eq!(constraints.max_advance(), Some(30.0f32));
    }

    #[test]
    fn layout_constraints_min_clamp() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        let clamped_constraints = constraints.min_clamp(Size::new(15.0, 25.0));
        assert_eq!(clamped_constraints.get_min(), Size::new(15.0, 25.0));
        assert_eq!(clamped_constraints.get_max(), Size::new(30.0, 40.0));
    }

    #[test]
    fn layout_constraints_max_clamp() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        let clamped_constraints = constraints.max_clamp(Size::new(25.0, 35.0));
        assert_eq!(clamped_constraints.get_min(), Size::new(10.0, 20.0));
        assert_eq!(clamped_constraints.get_max(), Size::new(25.0, 35.0));
    }

    #[test]
    fn layout_constraints_map() {
        let constraints = LayoutConstraints::new(Size::new(10.0, 20.0), Size::new(30.0, 40.0));
        let mapped_constraints = constraints.map(|size| size * 2.0);
        assert_eq!(mapped_constraints.get_min(), Size::new(20.0, 40.0));
        assert_eq!(mapped_constraints.get_max(), Size::new(60.0, 80.0));
    }
}
