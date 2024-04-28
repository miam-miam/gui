/// Main struct used to create procedural-type variables use this in your user-defined components
/// to forego the need to implement the `Variable` trait.
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

impl<T: PartialEq> Updateable<T> {
    /// Set the variable's value to `val`, invalidating the stored value if the value has changed.
    pub fn set_value(&mut self, val: T) {
        if self.value != val {
            self.value = val;
            self.updated = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Updateable;

    #[test]
    fn new_updateable() {
        let u = Updateable::new(5);
        assert_eq!(u.get_value(), &5);
        assert!(!u.is_updated());
    }

    #[test]
    fn set_updateable() {
        let mut u = Updateable::new(5);
        u.set_value(10);
        assert_eq!(u.get_value(), &10);
        assert!(u.is_updated());
    }

    #[test]
    fn invalidate_updateable() {
        let mut u = Updateable::new(5);
        *u.invalidate() = 10;
        assert_eq!(u.get_value(), &10);
        assert!(u.is_updated());
    }

    #[test]
    fn reset_updateable() {
        let mut u = Updateable::new(5);
        u.set_value(10);
        u.reset();
        assert!(!u.is_updated());
    }

    #[test]
    fn clone_updateable_value() {
        let u = Updateable::new(vec![1, 2, 3]);
        assert_eq!(u.value(), vec![1, 2, 3]);
    }
}
