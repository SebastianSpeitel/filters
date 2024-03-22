use super::{FalsyDefault, Filter, Optimizable, TruthyDefault};

#[derive(Debug)]
pub struct Or<F>(Vec<F>);

impl<F> From<Vec<F>> for Or<F> {
    #[inline]
    fn from(value: Vec<F>) -> Self {
        Self(value)
    }
}

impl<F> std::ops::Deref for Or<F> {
    type Target = Vec<F>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> std::ops::DerefMut for Or<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F: Filter<T> + TruthyDefault, T: ?Sized> Filter<T> for Or<F> {
    #[inline]
    fn matches(&self, obj: &T) -> bool {
        self.0.iter().any(|f| f.matches(obj))
    }
}

impl<F: Optimizable + TruthyDefault> Optimizable for Or<F> {
    #[inline]
    fn as_bool(&self) -> Option<bool> {
        for f in &self.0 {
            if f.as_bool()? {
                return Some(true);
            }
        }
        Some(false)
    }

    #[inline]
    fn optimize(&mut self) {
        let mut short_circuit = false;
        self.0.retain_mut(|f| {
            if short_circuit {
                // already short circuited
                return false;
            }
            f.optimize();
            let bool = f.as_bool();

            // if any filter is unconditionally true, short circuit
            if bool == Some(true) {
                short_circuit = true;
            }

            // if filter is unconditional, remove it
            bool.is_none()
        });

        // if any filter is unconditionally true, the whole filter is true
        if short_circuit {
            std::mem::swap(self, &mut Self::truthy_default());
        }
    }
}

impl<F: TruthyDefault> TruthyDefault for Or<F> {
    #[inline]
    fn truthy_default() -> Self {
        Self(vec![F::truthy_default()])
    }
}

impl<F> FalsyDefault for Or<F> {
    #[inline]
    fn falsy_default() -> Self {
        Self(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::super::TestFilter;
    use super::*;

    #[test]
    fn mixed() {
        let mut f = TestFilter::truthy_default() | TestFilter::falsy_default();

        assert_eq!(f.as_bool().unwrap(), true);

        f.optimize();
        assert_eq!(f.len(), 1);

        assert_eq!(f.as_bool().unwrap(), true);
    }

    #[test]
    fn truthy() {
        let mut f = TestFilter::truthy_default() | TestFilter::falsy_default();

        assert_eq!(f.as_bool().unwrap(), true);

        f.optimize();
        assert_eq!(f.len(), 1);

        assert_eq!(f.as_bool().unwrap(), true);
    }

    #[test]
    fn falsy() {
        let mut f = TestFilter::falsy_default() | TestFilter::falsy_default();

        assert_eq!(f.as_bool().unwrap(), false);

        f.optimize();
        assert_eq!(f.len(), 0);

        assert_eq!(f.as_bool().unwrap(), false);
    }
}
