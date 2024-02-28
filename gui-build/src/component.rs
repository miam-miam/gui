use crate::fluent;
use crate::fluent::FluentIdent;
use crate::widget::Widget;
use anyhow::Context;
use gui_core::parse::{
    ComponentDeclaration, NormalVariableDeclaration, StateDeclaration, VariableDeclaration,
};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub fn create_component(out_dir: &Path, component: &ComponentDeclaration) -> anyhow::Result<()> {
    let component_holder = format_ident!("{}Holder", *component.name);

    let normal_variables: Vec<&NormalVariableDeclaration> = component
        .variables
        .iter()
        .filter_map(|v| match v {
            VariableDeclaration::Normal(n) => Some(n),
            _ => None,
        })
        .collect();

    let widget_tree = Widget::new(component)?;

    let mut widget_set = TokenStream::new();
    widget_tree.gen_widget_set(&mut widget_set);

    let bundle_func = widget_tree
        .contains_fluents()
        .then(fluent::gen_bundle_function);

    let mut fluents = vec![];
    widget_tree.push_fluents(&mut fluents);

    create_bundle(out_dir, &component.name, &fluents[..])
        .context("Failed to create fluent bundle")?;

    let if_update: TokenStream = normal_variables
        .iter()
        .map(|n| widget_tree.gen_var_update(n))
        .collect();

    let fluent_arg_idents: Vec<&Ident> = fluents
        .iter()
        .filter(|f| !f.fluent.vars.is_empty())
        .map(|fluent| &fluent.ident)
        .collect();

    let fluent_properties: Vec<&Ident> = fluents
        .iter()
        .filter(|f| !f.fluent.vars.is_empty())
        .map(|fluent| &fluent.property_ident)
        .collect();

    let mut prop_update: TokenStream = TokenStream::new();
    widget_tree.gen_fluent_update(None, &mut prop_update);

    let struct_vars: TokenStream = normal_variables
        .iter()
        .map(|n| {
            let name = Ident::new(&n.name, Span::call_site());
            let var_type = TokenStream::from_str(&n.var_type).expect("a valid type");
            quote! {
                #[allow(non_camel_case_types)]
                pub(crate) struct #name;

                impl Variable for #name {
                    type VarType = #var_type;
                }
            }
        })
        .collect();

    let var_names = normal_variables
        .iter()
        .map(|n| Ident::new(&n.name, Span::call_site()));

    let state_declaration = create_state(component.states.as_slice());

    let mut struct_handlers = TokenStream::new();
    widget_tree.gen_handler_structs(&mut struct_handlers)?;

    let rs_path = Path::new(&out_dir).join(format!("{}.rs", component.name.as_str()));

    let widget_type = widget_tree.gen_widget_type();
    let widget_init = widget_tree.gen_widget_init();

    let largest_id = widget_tree.get_largest_id();

    let mut id_to_widgets = vec![];
    widget_tree.gen_widget_id_to_widget(None, &mut id_to_widgets);
    let event_match_arms = id_to_widgets.iter().map(|(id, widget_get)| {
        let component = id.component_id();
        let widget = id.widget_id();
        quote!((#component, #widget) => {#widget_get.event(event, handle_ref);})
    });

    let mut parent_ids = vec![];
    widget_tree.get_parent_ids(&mut parent_ids);
    let parent_match_arms = parent_ids.iter().map(|(parent, children)| {
        let unwrapped_vals = children.iter().map(|id| {
            let component = id.component_id();
            let widget = id.widget_id();
            quote!((#component, #widget))
        });
        quote!(#( #unwrapped_vals )|* => Some(#parent),)
    });

    let named_match_arms = widget_tree.iter().filter_map(|w| {
        let name = w.widget_declaration.name.as_ref()?.as_str();
        let id = w.id;
        Some(quote!(#name => Some(#id),))
    });

    let gen_module = quote! {
        #[allow(clippy::suspicious_else_formatting)]
        mod gen {
            use super::__private_CompStruct as CompStruct;
            use std::any::Any;
            use gui::gui_core::vello::SceneBuilder;
            use gui::gui_core::glazier::kurbo::Rect;
            use gui::gui_core::widget::{Widget, WidgetID, RenderHandle, ResizeHandle, EventHandle, UpdateHandle, WidgetEvent, Handle};
            use gui::gui_core::{Component, LayoutConstraints, Size, ToComponent, ToHandler, Update, Variable};

            #state_declaration

            #widget_set

            #bundle_func

            #struct_vars

            #struct_handlers

            #[allow(non_snake_case)]
            pub struct #component_holder {
                comp_struct: CompStruct,
                widget: #widget_type,
                #( #fluent_arg_idents: FluentArgs<'static> ),*
            }

            #[automatically_derived]
            impl ToComponent for CompStruct {
                type Component = #component_holder;

                fn to_component_holder(self) -> Self::Component {
                    #component_holder {
                        widget: #widget_init,
                        comp_struct: self,
                        #( #fluent_arg_idents: FluentArgs::new() ),*
                    }
                }

                fn largest_id(&self) -> WidgetID {
                    #largest_id
                }

                fn get_parent(&self, id: WidgetID) -> Option<WidgetID> {
                    match (id.component_id(), id.widget_id()) {
                        #(#parent_match_arms)*
                        _ => None,
                    }
                }

                fn get_id(&self, name: &str) -> Option<WidgetID> {
                    match name {
                        #(#named_match_arms)*
                        _ => None,
                    }
                }
            }

            #[automatically_derived]
            impl Component for #component_holder {
                fn render<'a>(
                    &mut self,
                    mut scene: SceneBuilder,
                    handle: &'a mut Handle,
                    global_positions: &'a mut [Rect],
                    active_widget: &'a mut Option<WidgetID>,
                    hovered_widgets: &'a [WidgetID],
                ) -> bool {
                    let mut render_handle = RenderHandle::new(handle, global_positions, active_widget, hovered_widgets, &mut self.comp_struct);
                    self.widget.render(&mut scene, &mut render_handle);
                    render_handle.unwrap()
                }

                fn update_vars<'a>(
                    &mut self,
                    force_update: bool,
                    handle: &'a mut Handle,
                    global_positions: &'a [Rect],
                ) -> bool {
                    let mut update_handle = UpdateHandle::new(handle, global_positions);
                    let handle_ref = &mut update_handle;
                    #( let mut #fluent_properties = false; )*
                    #if_update
                    #prop_update
                    #( <CompStruct as Update<#var_names>>::reset(&mut self.comp_struct); )*
                    update_handle.unwrap()
                }

                fn resize<'a>(
                    &mut self,
                    constraints: LayoutConstraints,
                    handle: &'a mut Handle,
                    local_positions: &'a mut [Rect],
                ) -> Size {
                    let mut resize_handle = ResizeHandle::new(handle, local_positions, &mut self.comp_struct);
                    self.widget.resize(constraints, &mut resize_handle)
                }

                fn propagate_event<'a>(
                    &mut self,
                    event: WidgetEvent,
                    handle: &'a mut Handle,
                    global_positions: &'a [Rect],
                    active_widget: &'a mut Option<WidgetID>,
                    hovered_widgets: &'a mut Vec<WidgetID>,
                ) -> bool {
                    let mut event_handle = EventHandle::new(handle, global_positions, active_widget, hovered_widgets, &mut self.comp_struct);
                    self.widget.event(event, &mut event_handle);
                    let (mut resize, events) = event_handle.unwrap();
                    for (id, e) in events {
                        if self.event(id, e, handle, global_positions, active_widget, hovered_widgets) {
                            resize = true;
                        }
                    }
                    resize
                }

                fn largest_id(&self) -> WidgetID {
                    self.comp_struct.largest_id()
                }

                fn get_parent(&self, id: WidgetID) -> Option<WidgetID> {
                    self.comp_struct.get_parent(id)
                }

                fn get_id(&self, name: &str) -> Option<WidgetID> {
                    self.comp_struct.get_id(name)
                }

                fn get_comp_struct(&mut self) -> &mut dyn Any {
                    &mut self.comp_struct
                }

                fn event<'a>(
                    &mut self,
                    id: WidgetID,
                    event: WidgetEvent,
                    handle: &'a mut Handle,
                    global_positions: &'a [Rect],
                    active_widget: &'a mut Option<WidgetID>,
                    hovered_widgets: &'a mut Vec<WidgetID>,
                ) -> bool {
                    let mut event_handle = EventHandle::new(handle, global_positions, active_widget, hovered_widgets, &mut self.comp_struct);
                    let handle_ref = &mut event_handle;
                    match (id.component_id(), id.widget_id()) {
                        #(#event_match_arms)*
                        _ => {},
                    }
                    let (mut resize, events) = event_handle.unwrap();
                    for (id, e) in events {
                        if self.event(id, e, handle, global_positions, active_widget, hovered_widgets) {
                            resize = true;
                        }
                    }
                    resize
                }
            }
        }
    };

    write_file(&rs_path, gen_module)
}

