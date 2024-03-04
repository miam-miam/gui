use gui_core::parse::fluent::Fluent;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FluentIdent {
    pub fluent: Fluent,
    pub ident: Ident,
    pub name: String,
    pub property: &'static str,
    pub property_ident: Ident,
}

impl FluentIdent {
    pub fn new(
        property: &'static str,
        fluent: Fluent,
        component_name: &str,
        widget_name: Option<&str>,
        widget_type_name: &str,
    ) -> Self {
        let fluent_widget_name = widget_name.unwrap_or(widget_type_name);
        Self {
            fluent,
            property,
            ident: format_ident!("{component_name}_{fluent_widget_name}_{property}"),
            name: format!("{component_name}-{fluent_widget_name}-{property}"),
            property_ident: Ident::new(property, Span::call_site()),
        }
    }

    pub fn new_state_override(
        property: &'static str,
        fluent: Fluent,
        component_name: &str,
        widget_name: &str,
        state_name: &str,
    ) -> Self {
        Self {
            fluent,
            property,
            ident: format_ident!("{component_name}_{widget_name}_{state_name}_{property}"),
            name: format!("{component_name}-{widget_name}-{state_name}-{property}"),
            property_ident: Ident::new(property, Span::call_site()),
        }
    }
}

pub fn gen_bundle_function() -> TokenStream {
    quote! {
        use gui::{FluentBundle, FluentArgs, FluentResource};
        use std::borrow::Cow;

        fn get_bundle_message<'a>(message: &'a str, args: Option<&'a FluentArgs<'_>>) -> Cow<'a, str> {
            use std::sync::OnceLock;
            use gui::langid;

            static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
            const FTL_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/Counter.ftl"));
            let mut errors = vec![];
            let bundle = BUNDLE.get_or_init(|| {
                let mut bundle = FluentBundle::new_concurrent(vec![langid!("en-GB")]);
                let resource = FluentResource::try_new(FTL_STRING.to_string())
                    .expect("FTL string is valid.");
                bundle.add_resource(resource).expect("No identifiers are overlapping.");
                bundle
            });
            let message = bundle.get_message(message).expect("Message exists.");
            let pattern = message.value().expect("Value exists.");
            bundle.format_pattern(pattern, args, &mut errors)
        }
    }
}
