use std::{cmp::Ordering, mem, ops::{Bound, RangeBounds}};

/// Memory-efficient map backed by a contiguous flat array.
///
/// Asymptotics:
///
/// | operation | average | worst   | best    |
/// |-----------|---------|---------|---------|
/// | lookup    | O(logn) | O(logn) | O(logn) |
/// | insert    | O(n)    | O(n)    | O(1)    |
/// | remove    | O(n)    | O(n)    | O(1)    |
///
/// Insert and remove work in O(1) if you are dealing with last element.
pub struct FlatMap<K: Ord, V> {
    items: Vec<(K, V)>,
}

/// If there are duplicates, the last one is kept.
impl<K: Ord, V> From<Vec<(K, V)>> for FlatMap<K, V> {
    fn from(mut items: Vec<(K, V)>) -> Self {
        items.reverse();
        items.sort_by(|a, b| K::cmp(&a.0, &b.0));
        items.dedup_by(|a, b| K::eq(&a.0, &b.0));
        FlatMap { items }
    }
}

impl<K: Ord + Clone, V: Clone> From<&[(K, V)]> for FlatMap<K, V> {
    fn from(value: &[(K, V)]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<K: Ord + Clone, V: Clone, const N: usize> From<[(K, V); N]> for FlatMap<K, V> {
    fn from(value: [(K, V); N]) -> Self {
        Self::from(value.to_vec())
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FlatMap<K, V> {
    fn from_iter<I: IntoIterator<Item=(K, V)>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

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

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.items
            .binary_search_by(|probe| K::cmp(&probe.0, key))
            .ok()
            .map(|i| &mut self.items[i].1)
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

    // modification

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some((last_key, last_value)) = &mut self.items.last_mut() {
            match K::cmp(last_key, &key) {
                Ordering::Less => {
                    self.items.push((key, value));
                    return None;
                }
                Ordering::Equal => {
                    return Some(mem::replace(last_value, value));
                }
                Ordering::Greater => {}
            }
        }

        match self.items.binary_search_by(|probe| K::cmp(&probe.0, &key)) {
            Ok(i) => self.items[i].1 = value,
            Err(i) => self.items.insert(i, (key, value)),
        }

        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some((last_key, _)) = &self.items.last() {
            match K::cmp(last_key, key) {
                Ordering::Less => return None,
                Ordering::Equal => {
                    return self.items.pop().map(|(_, v)| v);
                }
                Ordering::Greater => {}
            }
        }

        self.items
            .binary_search_by(|probe| K::cmp(&probe.0, key))
            .ok()
            .map(|i| self.items.remove(i).1)
    }

    // misc

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }


    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    // iterators

    pub fn iter(&self) -> impl Iterator<Item=(&K, &V)> {
        self.items.iter().map(|(k, v)| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&K, &mut V)> {
        self.items.iter_mut().map(|(k, v)| -> (&K, &mut V){ (k, v) })
    }

    pub fn keys(&self) -> impl Iterator<Item=&K> {
        self.items.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item=&V> {
        self.items.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut V> {
        self.items.iter_mut().map(|(_, v)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_key() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert!(m.contains_key(&1));
        assert!(m.contains_key(&3));
        assert!(m.contains_key(&5));
        assert!(!m.contains_key(&-100));
        assert!(!m.contains_key(&100));
    }

    #[test]
    fn test_get() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.get(&1), Some(&2));
        assert_eq!(m.get(&3), Some(&4));
        assert_eq!(m.get(&5), Some(&6));
        assert_eq!(m.get(&-100), None);
        assert_eq!(m.get(&100), None);
    }

    #[test]
    fn test_get_mut() {
        let mut m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        *m.get_mut(&3).unwrap() = 22;
        assert_eq!(m.get(&3), Some(&22));
    }

    #[test]
    fn test_get_key_value() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.get_key_value(&1), Some((&1, &2)));
        assert_eq!(m.get_key_value(&3), Some((&3, &4)));
        assert_eq!(m.get_key_value(&5), Some((&5, &6)));
        assert_eq!(m.get_key_value(&-100), None);
        assert_eq!(m.get_key_value(&100), None);
    }

    #[test]
    fn test_range() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)]);
        assert_eq!(m.range(2..8).collect::<Vec<_>>(), vec![(&3, &4), (&5, &6), (&7, &8)]);
    }

    #[test]
    fn test_insert() {
        let mut m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.insert(7, 8), None);
        assert_eq!(m.get(&7), Some(&8));
        assert_eq!(m.insert(7, 9), Some(8));
        assert_eq!(m.get(&7), Some(&9));
    }

    #[test]
    fn test_remove() {
        let mut m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.remove(&3), Some(4));
        assert_eq!(m.get(&3), None);
    }

    #[test]
    fn test_clear() {
        let mut m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn test_len() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn test_iter() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.iter().collect::<Vec<_>>(), vec![(&1, &2), (&3, &4), (&5, &6)]);
    }

    #[test]
    fn test_iter_mut() {
        let mut m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        let mut it = m.iter_mut();
        it.next();
        *it.next().unwrap().1 = 22;
        drop(it);
        assert_eq!(m.get(&3), Some(&22));
    }

    #[test]
    fn test_keys() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.keys().collect::<Vec<_>>(), vec![&1, &3, &5]);
    }

    #[test]
    fn test_values() {
        let m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        assert_eq!(m.values().collect::<Vec<_>>(), vec![&2, &4, &6]);
    }

    #[test]
    fn test_values_mut() {
        let mut m = FlatMap::from([(1, 2), (3, 4), (5, 6)]);
        m.values_mut().for_each(|v| *v = 22);
        assert_eq!(m.iter().collect::<Vec<_>>(), vec![(&1, &22), (&3, &22), (&5, &22)]);
    }
}


