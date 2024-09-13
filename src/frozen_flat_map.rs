use std::ops::{Bound, RangeBounds};

/// Memory-efficient immutable map backed by a contiguous flat array.
/// The implementation is identical to [`FlatMap`], 
/// except that [`Box<[T]>`] is used instead of [`Vec<T>`] to save memory.
pub struct FrozenFlatMap<K: Ord, V> {
    items: Box<[(K, V)]>,
}

/// If there are duplicates, the last one is kept.
impl<K: Ord, V> From<Vec<(K, V)>> for FrozenFlatMap<K, V> {
    fn from(mut items: Vec<(K, V)>) -> Self {
        items.reverse();
        items.sort_by(|a, b| K::cmp(&a.0, &b.0));
        items.dedup_by(|a, b| K::eq(&a.0, &b.0));
        FrozenFlatMap { items: items.into_boxed_slice() }
    }
}

impl<K: Ord + Clone, V: Clone> From<&[(K, V)]> for FrozenFlatMap<K, V> {
    fn from(value: &[(K, V)]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<K: Ord + Clone, V: Clone, const N: usize> From<[(K, V); N]> for FrozenFlatMap<K, V> {
    fn from(value: [(K, V); N]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FrozenFlatMap<K, V> {
    fn from_iter<I: IntoIterator<Item=(K, V)>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<K: Ord, V> FrozenFlatMap<K, V> {
    // lookup

    pub fn contains_key(&self, key: &K) -> bool {
        self.items
            .binary_search_by(|probe| K::cmp(&probe.0, key))
            .is_ok()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.items
            .binary_search_by(|probe| K::cmp(&probe.0, key))
            .ok()
            .map(|i| &self.items[i].1)
    }

    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.items
            .binary_search_by(|probe| K::cmp(&probe.0, key))
            .ok()
            .map(|i| {
                let (k, v) = &self.items[i];
                (k, v)
            })
    }

    pub fn range(&self, range: impl RangeBounds<K>) -> impl Iterator<Item=(&K, &V)> {
        let start_pos = match range.start_bound() {
            Bound::Included(key) => self
                .items
                .binary_search_by(|probe| K::cmp(&probe.0, key))
                .unwrap_or_else(|i| i),
            Bound::Excluded(key) => self
                .items
                .binary_search_by(|probe| K::cmp(&probe.0, key))
                .unwrap_or_else(|i| i + 1),
            Bound::Unbounded => 0,
        };

        let end_pos = match range.end_bound() {
            Bound::Included(key) => self
                .items
                .binary_search_by(|probe| K::cmp(&probe.0, key))
                .unwrap_or_else(|i| i + 1),
            Bound::Excluded(key) => self
                .items
                .binary_search_by(|probe| K::cmp(&probe.0, key))
                .unwrap_or_else(|i| i),
            Bound::Unbounded => self.items.len(),
        };

        self.items[start_pos..end_pos].iter().map(|(k, v)| (k, v))
    }

    // misc

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    // iterators

    pub fn iter(&self) -> impl Iterator<Item=(&K, &V)> {
        self.items.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item=&K> {
        self.items.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item=&V> {
        self.items.iter().map(|(_, v)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_key() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert!(m.contains_key(&1));
        assert!(m.contains_key(&3));
        assert!(m.contains_key(&5));
        assert!(!m.contains_key(&-100));
        assert!(!m.contains_key(&100));
    }

    #[test]
    fn test_get() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.get(&1), Some(&2));
        assert_eq!(m.get(&3), Some(&4));
        assert_eq!(m.get(&5), Some(&6));
        assert_eq!(m.get(&-100), None);
        assert_eq!(m.get(&100), None);
    }

    #[test]
    fn test_get_key_value() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.get_key_value(&1), Some((&1, &2)));
        assert_eq!(m.get_key_value(&3), Some((&3, &4)));
        assert_eq!(m.get_key_value(&5), Some((&5, &6)));
        assert_eq!(m.get_key_value(&-100), None);
        assert_eq!(m.get_key_value(&100), None);
    }

    #[test]
    fn test_range() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)]);
        assert_eq!(m.range(2..8).collect::<Vec<_>>(), vec![(&3, &4), (&5, &6), (&7, &8)]);
    }

    #[test]
    fn test_len() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_iter() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.iter().collect::<Vec<_>>(), vec![(&1, &2), (&3, &4), (&5, &6)]);
    }
    
    #[test]
    fn test_keys() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.keys().collect::<Vec<_>>(), vec![&1, &3, &5]);
    }

    #[test]
    fn test_values() {
        let m = FrozenFlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.values().collect::<Vec<_>>(), vec![&2, &4, &6]);
    }
}


