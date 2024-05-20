pub use typetag;

pub use gui_core::{Component, MutWidgetChildren, ToComponent, ToHandler, WidgetChildren};
pub use gui_core::widget;
#[doc(inline)]
pub use gui_derive::WidgetBuilder;

/// Used by generated code. Not public api
#[doc(hidden)]
#[path = "private"]
pub mod __private {
    pub mod assertions;
    pub mod fakes;
}

