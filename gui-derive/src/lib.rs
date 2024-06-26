use proc_macro::TokenStream;

use quote::ToTokens;
use syn::parse_macro_input;

use crate::derive::Derive;
use crate::type_registry::TypeRegistry;
use crate::widget_builder::WidgetBuilder;

mod derive;
mod type_registry;
mod widget_builder;

/// Add this to all user-defined components to associate them with their layout file equivalents.
#[proc_macro_derive(ToComponent)]
pub fn derive_to_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Derive);
    input.to_token_stream().into()
}

/// Put this in the crate root.
#[proc_macro]
pub fn type_registry(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as TypeRegistry);
    input.to_token_stream().into()
}

/// Add this to all custom widget builders you want to create.
///
/// # Attributes
///
/// ## Container Attributes
///
/// Attributes are added using `#[widget]`. Each derive invocation must contain a container attribute:
/// `#[widget(name = "widgetName", type_path = "type_path", init_path = "init_path")]` where:
/// - `name` describes the name of the widget (it will be automatically prefixed with the name of your library in the form
/// `"<crate>::<name>"`). This is used for typetag deserialization and widget naming for debug printing.
/// - `type_path` which is the type of the runtime widget (of the form
/// `::crate_name::widget_struct<type_param1, type_param2>`). 3 different parameters can be used
/// (`#handler` (a type that implements `ToHandler`), `#component` (a type that implements
/// `ToComponent`) and `#child` (the type of the children this widget can hold))
/// - `init_path` (of the form `new`) which describes how the runtime widget should be created
/// of type `fn(WidgetId) -> RuntimeWidget`
///
/// ## Field Attributes
///
/// Defaults are not required and are only available with `property` or `static_only`.
/// - `default = expression` of type `T` where `T: ToTokens`
/// - `default_with = "path_to_function"` of type `fn(&WidgetBuilder) -> T` where `T: ToTokens`
/// - `<property|static_only|var_only> = "path_to_function"` of type `fn(&mut RuntimeWidget, T, &mut UpdateHandle) -> ()`.
/// Should you get an error informing you that a type could not be inferred please add the `bound` attribute.
/// - `<property|static|var|>bound = "T: Trait"` used for type assertions to ensure that the given
/// function can deal with all the types declared by the bound.
/// - `fluent = "path_to_function"` of type `fn<'a>(&mut RuntimeWidget, Cow<'a, str>, &mut UpdateHandle) -> ()`
/// - `component = "path_to_function"` of type `fn(&mut RuntimeWidget, WidgetId, &mut UpdateHandle) -> ()`
/// - `child = "path_to_function"` of type `fn(&mut RuntimeWidget) -> &mut Child`
/// - `children = "path_to_function"` of type `fn(&mut RuntimeWidget, usize) -> &mut Child`
///
#[proc_macro_derive(WidgetBuilder, attributes(widget))]
pub fn derive_widget_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WidgetBuilder);
    input.to_token_stream().into()
}
