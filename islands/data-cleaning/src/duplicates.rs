//! Duplicates handling - equivalent to df.duplicated() and drop_duplicates(subset=[...])

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DuplicateKey {
    pub normalized_email: Option<String>,
    pub position_id: String,
}

pub fn find_duplicates(keys: &[DuplicateKey]) -> Vec<Vec<usize>> {
    let mut map: HashMap<DuplicateKey, Vec<usize>> = HashMap::new();
    for (idx, key) in keys.iter().enumerate() {
        map.entry(key.clone()).or_default().push(idx);
    }
    map.into_values().filter(|v| v.len() > 1).collect()
}

/// dedup like drop_duplicates(keep='last') - keeps last occurrence
pub fn dedup_candidates(keys: Vec<DuplicateKey>) -> Vec<DuplicateKey> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    // iterate reverse to keep last
    for key in keys.into_iter().rev() {
        if seen.insert(key.clone()) {
            result.push(key);
        }
    }
    result.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_duplicate_detection() {
        let keys = vec![
            DuplicateKey { normalized_email: Some("hadi@test.com".into()), position_id: "1".into() },
            DuplicateKey { normalized_email: Some("hadi@test.com".into()), position_id: "1".into() },
            DuplicateKey { normalized_email: Some("ali@test.com".into()), position_id: "1".into() },
        ];
        let dups = find_duplicates(&keys);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].len(), 2);
    }
}
