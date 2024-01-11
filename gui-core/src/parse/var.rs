use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Var<T> {
    Variable(String),
    #[serde(untagged)]
    Value(T),
}
