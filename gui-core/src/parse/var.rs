use proc_macro2::Ident;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Var<T> {
    Variable(Name),
    #[serde(untagged)]
    Value(T),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Name(String);

impl Name {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Name {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        syn::parse_str::<Ident>(s)?;
        Ok(Name(String::from(s)))
    }
}

impl TryFrom<String> for Name {
    type Error = syn::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        syn::parse_str::<Ident>(&value)?;
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
        syn::parse_str::<Ident>(&v).map_err(|_| {
            Error::invalid_value(
                Unexpected::Other("name does not follow rust ident rules."),
                &self,
            )
        })?;
        Ok(Name(v))
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
