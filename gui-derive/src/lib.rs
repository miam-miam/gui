mod derive;
mod type_registry;

use crate::derive::Derive;
use crate::type_registry::TypeRegistry;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

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
/// `#[widget(name = "widgetName", type = "type_path", init = "init_path")]` where:
/// - `name` describes the name of the widget (prefixed with the name of your library in the form
/// <crate>::<name>). This is used for typetag deserialization and widget naming for debug printing.
/// - `type_path` which is the type of the runtime widget (of the form
/// `::crate_name::widget_struct<type_param1, type_param2>`). 3 different parameters can be used
/// (`#handler` (a type that implements `ToHandler`), `#component` (a type that implements
/// `ToComponent`) and `#child` (the type of the children this widget can hold))
/// - `init_path` (of the form `new(#id)`) which describes how the runtime widget should be created.
/// The `#id` parameter must be used, any children field attributes can also be used
///
/// ## Field Attributes
///
/// Defaults are not required and are only available with `property` or `static_only`.
/// - `default = "const_expression"`
/// - `default_with = "path_to_function"` of type `fn(&WidgetBuilder) -> T` where `T: ToTokens`
/// - `<property|static_only|var_only> = "path_to_function"` of type `fn(&mut RuntimeWidget, T, &mut UpdateHandle) -> ()` where `T: ToTokens`
/// - `fluent = "path_to_function"` of type `fn<'a>(&mut RuntimeWidget, Cow<'a, str>, &mut UpdateHandle) -> ()`
/// - `component = "path_to_function"` of type `fn(&mut RuntimeWidget, WidgetId, &mut UpdateHandle) -> ()`
/// - `child = "path_to_function"` of type `fn(&mut RuntimeWidget) -> &mut Child`
/// - `children = "path_to_function"` of type `fn(&mut RuntimeWidget, usize) -> &mut Child`
///
#[proc_macro_derive(WidgetBuilder, attributes(widget))]
pub fn derive_widget_builder(input: TokenStream) -> TokenStream {
    input
}
