use std::ops::Not;

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Data, DeriveInput, Error, Fields, Generics, Path};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use field_attributes::FieldAttributes;

use crate::widget_builder::attributes::StructAttributes;
use crate::widget_builder::field_attributes::{Extension, Property, StaticDefault};
use crate::widget_builder::interpolated_path::InterpolatedType;

mod attributes;
mod field_attributes;
mod interpolated_path;

#[derive(Clone)]
pub struct WidgetBuilder {
    name: Ident,
    generics: Generics,
    attributes: StructAttributes,
    fields: Vec<FieldAttributes>,
}

impl Parse for WidgetBuilder {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = stream.parse()?;

        match input.data {
            Data::Struct(s) => {
                if let Fields::Named(fields) = s.fields {
                    let attributes = StructAttributes::new(&input.attrs)?;
                    let fields = fields
                        .named
                        .into_iter()
                        .map(FieldAttributes::new)
                        .collect::<syn::Result<_>>()?;
                    Ok(WidgetBuilder {
                        name: input.ident,
                        generics: input.generics,
                        attributes,
                        fields,
                    })
                } else {
                    Err(Error::new(
                        s.fields.span(),
                        "The derive macro only supports named properties",
                    ))
                }
            }
            Data::Enum(e) => Err(Error::new(
                e.enum_token.span,
                "The derive macro does not currently support enums.",
            )),
            Data::Union(u) => Err(Error::new(
                u.union_token.span,
                "The derive macro does not currently support unions.",
            )),
        }
    }
}

