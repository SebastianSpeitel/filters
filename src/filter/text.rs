use super::{Filter, Optimizable};

#[derive(Debug)]
pub struct TextFilter {
    search: Box<str>,
}

impl TextFilter {
    /// If the filter matches exactly one string, return that string.
    pub fn exact(&self) -> Option<&str> {
        Some(&self.search)
    }
}

impl Filter<str> for TextFilter {
    #[inline]
    fn matches(&self, obj: &str) -> bool {
        self.search.as_ref() == obj
    }
}

impl Optimizable for TextFilter {}

impl From<String> for TextFilter {
    #[inline]
    fn from(value: String) -> Self {
        Self {
            search: value.into_boxed_str(),
        }
    }
}

impl From<&str> for TextFilter {
    #[inline]
    fn from(value: &str) -> Self {
        Self::from(value.to_owned())
    }
}
