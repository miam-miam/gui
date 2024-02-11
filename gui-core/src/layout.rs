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