impl WidgetBuilder {
    fn combine_func(&self) -> TokenStream {
        let fields = self
            .fields
            .iter()
            .map(|f| f.field.ident.as_ref().unwrap())
            .collect_vec();
        quote! {
            fn combine(&mut self, rhs: &dyn WidgetBuilder) {
                if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
                    #(
                        if let Some(p) = &other. #fields {
                            self. #fields .get_or_insert_with(|| p.clone());
                        }
                    )*
                }
            }
        }
    }

    fn get_property_names(&self) -> Vec<(&Ident, Extension, Option<&StaticDefault>, &Path)> {
        self.fields
            .iter()
            .flat_map(|f| {
                let ident = f.field.ident.as_ref().unwrap();
                f.property_names()
                    .into_iter()
                    .map(move |(ext, default, path)| (ident, ext, default, path))
            })
            .collect_vec()
    }

    fn property_func(
        &self,
        property_names: &[(&Ident, Extension, Option<&StaticDefault>, &Path)],
    ) -> Option<TokenStream> {
        let widget = format_ident!("widget");
        let value = format_ident!("value");
        let handle = format_ident!("handle");
        let fields: TokenStream = property_names
            .iter()
            .map(|(ident, ext, _, path)| {
                let property = ext.form_prop_name(ident);
                quote! {
                    #property => stream.extend(quote!( ##widget . #path (##value, ##handle); )),
                }
            })
            .collect();

        fields.is_empty().not().then(|| quote! {
            fn on_property_update(&self, property: &'static str, widget: &Ident, value: &Ident, handle: &Ident, stream: &mut TokenStream) {
                match property {
                    #fields
                    _ => {}
                }
            }
        })
    }

    fn widget_funcs(&self) -> Option<TokenStream> {
        let children_mut: TokenStream = self
            .fields
            .iter()
            .filter_map(|f| f.children.as_ref().map(|_| f.field.ident.as_ref().unwrap()))
            .map(|property| {
                quote! {
                    if !self. #property . is_empty() {
                        result.push(Children::Many(self . #property .iter_mut().collect()));
                    }
                }
            })
            .collect();

        let child_mut: TokenStream = self
            .fields
            .iter()
            .filter_map(|f| f.child.as_ref().map(|_| f.field.ident.as_ref().unwrap()))
            .map(|property| {
                quote! {
                    if let Some(c) = &mut self. #property {
                        result.push(Children::One(c));
                    }
                }
            })
            .collect();

        let children_ref: TokenStream = self.fields.iter().filter_map(|f| f.children.as_ref().map(|c| (f.field.ident.as_ref().unwrap(), c))).map(|(property, path)| {
            quote! {
                if !self. #property . is_empty() {
                    result.push(( quote!( . #path () ), Children::Many(self . #property .iter().collect())));
                }
            }
        }).collect();

        let child_ref: TokenStream = self
            .fields
            .iter()
            .filter_map(|f| {
                f.child
                    .as_ref()
                    .map(|c| (f.field.ident.as_ref().unwrap(), c))
            })
            .map(|(property, path)| {
                quote! {
                    if let Some(c) = &self. #property {
                        result.push(( quote!( . #path () ), Children::One(c)));
                    }
                }
            })
            .collect();

        if children_ref.is_empty() && child_ref.is_empty() {
            return None;
        }

        Some(quote! {
            fn get_widgets(&mut self) -> Option<Vec<MutWidgetChildren>> {
                let mut result = vec![];
                #children_mut
                #child_mut
                result
            }

            fn widgets(&self) -> Option<Vec<(TokenStream, WidgetChildren)>> {
                let mut result = vec![];
                #children_ref
                #child_ref
                result
            }
        })
    }

    fn statics_func(
        &self,
        property_names: &[(&Ident, Extension, Option<&StaticDefault>, &Path)],
    ) -> Option<TokenStream> {
        let statics: TokenStream = property_names
            .iter()
            .filter(|(_, e, ..)| Property::from(e).is_static())
            .map(|(property_ident, ext, default, _)| {
                let property_name = ext.form_prop_name(property_ident);
                let binding = if let Extension::Unnecessary(Property::Both) | Extension::Static(true) = ext {
                    quote!(Var::Value(v))
                } else {
                    quote!(v)
                };

                let default = default.map(|d| match d {
                    StaticDefault::Expression(e) => quote! {
                        None => result.push((#property_name, (#e).to_token_stream()))
                    },
                    StaticDefault::Function(f) => quote! {
                        None => result.push((#property_name, self . #f ()))
                    },
                });

                quote! {
                    match &self . #property_ident {
                        Some(#binding) => result.push((#property_name, v.to_token_stream())),
                        #default
                        _ => {}
                    }
                }
            })
            .collect();

        statics.is_empty().not().then(|| {
            quote! {
                fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
                    let mut result = vec![];
                    #statics
                    result
                }
            }
        })
    }

    fn fluents_func(
        &self,
        property_names: &[(&Ident, Extension, Option<&StaticDefault>, &Path)],
    ) -> Option<TokenStream> {
        let fluents: TokenStream = property_names
            .iter()
            .filter(|(_, e, ..)| Property::from(e) == Property::Fluent)
            .map(|(property, ext, ..)| {
                let property_name = ext.form_prop_name(property);
                quote! {
                    result.extend(self . #property .iter().map(|f| ( #property_name, f.clone() )));
                }
            })
            .collect();

        fluents.is_empty().not().then(|| {
            quote! {
                fn get_fluents(&self) -> Vec<(&'static str, Fluent)> {
                    let mut result = vec![];
                    #fluents
                    result
                }
            }
        })
    }

    fn vars_func(
        &self,
        property_names: &[(&Ident, Extension, Option<&StaticDefault>, &Path)],
    ) -> Option<TokenStream> {
        let vars: TokenStream = property_names
            .iter()
            .filter(|(_, e, ..)| Property::from(e).is_var())
            .map(|(property, ext, ..)| {
                let property_name = ext.form_prop_name(property);

                let binding = if let Extension::Unnecessary(Property::Both) | Extension::Var(true) = ext {
                    quote!(Var::Variable(v))
                } else {
                    quote!(v)
                };

                quote! {
                    if let Some(#binding) = &self. #property {
                        result.push(( #property_name, v.clone() ));
                    }
                }
            })
            .collect();

        vars.is_empty().not().then(|| {
            quote! {
                fn get_vars(&self) -> Vec<(&'static str, Name)> {
                    let mut result = vec![];
                    #vars
                    result
                }
            }
        })
    }

    fn components_func(
        &self,
        property_names: &[(&Ident, Extension, Option<&StaticDefault>, &Path)],
    ) -> Option<TokenStream> {
        let components: TokenStream = property_names
            .iter()
            .filter(|(_, ext, ..)| Property::from(ext) == Property::Component)
            .map(|(property, ext, ..)| {
                let property_name = ext.form_prop_name(property);
                quote! {
                    if let Some(c) = &self. #property {
                        result.push(( #property_name, c.clone() ));
                    }
                }
            })
            .collect();

        components.is_empty().not().then(|| {
            quote! {
                fn get_components(&self) -> Vec<(&'static str, ComponentVar)> {
                    let mut result = vec![];
                    #components
                    result
                }
            }
        })
    }
}

impl ToTokens for WidgetBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let builder_name = &self.name;
        let widget_name = &self.attributes.widget_name;
        let type_path = &self.attributes.type_path;
        let mut erased_type_path = self.attributes.type_path.clone();
        if let Some(generics) = &mut erased_type_path.generics {
            generics.args.iter_mut().for_each(InterpolatedType::erase_interpolated)
        }
        let init_path = &self.attributes.init_path;
        let id = format_ident!("id");
        let combine_func = self.combine_func();

        let property_names = self.get_property_names();
        let property_func = self.property_func(&property_names);
        let statics_func = self.statics_func(&property_names);
        let fluents_func = self.fluents_func(&property_names);
        let vars_func = self.vars_func(&property_names);
        let components_func = self.components_func(&property_names);

        let widget_funcs = self.widget_funcs();
        let has_handler_func = self.attributes.has_handler.then(|| {
            quote! {
                fn has_handler(&self) -> bool {
                    true
                }
            }
        });

        tokens.extend(quote! {
            #[typetag::deserialize(name = #widget_name)]
            impl #impl_generics WidgetBuilder for #builder_name #ty_generics #where_clause {
                fn widget_type(&self, handler: Option<&Ident>, component: &Ident, child: Option<&TokenStream>, stream: &mut TokenStream) {
                    stream.extend(quote!(#type_path))
                }

                fn name(&self) -> &'static str {
                    #widget_name
                }

                #combine_func

                fn create_widget(&self, id: WidgetID, stream: &mut TokenStream) {
                    stream.extend(quote!(#erased_type_path :: #init_path (##id)))
                }

                #property_func
                #statics_func
                #fluents_func
                #vars_func
                #has_handler_func
                #components_func

                #widget_funcs
            }
        })
    }
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use crate::widget_builder::WidgetBuilder;

    #[test]
    pub fn parse_attributes() {
        let builder: WidgetBuilder = parse_quote! {
            #[widget(name = "test", init_path = "new", type_path = "::gui::Test")]
            struct TestBuilder {
                #[widget(property = "set_prop_a", default = 5)]
                prop_a: Option<Var<u8>>
            }
        };
        let multi_attributes: WidgetBuilder = parse_quote! {
            #[widget(name = "test", type_path = "::gui::Test")]
            #[widget(init_path = "new")]
            struct TestBuilder {
                #[widget(property = "set_prop_a")]
                #[widget(default = 5)]
                prop_a: Option<Var<u8>>
            }
        };

        assert_eq!(builder.attributes, multi_attributes.attributes);
        assert_eq!(builder.fields, multi_attributes.fields);
    }
}
