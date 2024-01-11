pub mod common;
pub mod parse;
pub mod widget;

pub use parse::colour::Colour;
pub use parse::var::Var;

pub use glazier;
pub use parley;
pub use vello;

pub use parley::font::FontContext;
pub use vello::SceneBuilder;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}