#[cfg(not(feature = "pretty"))]
fn write_file(path: &Path, stream: TokenStream) -> anyhow::Result<()> {
    fs::write(path, format!("{}", stream))?;
    Ok(())
}
#[cfg(feature = "pretty")]
fn write_file(path: &Path, stream: TokenStream) -> anyhow::Result<()> {
    let file = syn::parse2::<syn::File>(stream.clone()).inspect_err(|_| {
        let _ = fs::write(path, stream.to_string());
    })?;

    fs::write(path, prettyplease::unparse(&file))?;
    Ok(())
}

fn create_bundle(
    out_dir: &Path,
    component_name: &str,
    fluents: &[FluentIdent],
) -> anyhow::Result<()> {
    let ftl_path = out_dir.join(format!("{component_name}.ftl"));
    let mut bundle = String::new();
    for fluent in fluents {
        bundle = bundle + &format!("{} = {}\n", fluent.name, fluent.fluent.text);
    }
    fs::write(ftl_path, bundle)?;
    Ok(())
}

fn create_state(states: &[StateDeclaration]) -> TokenStream {
    if states.is_empty() {
        return TokenStream::new();
    }

    let names = states.iter().map(|s| format_ident!("{}", s.name.as_str()));

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Default, Copy, Clone, Eq, PartialEq)]
        pub(crate) enum State {
            #[default]
            #(#names),*
        }
        #[allow(non_camel_case_types)]
        pub(crate) struct state;

        impl Variable for state {
            type VarType = State;
        }
    }
}
