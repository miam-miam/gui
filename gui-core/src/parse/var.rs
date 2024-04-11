use proc_macro2::Ident;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use syn::__private::Span;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Var<T> {
    Variable(Name),
    #[serde(untagged)]
    Value(T),
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ComponentVar {
    Variable(Name),
}

impl ComponentVar {
    pub fn get_component_name(&self) -> &Name {
        let Self::Variable(name) = self;
        name
    }

    pub fn unwrap(self) -> Name {
        let Self::Variable(name) = self;
        name
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Name(String);

impl Name {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn validate_str(value: &str) -> Result<(), syn::Error> {
        if !value.starts_with(|c: char| c.is_ascii_alphabetic()) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Name: {value} must start with an ASCII alphabetic character"),
            ));
        }
        if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Name: {value} must be made up of only ASCII alphabetic characters and underscores"),
            ));
        }
        Ok(())
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Name {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        syn::parse_str::<Ident>(s)?;
        Name::validate_str(s)?;
        Ok(Name(String::from(s)))
    }
}

impl TryFrom<String> for Name {
    type Error = syn::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        syn::parse_str::<Ident>(&value)?;
        Name::validate_str(&value)?;
        Ok(Name(value))
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

struct NameVisitor;

impl<'de> Visitor<'de> for NameVisitor {
    type Value = Name;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a valid rust Ident")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.try_into().map_err(|_| {
            Error::invalid_value(
                Unexpected::Other("name does not follow rust/fluent ident rules."),
                &self,
            )
        })
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(NameVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::{ComponentVar, Name};
    use crate::Var;
    use serde::Deserialize;
    use serde_yaml::Value;

    #[test]
    fn name_deserialization() {
        assert_eq!(
            serde_yaml::from_value::<Name>(Value::String("variable_9".into())).unwrap(),
            Name("variable_9".into())
        );
        assert_eq!(
            serde_yaml::from_str::<Name>("VAriable").unwrap(),
            Name("VAriable".into())
        );
        assert!(serde_yaml::from_str::<Name>("not-allowed-hyphens").is_err());
        assert!(serde_yaml::from_str::<Name>("0StartNumbers").is_err());
        assert!(serde_yaml::from_str::<Name>("await").is_err());
        assert!(serde_yaml::from_str::<Name>("fn").is_err());
    }

    #[test]
    fn variable_deserialization() {
        assert_eq!(
            serde_yaml::from_str::<Var<u8>>("12").unwrap(),
            Var::Value(12u8)
        );
        assert_eq!(
            serde_yaml::from_str::<Var<u8>>(
                r#"
        variable:
          variable_name"#
            )
            .unwrap(),
            Var::Variable("variable_name".parse().unwrap())
        );
    }

    #[test]
    fn component_var_deserialization() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct Widget {
            #[serde(flatten)]
            component: Option<ComponentVar>,
        }

        assert_eq!(
            serde_yaml::from_str::<Widget>(
                r#"
              variable: component_name"#
            )
            .unwrap(),
            Widget {
                component: Some(ComponentVar::Variable("component_name".parse().unwrap()))
            }
        );
    }
}
