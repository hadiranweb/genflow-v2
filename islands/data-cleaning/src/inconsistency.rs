//! Inconsistency handling - case, whitespace, fuzzy matching
//! PDF: Case (Canada vs canada), Whitespace ( Germany), Fuzzy (Tehran vs Teharn)

use deunicode::deunicode;
use regex::Regex;
use strsim::normalized_levenshtein;

/// Equivalent to str.lower() + str.strip() + NFKD
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub fn normalize_phone(phone: &str) -> String {
    let re = Regex::new(r"[^\d+]").unwrap();
    re.replace_all(phone.trim(), "").to_string()
}

pub fn normalize_name(name: &str) -> String {
    // Remove extra spaces like PDF page 50
    name.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_skill(skill: &str) -> String {
    let lower = skill.trim().to_lowercase();
    // Replace - _ . with space, then normalize
    let unidecoded = deunicode(&lower);
    let re = Regex::new(r"[_\-\.]+").unwrap();
    let normalized = re.replace_all(&unidecoded, " ").to_string();
    normalized.split_whitespace().collect::<Vec<_>>().join("_")
}

/// Fuzzy match like fuzzywuzzy - score > threshold (0.0-1.0)
/// PDF page 53: score 90+ -> map to standard
pub fn fuzzy_match_skill(input: &str, dictionary: &[String], threshold: f64) -> Option<String> {
    let normalized_input = normalize_skill(input);
    let mut best_score = 0.0;
    let mut best_match: Option<String> = None;

    for standard in dictionary {
        let normalized_standard = normalize_skill(standard);
        let score = normalized_levenshtein(&normalized_input, &normalized_standard);
        if score > threshold && score > best_score {
            best_score = score;
            best_match = Some(standard.clone());
        }
        // Also check contains like old code but normalized
        if normalized_input.contains(&normalized_standard) || normalized_standard.contains(&normalized_input) {
            let contain_score = 0.85;
            if contain_score > best_score && contain_score >= threshold {
                best_score = contain_score;
                best_match = Some(standard.clone());
            }
        }
    }
    best_match
}

/// Skill dictionary standard (like canonical country list)
pub fn standard_skills() -> Vec<String> {
    vec![
        "react".into(),
        "inventory_management".into(),
        "sales_pipeline".into(),
        "project_management".into(),
        "data_analysis".into(),
        "leadership".into(),
        "communication".into(),
    ]
}
