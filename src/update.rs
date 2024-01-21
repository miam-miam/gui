#[derive(Debug, Default)]
pub struct Updateable<T> {
    updated: bool,
    value: T,
}

impl<T> Updateable<T> {
    pub fn new(value: T) -> Self {
        Updateable {
            updated: false,
            value,
        }
    }
    pub fn is_updated(&self) -> bool {
        self.updated
    }

    pub fn set_value(&mut self, val: T) {
        self.value = val;
        self.updated = true;
    }

    pub fn invalidate(&mut self) -> &mut T {
        self.updated = true;
        &mut self.value
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn reset(&mut self) {
        self.updated = false;
    }
}

impl<T: Clone> Updateable<T> {
    pub fn value(&self) -> T {
        self.value.clone()
    }
}
