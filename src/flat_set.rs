use crate::FlatMap;

struct NoValue;

pub struct FlatSet<K: Ord> {
    inner: FlatMap<K, NoValue>,
}

impl<K: Ord> From<Vec<K>> for FlatSet<K> {
    fn from(mut values: Vec<K>) -> Self {
        FlatSet {
            inner: FlatMap::from(values.drain(..)
                                       .map(|k| (k, NoValue))
                                       .collect::<Vec<_>>())
        }
    }
}

impl<K: Ord + Clone> From<&[K]> for FlatSet<K> {
    fn from(values: &[K]) -> Self {
        Self::from(values.to_vec())
    }
}

impl<K: Ord + Clone, const N: usize> From<[K; N]> for FlatSet<K> {
    fn from(value: [K; N]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<K: Ord> FromIterator<K> for FlatSet<K> {
    fn from_iter<I: IntoIterator<Item=K>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<K: Ord> FlatSet<K> {
    pub fn new() -> Self {
        Self { inner: FlatMap::new() }
    }

    // lookup

    pub fn contains(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    // modification

    pub fn insert(&mut self, key: K) -> bool {
        self.inner.insert(key, NoValue).is_none()
    }

    pub fn remove(&mut self, key: &K) {
        self.inner.remove(key);
    }

    // misc

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    // iterators

    pub fn iter(&self) -> impl Iterator<Item=&K> {
        // BTreeSet::inser
        self.inner.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let mut m = FlatSet::from([1, 2, 3]);
        assert!(m.contains(&1));
        assert!(m.contains(&2));
        assert!(m.contains(&3));
        assert!(!m.contains(&-100));
        assert!(!m.contains(&100));
    }

    #[test]
    fn test_insert() {
        let mut m = FlatSet::from([1, 2, 3]);
        assert!(m.insert(4));
        assert!(!m.insert(4));
    }

    #[test]
    fn test_remove() {
        let mut m = FlatSet::from([1, 2, 3]);
        m.remove(&2);
        assert!(!m.contains(&2));
    }

    #[test]
    fn test_is_empty() {
        let mut m = FlatSet::from([1, 2, 3]);
        assert!(!m.is_empty());
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn test_len() {
        let m = FlatSet::from([1, 2, 3]);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_iter() {
        let m = FlatSet::from([1, 2, 3]);
        assert_eq!(m.iter().count(), 3);
    }
}
