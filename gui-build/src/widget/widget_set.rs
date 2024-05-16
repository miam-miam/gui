use std::sync::atomic::{AtomicU32, Ordering};

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use gui_core::{Children, WidgetChildren};
use gui_core::parse::StateDeclaration;

use crate::widget::Widget;

/// A collection of widgets that can be retrieved using their associated [`TokenStream`].
/// A widget set is only created if there is more than one widget stored.
#[derive(Clone, Debug)]
pub struct WidgetSet<'a> {
    pub widgets: Vec<(TokenStream, Children<(u32, Widget<'a>)>)>,
    /// None if the widget length is smaller or equal to 1. Each count is unique to
    /// guarantee that multiple WidgetSet implementations are not accidentally created.
    count: Option<u32>,
}

impl<'a> WidgetSet<'a> {
    pub fn new(
        component_name: &str,
        widgets: Vec<(TokenStream, WidgetChildren<'a>)>,
        states: &'a [StateDeclaration],
        component_id: u32,
    ) -> anyhow::Result<Self> {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let mut i = 0;

        let widgets = widgets
            .into_iter()
            .filter(|(_, w)| !w.is_empty())
            .map(|(s, w)| {
                let widget = w.try_map(|w| {
                    i += 1;
                    Ok::<_, anyhow::Error>((
                        i - 1,
                        Widget::new_inner(component_name, w, states, component_id)?,
                    ))
                })?;
                Ok((s, widget))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self {
            count: (i > 1).then(|| COUNTER.fetch_add(1, Ordering::Relaxed)),
            widgets,
        })
    }

    pub fn gen_widget_type(&self) -> TokenStream {
        match &self.widgets[..] {
            [(_, Children::One((_, child)))] => child.gen_widget_type(),
            [(_, Children::Many(v))] if v.len() == 1 => v[0].1.gen_widget_type(),
            [] => quote!(()),
            _ => {
                let count = self.count.expect("widget set should be created.");
                let ident = format_ident!("WidgetSet{count}");
                quote!(#ident)
            }
        }
    }

    pub fn gen_widget_init(&self) -> TokenStream {
        match &self.widgets[..] {
            [(s, Children::One((_, child)))] => {
                let stream = child.gen_widget_init();
                quote!(*widget #s = Some(#stream))
            }
            [(s, Children::Many(v))] if v.len() == 1 => {
                let stream = v[0].1.gen_widget_init();
                quote!(*widget #s = vec![#stream])
            }
            [] => TokenStream::new(),
            _ => {
                let count = self.count.expect("widget set should be created.");
                let widget_set = format_ident!("WidgetSet{count}");

                self.widgets
                    .iter()
                    .map(|(s, child)| match child {
                        Children::One((i, child)) => {
                            let stream = child.gen_widget_init();
                            let ident = format_ident!("W{i}");
                            quote!(*widget #s = Some(#widget_set :: #ident (#stream)))
                        }
                        Children::Many(children) => {
                            let inits = children.iter().map(|(i, child)| {
                                let stream = child.gen_widget_init();
                                let ident = format_ident!("W{i}");
                                quote!(#widget_set :: #ident (#stream))
                            });
                            quote!(*widget #s = vec![#(#inits),*])
                        }
                    })
                    .collect()
            }
        }
    }

    pub fn gen_widget_set(&self, stream: &mut TokenStream) {
        let all_widgets = self
            .widgets
            .iter()
            .flat_map(|(_, w)| w.iter())
            .collect_vec();

        if let Some(count) = self.count {
            let widget_set = format_ident!("WidgetSet{count}");

            let variants = all_widgets
                .iter()
                .map(|(i, _)| format_ident!("W{i}"))
                .collect_vec();

            let func_names = all_widgets.iter().map(|(i, _)| format_ident!("w{i}"));

            let types = all_widgets
                .iter()
                .map(|(_, w)| w.gen_widget_type())
                .collect_vec();

            let ids = all_widgets.iter().map(|(_, w)| w.id);

            stream.extend(quote! {
                enum #widget_set {
                    #( #variants(#types) ),*
                }

                impl #widget_set {
                    #(
                        pub fn #func_names(&mut self) -> &mut #types {
                            if let #widget_set::#variants(val) = self {
                                val
                            } else {
                                panic!("Incorrect wrapped type.")
                            }
                        }
                    )*
                }

                impl Widget<CompStruct> for #widget_set {
                    fn id(&self) -> WidgetID {
                        match self {
                            #( #widget_set::#variants(_) => #ids ),*
                        }
                    }

                     fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<CompStruct>) {
                        match self {
                            #( #widget_set::#variants(w) => <#types as Widget<CompStruct>>::render(w, scene, handle) ),*
                        }
                    }

                    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<CompStruct>) -> Size {
                        match self {
                            #( #widget_set::#variants(w) => <#types as Widget<CompStruct>>::resize(w, constraints, handle) ),*
                        }
                    }

                    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<CompStruct>) {
                        match self {
                            #( #widget_set::#variants(w) => <#types as Widget<CompStruct>>::event(w, event, handle) ),*
                        }
                    }

                }
            });
        }

        for (_, w) in all_widgets {
            w.gen_widget_set(stream)
        }
    }

    //TODO: Fix this to cache results instead of re-computing this every time.
    pub fn gen_widget_gets<'b>(
        &'b self,
        stream: &'b TokenStream,
    ) -> impl Iterator<Item = (TokenStream, &Widget)> + '_ {
        self.widgets
            .iter()
            .flat_map(|(get_widget, widgets)| match widgets {
                Children::One((i, w)) => {
                    let mut s = stream.clone();
                    s.extend(get_widget.clone());
                    // TODO: Find a better method than using an &mut Option<ChildWidget> for children
                    s.extend(quote!(.as_mut().unwrap()));
                    if self.count.is_some() {
                        let func = format_ident!("w{i}");
                        s.extend(quote!( .#func() ));
                    }
                    vec![(s, w)]
                }
                Children::Many(m) => m
                    .iter()
                    .enumerate()
                    .map(|(count, (i, w))| {
                        let mut s = stream.clone();
                        s.extend(get_widget.clone());
                        if self.count.is_some() {
                            let func = format_ident!("w{i}");
                            s.extend(quote!( [#count].#func() ));
                        }
                        (s, w)
                    })
                    .collect_vec(),
            })
    }
}
