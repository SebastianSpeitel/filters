use super::{FalsyDefault, Filter, Optimizable, TruthyDefault};

#[derive(Debug)]
pub struct Not<F>(pub F);

impl<F: Filter<T>, T: ?Sized> Filter<T> for Not<F> {
    #[inline]
    fn matches(&self, obj: &T) -> bool {
        !self.0.matches(obj)
    }
}
impl<F: Optimizable> Optimizable for Not<F> {
    #[inline]
    fn as_bool(&self) -> Option<bool> {
        self.0.as_bool().map(|b| !b)
    }

    #[inline]
    fn optimize(&mut self) {
        self.0.optimize();
    }
}

impl<F: FalsyDefault> TruthyDefault for Not<F> {
    #[inline]
    fn truthy_default() -> Self {
        Self(F::falsy_default())
    }
}

impl<F: TruthyDefault> FalsyDefault for Not<F> {
    #[inline]
    fn falsy_default() -> Self {
        Self(F::truthy_default())
    }
}
