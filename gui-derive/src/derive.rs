use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use std::env;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Fields};

#[derive(Clone, Debug)]
pub struct Derive {
    component_ident: Ident,
    component: String,
    vars_to_gen: Vec<(String, Ident)>,
    components_to_gen: Vec<(String, Ident)>,
}

impl Parse for Derive {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = stream.parse()?;
        let expected_component_name = format!("{}", input.ident);

        match input.data {
            Data::Struct(s) => {
                let env = env::var("GUI_COMPONENTS").unwrap();
                let component: String = env
                    .split(',')
                    .find(|c| c == &expected_component_name)
                    .ok_or_else(|| {
                        syn::Error::new(
                            input.ident.span(),
                            format!("Could not find component named {expected_component_name}"),
                        )
                    })?
                    .into();

                if let Fields::Named(fields) = s.fields {
                    let env_vars = env::var(format!("GUI_COMPONENT_{component}_VAR")).unwrap();
                    let component_vars: HashSet<&str> = env_vars.split(',').collect();
                    let env_component =
                        env::var(format!("GUI_COMPONENT_{component}_COMPONENT")).unwrap();
                    let components: HashSet<&str> = env_component.split(',').collect();

                    let fields_iter = fields
                        .named
                        .iter()
                        .filter_map(|f| f.ident.as_ref())
                        .map(|i| (format!("{i}"), i.clone()));

                    let vars_to_gen = fields_iter
                        .clone()
                        .filter(|(s, _i)| component_vars.contains(s.as_str()))
                        .collect();

                    let components_to_gen = fields_iter
                        .filter(|(s, _i)| components.contains(s.as_str()))
                        .collect();

                    Ok(Self {
                        component_ident: input.ident,
                        component,
                        vars_to_gen,
                        components_to_gen,
                    })
                } else {
                    Ok(Self {
                        component_ident: input.ident,
                        component,
                        vars_to_gen: vec![],
                        components_to_gen: vec![],
                    })
                }
            }
            Data::Enum(e) => Err(syn::Error::new(
                e.enum_token.span,
                "The derive macro does not currently support enums.",
            )),
            Data::Union(u) => Err(syn::Error::new(
                u.union_token.span,
                "The derive macro does not currently support unions.",
            )),
        }
    }
}

impl ToTokens for Derive {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let component_file = format!("/{}.rs", self.component);
        let component_ident = &self.component_ident;

        let gen_vars = self.vars_to_gen.iter().map(|(v_name, ident)| {
            let var_ident = Ident::new(v_name, ident.span());
            quote! {
                impl ::gui::Update<gen::#var_ident> for #component_ident {
                    fn is_updated(&self) -> bool {
                        self.#ident.is_updated()
                    }
                    fn value(&self) -> <gen::#var_ident as ::gui::gui_core::Variable>::VarType {
                        self.#ident.value()
                    }
                    fn reset(&mut self) {
                        self.#ident.reset();
                    }
                }
            }
        });

        let gen_components = self.components_to_gen.iter().map(|(c_name, ident)| {
            let comp_ident = Ident::new(c_name, ident.span());
            quote! {
                impl ::gui::gui_core::ComponentHolder<gen::#comp_ident> for #component_ident {
                    fn comp_holder(&mut self) -> &mut ::gui::CompHolder<<gen::#comp_ident as ::gui::gui_core::Variable>::VarType> {
                        &mut self.#ident
                    }
                }
            }
        });

        tokens.extend(quote! {
            use #component_ident as __private_CompStruct;
            include!(concat!(env!("OUT_DIR"), #component_file));
            #(#gen_vars)*
            #(#gen_components)*
        })
    }
}
