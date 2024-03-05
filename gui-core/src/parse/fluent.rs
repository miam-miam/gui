use crate::parse::var::Name;
use anyhow::anyhow;
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Message, Pattern, PatternElement};
use fluent_syntax::parser;
use itertools::Itertools;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Fluent {
    pub vars: Vec<Name>,
    pub text: String,
}

struct FluentVisitor;

fn get_vars(pattern: &Pattern<String>) -> anyhow::Result<Vec<Name>> {
    pattern
        .elements
        .iter()
        .filter_map(|elem| {
            if let PatternElement::Placeable { expression } = elem {
                let mut current_expr = expression;
                loop {
                    let (Expression::Select { selector, .. } | Expression::Inline(selector)) =
                        current_expr;
                    match selector {
                        InlineExpression::VariableReference { id } => {
                            return Some(id.name.parse().map_err(|e| anyhow!("{e}")))
                        }
                        InlineExpression::Placeable { expression } => {
                            current_expr = expression.as_ref();
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
            None
        })
        .process_results(|iter| iter.unique().collect())
}

impl<'de> Visitor<'de> for FluentVisitor {
    type Value = Fluent;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string using the project fluent syntax")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let test_parse = format!("test = {}", v);
        let resource = parser::parse_runtime(test_parse)
            .map_err(|e| Error::invalid_value(Unexpected::Other(&format!("{:#?}", e.1)), &self))?;
        if resource.body.len() != 1 {
            return Err(Error::invalid_value(
                Unexpected::Other("multiple fluent entries"),
                &self,
            ));
        }
        match &resource.body[0] {
            Entry::Message(Message {
                value: Some(pattern),
                ..
            }) => Ok(Fluent {
                vars: get_vars(pattern).map_err(|e| {
                    Error::invalid_value(Unexpected::Other(&format!("{:#?}", e)), &self)
                })?,
                text: v.to_string(),
            }),
            _ => Err(Error::invalid_value(
                Unexpected::Other("a non-message entry"),
                &self,
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Fluent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FluentVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::parse::fluent::Fluent;
    use std::collections::HashMap;

    #[test]
    fn check_fluent_yaml_parsing() {
        let yaml = r#"test: Hello, {$userName}!
just_var: "{$user} {$user}"
complex_test: |
    {$userName} {$photoCount ->
        [one] added a new photo
       *[other] added {$photoCount} new photos
    } to {$userGender ->
        [male] his stream
        [female] her stream
       *[other] their stream
    }.
        "#;

        let ser: HashMap<String, Fluent> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            ser.get("test"),
            Some(&Fluent {
                vars: vec!["userName".parse().unwrap()],
                text: String::from("Hello, {$userName}!")
            })
        );

        assert_eq!(
            ser.get("just_var"),
            Some(&Fluent {
                vars: vec!["user".parse().unwrap()],
                text: String::from("{$user} {$user}")
            })
        );

        assert_eq!(
            ser.get("complex_test").unwrap().vars,
            vec![
                "userName".parse().unwrap(),
                "photoCount".parse().unwrap(),
                "userGender".parse().unwrap()
            ]
        )
    }
}
