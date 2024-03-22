use super::{And, DataFilter, FalsyDefault, Filter, Not, Optimizable, Or, TruthyDefault};
use datalink::links::Link;

#[derive(Default, Debug)]
#[non_exhaustive]
pub enum LinkFilter {
    #[default]
    Any,
    Key(DataFilter),
    Target(DataFilter),
    Or(Or<LinkFilter>),
    And(And<LinkFilter>),
    Not(Box<Not<LinkFilter>>),
    None,
}

impl LinkFilter {
    #[inline]
    #[must_use]
    pub const fn any() -> Self {
        Self::Any
    }
    #[inline]
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }
    #[inline]
    #[must_use]
    pub fn key(f: impl Into<DataFilter>) -> Self {
        Self::Key(f.into())
    }
    #[inline]
    #[must_use]
    pub fn target(f: impl Into<DataFilter>) -> Self {
        Self::Target(f.into())
    }
    #[inline]
    #[must_use]
    pub fn and(mut self, f: impl Into<Self>) -> Self {
        match &mut self {
            Self::And(and) => {
                and.push(f.into());
                self
            }
            _ => Self::And(vec![self, f.into()].into()),
        }
    }
    #[inline]
    #[must_use]
    pub fn or(mut self, f: impl Into<Self>) -> Self {
        match &mut self {
            Self::Or(or) => {
                or.push(f.into());
                self
            }
            _ => Self::Or(vec![self, f.into()].into()),
        }
    }
}
impl<L: Link + ?Sized> Filter<L> for LinkFilter {
    #[inline]
    fn matches(&self, l: &L) -> bool {
        use LinkFilter as E;
        match self {
            E::Any => true,
            E::None => false,
            E::Not(f) => !f.matches(l),
            E::And(f) => f.matches(l),
            E::Or(f) => f.matches(l),
            E::Key(f) => l.key().is_some_and(|k| f.matches(k)),
            E::Target(f) => f.matches(l.target()),
        }
    }
}

impl Optimizable for LinkFilter {
    #[inline]
    fn as_bool(&self) -> Option<bool> {
        use LinkFilter as E;
        match self {
            E::Any => Some(true),
            E::None => Some(false),
            E::And(f) => f.as_bool(),
            E::Or(f) => f.as_bool(),
            E::Not(f) => f.as_bool(),
            E::Key(f) | E::Target(f) => f.as_bool(),
        }
    }

    #[inline]
    fn optimize(&mut self) {
        use LinkFilter as E;
        match self {
            E::And(f) => f.optimize(),
            E::Or(f) => f.optimize(),
            E::Not(f) => f.optimize(),
            E::Key(f) | E::Target(f) => f.optimize(),
            E::Any => {
                *self = Self::truthy_default();
                return;
            }
            E::None => {
                *self = Self::falsy_default();
                return;
            }
        }
        match self.as_bool() {
            Some(true) => *self = Self::truthy_default(),
            Some(false) => *self = Self::falsy_default(),
            None => {}
        }
    }
}

impl FalsyDefault for LinkFilter {
    #[inline]
    fn falsy_default() -> Self {
        Self::None
    }
}

impl TruthyDefault for LinkFilter {
    #[inline]
    fn truthy_default() -> Self {
        Self::Any
    }
}

impl<F: Into<Self>> std::ops::BitAnd<F> for LinkFilter {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: F) -> Self {
        self.and(rhs)
    }
}
impl<F: Into<Self>> std::ops::BitOr<F> for LinkFilter {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: F) -> Self {
        self.or(rhs)
    }
}
impl std::ops::Not for LinkFilter {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self::Not(Box::new(Not(self)))
    }
}
