use std::slice::IterMut;

use itertools::Either;

use crate::parse::WidgetDeclaration;

pub type MutWidgetChildren<'a> = Children<&'a mut WidgetDeclaration>;
pub type WidgetChildren<'a> = Children<&'a WidgetDeclaration>;

#[derive(Clone, Debug)]
pub enum Children<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> Children<T> {
    pub fn single(&self) -> Option<&T> {
        if let Children::One(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn multi(&self) -> Option<&Vec<T>> {
        if let Children::Many(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Children::One(_) => 1,
            Children::Many(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.multi().is_some_and(|v| v.is_empty())
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> Children<U> {
        match self {
            Children::One(s) => Children::One(f(s)),
            Children::Many(m) => Children::Many(m.into_iter().map(f).collect()),
        }
    }

    pub fn try_map<U, E, F: FnMut(T) -> Result<U, E>>(self, mut f: F) -> Result<Children<U>, E> {
        Ok(match self {
            Children::One(s) => Children::One(f(s)?),
            Children::Many(m) => {
                Children::Many(m.into_iter().map(f).collect::<Result<Vec<_>, E>>()?)
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.single()
            .into_iter()
            .chain(self.multi().into_iter().flat_map(|m| m.iter()))
    }

    pub fn iter_mut(&mut self) -> SingleOrMultiIterMut<T> {
        let either = match self {
            Children::One(s) => Either::Left(Some(s)),
            Children::Many(m) => Either::Right(m.iter_mut()),
        };
        SingleOrMultiIterMut(either)
    }
}

pub struct SingleOrMultiIterMut<'a, T>(Either<Option<&'a mut T>, IterMut<'a, T>>);

impl<'a, T> Iterator for SingleOrMultiIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Either::Left(l) => l.take(),
            Either::Right(r) => r.next(),
        }
    }
}
