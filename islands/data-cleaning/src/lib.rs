//! GenFlow Data Cleaning Island
//! Inspired by Data Cleaning PDF (Missing, Duplicates, Inconsistency)
//! Maps Pandas concepts to Rust
#![allow(clippy::all)]
#![allow(unused)]

pub mod missing;
pub mod duplicates;
pub mod inconsistency;

pub use missing::{FillStrategy, group_impute, handle_missing_score};
pub use duplicates::{DuplicateKey, dedup_candidates, find_duplicates};
pub use inconsistency::{fuzzy_match_skill, normalize_email, normalize_phone, normalize_skill};

/// Main pipeline - cleans candidate input before matching engine
pub fn clean_candidate_pipeline(
    email: Option<String>,
    phone: Option<String>,
    full_name: Option<String>,
) -> CleanedIdentity {
    CleanedIdentity {
        email: email
            .map(|e| normalize_email(&e))
            .filter(|e| !e.is_empty()),
        phone: phone
            .map(|p| normalize_phone(&p))
            .filter(|p| !p.is_empty()),
        full_name: full_name
            .map(|n| inconsistency::normalize_name(&n))
            .filter(|n| !n.is_empty()),
    }
}

#[derive(Debug, Clone)]
pub struct CleanedIdentity {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub full_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_email_normalization() {
        assert_eq!(normalize_email("  HADI@Example.COM "), "hadi@example.com");
    }
    #[test]
    fn test_skill_fuzzy() {
        let dict = vec!["react".to_string(), "inventory_management".to_string()];
        assert_eq!(
            fuzzy_match_skill("React.js", &dict, 0.8),
            Some("react".to_string())
        );
        assert_eq!(
            fuzzy_match_skill("inventory", &dict, 0.6),
            Some("inventory_management".to_string())
        );
    }
}
