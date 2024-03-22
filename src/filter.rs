mod or;
pub use or::Or;
mod and;
pub use and::And;
mod not;
pub use not::Not;
mod text;
pub use text::TextFilter;
#[cfg(test)]
mod testfilter;
#[cfg(test)]
pub use testfilter::TestFilter;

pub trait Filter<T: ?Sized> {
    fn matches(&self, obj: &T) -> bool;

    #[inline]
    fn matches_owned(&self, obj: T) -> bool
    where
        T: Sized,
    {
        self.matches(&obj)
    }
}

pub trait Optimizable {
    #[inline]
    fn optimize(&mut self) {}

    #[inline]
    fn as_bool(&self) -> Option<bool> {
        None
    }
}

pub trait TruthyDefault {
    fn truthy_default() -> Self;
}

pub trait FalsyDefault {
    fn falsy_default() -> Self;
}

impl<T> Filter<T> for bool {
    #[inline]
    fn matches(&self, _: &T) -> bool {
        *self
    }
}

impl Optimizable for bool {
    #[inline]
    fn as_bool(&self) -> Option<bool> {
        Some(*self)
    }
}

impl TruthyDefault for bool {
    #[inline]
    fn truthy_default() -> Self {
        true
    }
}

impl FalsyDefault for bool {
    #[inline]
    fn falsy_default() -> Self {
        false
    }
}

impl<F: Filter<T>, T: ?Sized> Filter<T> for Box<F> {
    #[inline]
    fn matches(&self, obj: &T) -> bool {
        self.as_ref().matches(obj)
    }
}

impl<F: Optimizable> Optimizable for Box<F> {
    #[inline]
    fn optimize(&mut self) {
        self.as_mut().optimize();
    }
}

impl<F: FalsyDefault> FalsyDefault for Box<F> {
    #[inline]
    fn falsy_default() -> Self {
        Box::new(F::falsy_default())
    }
}

impl<F: TruthyDefault> TruthyDefault for Box<F> {
    #[inline]
    fn truthy_default() -> Self {
        Box::new(F::truthy_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_safety() {
        fn _f<On>(_d: &dyn Filter<On>) {}
    }
}
