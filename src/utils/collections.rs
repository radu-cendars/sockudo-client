//! Collection utility functions.

use std::collections::HashMap;
use std::hash::Hash;

/// Apply a function to each element of a collection
pub fn apply<T, F>(items: &[T], mut f: F)
where
    F: FnMut(&T),
{
    for item in items {
        f(item);
    }
}

/// Map a function over a collection
pub fn map<T, U, F>(items: &[T], f: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    items.iter().map(f).collect()
}

/// Filter a collection
pub fn filter<T, F>(items: Vec<T>, f: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    items.into_iter().filter(f).collect()
}

/// Get keys from a HashMap
pub fn keys<K: Clone, V>(map: &HashMap<K, V>) -> Vec<K> {
    map.keys().cloned().collect()
}

/// Get values from a HashMap
pub fn values<K, V: Clone>(map: &HashMap<K, V>) -> Vec<V> {
    map.values().cloned().collect()
}

/// Apply a function to each entry in a HashMap
pub fn object_apply<K, V, F>(map: &HashMap<K, V>, mut f: F)
where
    F: FnMut(&K, &V),
{
    for (k, v) in map {
        f(k, v);
    }
}

/// Extend/merge two objects (latter takes precedence)
pub fn extend<K: Eq + Hash + Clone, V: Clone>(
    base: HashMap<K, V>,
    extension: HashMap<K, V>,
) -> HashMap<K, V> {
    let mut result = base;
    for (k, v) in extension {
        result.insert(k, v);
    }
    result
}

/// Check if a string is blank (empty or whitespace only)
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Flatten nested arrays
pub fn flatten<T: Clone>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply() {
        let items = vec![1, 2, 3];
        let mut sum = 0;
        apply(&items, |x| sum += x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_map() {
        let items = vec![1, 2, 3];
        let doubled = map(&items, |x| x * 2);
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn test_filter() {
        let items = vec![1, 2, 3, 4, 5];
        let evens = filter(items, |x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
    }

    #[test]
    fn test_extend() {
        let mut base = HashMap::new();
        base.insert("a", 1);
        base.insert("b", 2);
        
        let mut ext = HashMap::new();
        ext.insert("b", 3);
        ext.insert("c", 4);
        
        let result = extend(base, ext);
        assert_eq!(result.get(&"a"), Some(&1));
        assert_eq!(result.get(&"b"), Some(&3)); // overwritten
        assert_eq!(result.get(&"c"), Some(&4));
    }
}
