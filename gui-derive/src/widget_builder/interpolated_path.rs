use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::PathSep;
use syn::{parse_quote, Token, Type, TypeInfer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpolatedPath {
    pub leading_colon: Token![::],
    pub segments: Punctuated<Ident, PathSep>,
    pub generics: Option<InterpolatedGenerics>,
}

impl Parse for InterpolatedPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let leading_colon = input.parse()?;
        let segments = Punctuated::parse_separated_nonempty(input)?;
        let generics = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(InterpolatedPath {
            leading_colon,
            segments,
            generics,
        })
    }
}

impl ToTokens for InterpolatedPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let segments = &self.segments;
        let generics = &self.generics;
        if segments.first().is_some_and(|i| *i == "crate") {
            tokens.extend(quote!(#segments #generics ))
        } else {
            tokens.extend(quote!(:: #segments #generics ))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpolatedGenerics {
    pub lt_token: Token![<],
    pub args: Punctuated<InterpolatedType, Token![,]>,
    pub gt_token: Token![>],
}

impl Parse for InterpolatedGenerics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(InterpolatedGenerics {
            lt_token: input.parse()?,
            args: Punctuated::parse_separated_nonempty(input)?,
            gt_token: input.parse()?,
        })
    }
}

impl ToTokens for InterpolatedGenerics {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let args = &self.args;
        tokens.extend(quote!(< #args >))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InterpolatedType {
    Interpolated { pound: Token![#], name: Ident },
    Type(Type),
}

impl InterpolatedType {
    pub fn erase_interpolated(&mut self) {
        if let InterpolatedType::Interpolated { .. } = self {
            *self = InterpolatedType::Type(Type::Infer(TypeInfer {
                underscore_token: Default::default(),
            }))
        }
    }

    pub fn convert_to_fakes(&mut self, fake_path: &TokenStream) {
        if let InterpolatedType::Interpolated { name, .. } = self {
            match name.to_string().as_str() {
                "handler" => *self = InterpolatedType::Type(parse_quote!(#fake_path Handler)),
                "component" => *self = InterpolatedType::Type(parse_quote!(#fake_path ToComp)),
                "child" => {
                    *self = InterpolatedType::Type(parse_quote!(::gui_custom::widget::WidgetID))
                }
                _ => {}
            }
        }
    }
}

impl Parse for InterpolatedType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![#]) {
            InterpolatedType::Interpolated {
                pound: input.parse()?,
                name: input.parse()?,
            }
        } else {
            InterpolatedType::Type(input.parse()?)
        })
    }
}

impl ToTokens for InterpolatedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            InterpolatedType::Interpolated { pound, name } => tokens.extend(quote!(#pound #name)),
            InterpolatedType::Type(t) => tokens.extend(quote!(#t)),
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use quote::ToTokens;

    use crate::widget_builder::interpolated_path::InterpolatedPath;

    fn assert_type_params(type_path: &str, expected: &[&str]) {
        let path: InterpolatedPath = syn::parse_str(type_path).unwrap();
        let found = path.generics.map_or_else(Vec::new, |g| {
            g.args
                .into_iter()
                .map(|t| t.into_token_stream().to_string())
                .collect_vec()
        });
        assert_eq!(expected, found);
    }

    #[test]
    fn parse_type_path() {
        assert_type_params(
            "::crate_name::widget_struct<type_param1, type_param2>",
            &["type_param1", "type_param2"],
        );
        assert_type_params(
            "::crate_name::widget_struct<#type_param1, #type_param2>",
            &["# type_param1", "# type_param2"],
        );
        assert_type_params(
            "::crate_name::module::widget_struct<[i32; 2], #type_param2, (u32, u8)>",
            &["[i32 ; 2]", "# type_param2", "(u32 , u8)"],
        );
        assert_type_params("::crate_name::module::widget_struct", &[]);
    }

    #[test]
    fn must_have_leading_colon() {
        let res: Result<InterpolatedPath, _> = syn::parse_str("widget_struct<t1, t2>");
        assert!(res.is_err(), "Must have leading colons.")
    }
}
