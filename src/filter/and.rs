use super::{FalsyDefault, Filter, Optimizable, TruthyDefault};

#[derive(Debug)]
pub struct And<F>(Vec<F>);

impl<F> From<Vec<F>> for And<F> {
    #[inline]
    fn from(value: Vec<F>) -> Self {
        Self(value)
    }
}

impl<F> std::ops::Deref for And<F> {
    type Target = Vec<F>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> std::ops::DerefMut for And<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F: Filter<T>, T: ?Sized> Filter<T> for And<F> {
    #[inline]
    fn matches(&self, obj: &T) -> bool {
        self.0.iter().all(|f| f.matches(obj))
    }
}

impl<F: Optimizable + FalsyDefault> Optimizable for And<F> {
    #[inline]
    fn as_bool(&self) -> Option<bool> {
        for f in &self.0 {
            if !f.as_bool()? {
                return Some(false);
            }
        }
        Some(true)
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

            // if any filter is unconditionally false, short circuit
            if bool == Some(false) {
                short_circuit = true;
            }

            // if filter is unconditional, remove it
            bool.is_none()
        });

        // if any filter is unconditionally false, the whole filter is false
        if short_circuit {
            std::mem::swap(self, &mut Self::falsy_default());
        }
    }
}

impl<F> TruthyDefault for And<F> {
    #[inline]
    fn truthy_default() -> Self {
        Self(Vec::new())
    }
}

impl<F: FalsyDefault> FalsyDefault for And<F> {
    #[inline]
    fn falsy_default() -> Self {
        Self(vec![F::falsy_default()])
    }
}

#[cfg(test)]
mod tests {
    use super::super::TestFilter;
    use super::*;

    #[test]
    fn mixed() {
        let mut f = TestFilter::truthy_default() & TestFilter::falsy_default();

        assert_eq!(f.as_bool().unwrap(), false);

        f.optimize();
        assert_eq!(f.len(), 1);

        assert_eq!(f.as_bool().unwrap(), false);
    }

    #[test]
    fn truthy() {
        let mut f = TestFilter::truthy_default() & TestFilter::truthy_default();

        assert_eq!(f.as_bool().unwrap(), true);

        f.optimize();
        assert_eq!(f.len(), 0);

        assert_eq!(f.as_bool().unwrap(), true);
    }

    #[test]
    fn falsy() {
        let mut f = TestFilter::falsy_default() & TestFilter::falsy_default();

        assert_eq!(f.as_bool().unwrap(), false);

        f.optimize();
        assert_eq!(f.len(), 1);

        assert_eq!(f.as_bool().unwrap(), false);
    }
}
