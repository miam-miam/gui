use proc_macro2::TokenStream;
use quote::ToTokens;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

#[derive(Clone, Debug, Default)]
pub struct EqTokenStream(TokenStream, OnceLock<String>);

impl Hash for EqTokenStream {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.get_or_init(|| self.0.to_string()).hash(state)
    }
}

impl PartialEq for EqTokenStream {
    fn eq(&self, other: &Self) -> bool {
        self.1
            .get_or_init(|| self.0.to_string())
            .eq(other.1.get_or_init(|| other.0.to_string()))
    }
}
impl Eq for EqTokenStream {}

impl ToTokens for EqTokenStream {
    fn to_tokens(&self, dst: &mut TokenStream) {
        self.0.to_tokens(dst)
    }

    fn into_token_stream(self) -> TokenStream {
        self.0
    }
}

impl EqTokenStream {
    pub fn get(&self) -> &TokenStream {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut TokenStream {
        self.1.take();
        &mut self.0
    }
}

impl From<TokenStream> for EqTokenStream {
    fn from(value: TokenStream) -> Self {
        EqTokenStream(value, OnceLock::new())
    }
}
