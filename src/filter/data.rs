use std::borrow::Borrow;

use super::{
    And, FalsyDefault, Filter, LinkFilter, Not, Optimizable, Or, TextFilter, TruthyDefault,
};
use datalink::{id::ID, BoxedData, Data};

#[derive(Default, Debug)]
#[non_exhaustive]
pub enum DataFilter {
    #[default]
    Any,
    Or(Or<DataFilter>),
    And(And<DataFilter>),
    Not(Box<Not<DataFilter>>),
    Text(TextFilter),
    Unique,
    Id(ID),
    NotId(ID),
    Linked(Box<LinkFilter>),
    None,
}

impl DataFilter {
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
    pub fn text(f: impl Into<TextFilter>) -> Self {
        Self::Text(f.into())
    }
    #[inline]
    #[must_use]
    pub const fn unique() -> Self {
        Self::Unique
    }
    #[inline]
    #[must_use]
    pub fn id(id: impl Into<ID>) -> Self {
        Self::Id(id.into())
    }
    #[inline]
    #[must_use]
    pub fn not_id(id: impl Into<ID>) -> Self {
        Self::NotId(id.into())
    }
    #[inline]
    #[must_use]
    pub fn linked(filter: impl Into<LinkFilter>) -> Self {
        Self::Linked(Box::new(filter.into()))
    }
    #[cfg(feature = "unique")]
    #[inline]
    #[must_use]
    pub fn eq(data: &impl datalink::data::unique::Unique) -> Self {
        Self::Id(data.id())
    }
    #[cfg(feature = "unique")]
    #[inline]
    #[must_use]
    pub fn ne(data: &impl datalink::data::unique::Unique) -> Self {
        Self::NotId(data.id())
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
impl<D: Data + ?Sized> Filter<D> for DataFilter {
    #[inline]
    fn matches(&self, d: &D) -> bool {
        use DataFilter as E;
        match self {
            E::Any => true,
            E::None => false,
            E::And(and) => and.iter().all(|f| Filter::<D>::matches(f, d)),
            E::Or(or) => or.iter().any(|f| Filter::<D>::matches(f, d)),
            E::Id(id) => d.get_id().is_some_and(|ref i| i == id),
            E::NotId(id) => !d.get_id().is_some_and(|ref i| i == id),
            E::Not(f) => f.matches(d),
            E::Unique => d.get_id().is_some(),
            E::Linked(f) => {
                struct Searcher<'a>(bool, &'a LinkFilter);
                impl datalink::links::Links for Searcher<'_> {
                    #[inline]
                    fn push(
                        &mut self,
                        target: BoxedData,
                        key: Option<BoxedData>,
                    ) -> datalink::links::Result {
                        if let Some(key) = key {
                            self.push_keyed(target, key)
                        } else {
                            self.push_unkeyed(target)
                        }
                    }
                    #[inline]
                    fn push_keyed(
                        &mut self,
                        target: BoxedData,
                        key: BoxedData,
                    ) -> datalink::links::Result {
                        if self.1.matches_owned((key, target)) {
                            self.0 = true;
                            datalink::links::BREAK
                        } else {
                            datalink::links::CONTINUE
                        }
                    }
                    #[inline]
                    fn push_unkeyed(&mut self, target: BoxedData) -> datalink::links::Result {
                        if Filter::<BoxedData>::matches_owned(self.1, target) {
                            self.0 = true;
                            datalink::links::BREAK
                        } else {
                            datalink::links::CONTINUE
                        }
                    }
                }
                let mut searcher = Searcher(false, f);
                let _ = d.borrow().provide_links(&mut searcher);
                searcher.0
            }
            E::Text(f) => {
                enum Matcher<'a> {
                    Found,
                    Selecting(&'a TextFilter),
                }
                impl datalink::value::ValueBuiler<'_> for Matcher<'_> {
                    fn str(&mut self, value: std::borrow::Cow<'_, str>) {
                        match self {
                            Matcher::Selecting(f) if f.matches(value.as_ref()) => {
                                *self = Matcher::Found
                            }
                            _ => {}
                        }
                    }
                }
                let mut m = Matcher::Selecting(f);
                d.borrow().provide_value(&mut m);
                matches!(m, Matcher::Found)
            }
        }
    }
}

impl Optimizable for DataFilter {
    #[inline]
    fn as_bool(&self) -> Option<bool> {
        use DataFilter as E;
        match self {
            E::Any => Some(true),
            E::None => Some(false),
            E::And(f) => f.as_bool(),
            E::Or(f) => f.as_bool(),
            E::Not(f) => f.as_bool(),
            E::Text(f) => f.as_bool(),
            _ => None,
        }
    }

    #[inline]
    fn optimize(&mut self) {
        use DataFilter as E;
        match self {
            E::And(f) => f.optimize(),
            E::Or(f) => f.optimize(),
            E::Not(f) => f.optimize(),
            E::Text(f) => f.optimize(),
            _ => {}
        }
        match self.as_bool() {
            Some(true) => *self = Self::truthy_default(),
            Some(false) => *self = Self::falsy_default(),
            None => {}
        }
    }
}

impl TruthyDefault for DataFilter {
    #[inline]
    fn truthy_default() -> Self {
        Self::Any
    }
}

impl FalsyDefault for DataFilter {
    #[inline]
    fn falsy_default() -> Self {
        Self::None
    }
}

impl<F: Into<Self>> std::ops::BitAnd<F> for DataFilter {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: F) -> Self {
        self.and(rhs)
    }
}
impl<F: Into<Self>> std::ops::BitOr<F> for DataFilter {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: F) -> Self {
        self.or(rhs)
    }
}
impl std::ops::Not for DataFilter {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self::Not(Box::new(Not(self)))
    }
}
