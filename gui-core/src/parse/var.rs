use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Var<T> {
    Variable(String),
    #[serde(untagged)]
    Value(T),
}
