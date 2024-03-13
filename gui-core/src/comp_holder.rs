use crate::ToComponent;

#[derive(Clone, Debug)]
pub struct CompHolder<T: ToComponent> {
    component: Option<T>,
}

impl<T: ToComponent + Default> Default for CompHolder<T> {
    fn default() -> Self {
        Self {
            component: Some(T::default()),
        }
    }
}

impl<T: ToComponent> CompHolder<T> {
    pub fn new(component: T) -> Self {
        Self {
            component: Some(component),
        }
    }

    pub fn replace_component(&mut self, component: T) {
        self.component = Some(component);
    }
}

#[doc(hidden)]
impl<T: ToComponent> CompHolder<T> {
    pub fn take(&mut self) -> Option<T> {
        self.component.take()
    }

    pub fn is_updated(&self) -> bool {
        self.component.is_some()
    }
}
