use super::{FalsyDefault, Filter, Optimizable, TruthyDefault};

#[derive(Debug, PartialEq)]
pub enum TestFilter {
    Any,
    None,
    Panic,
}

impl<T> Filter<T> for TestFilter {
    fn matches(&self, _: &T) -> bool {
        match self {
            TestFilter::Any => true,
            TestFilter::None => false,
            TestFilter::Panic => panic!(),
        }
    }
}

impl Optimizable for TestFilter {
    fn as_bool(&self) -> Option<bool> {
        match self {
            TestFilter::Any => Some(true),
            TestFilter::None => Some(false),
            TestFilter::Panic => None,
        }
    }

    fn optimize(&mut self) {
        *self = match self {
            TestFilter::Any => TestFilter::Any,
            TestFilter::None => TestFilter::None,
            TestFilter::Panic => unreachable!(),
        };
    }
}

impl TruthyDefault for TestFilter {
    fn truthy_default() -> Self {
        TestFilter::Any
    }
}

impl FalsyDefault for TestFilter {
    fn falsy_default() -> Self {
        TestFilter::None
    }
}

impl std::ops::BitAnd for TestFilter {
    type Output = super::And<Self>;
    fn bitand(self, rhs: Self) -> Self::Output {
        vec![self, rhs].into()
    }
}

impl std::ops::BitOr for TestFilter {
    type Output = super::Or<Self>;
    fn bitor(self, rhs: Self) -> Self::Output {
        vec![self, rhs].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truthy() {
        let mut filter = TestFilter::truthy_default();

        assert_eq!(filter, TestFilter::Any);

        assert_eq!(filter.as_bool(), Some(true));

        filter.optimize();

        assert_eq!(filter, TestFilter::Any);
    }

    #[test]
    fn falsy() {
        let mut filter = TestFilter::falsy_default();

        assert_eq!(filter, TestFilter::None);

        assert_eq!(filter.as_bool(), Some(false));

        filter.optimize();

        assert_eq!(filter, TestFilter::None);
    }

    #[test]
    #[should_panic]
    fn panics() {
        let filter = TestFilter::Panic;

        filter.matches_owned(());
    }
}
