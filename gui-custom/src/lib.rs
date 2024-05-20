pub use gui_core::*;
#[doc(inline)]
pub use gui_derive::WidgetBuilder;

/// Used by generated code. Not public api
#[doc(hidden)]
#[path = "private"]
pub mod __private {
    pub use proc_macro2::Ident;
    pub use proc_macro2::TokenStream;
    pub use quote::quote;
    pub use quote::ToTokens;

    pub use gui_core::parse::fluent::Fluent;
    pub use gui_core::parse::var::{ComponentVar, Name};
    pub use gui_core::widget::{WidgetBuilder, WidgetID};
    pub use gui_core::{Children, MutWidgetChildren, WidgetChildren};

    pub mod assertions;
    pub mod fakes;
}
