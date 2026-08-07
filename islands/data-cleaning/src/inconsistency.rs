#![allow(clippy::all)]
#![allow(unused)]
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
    match Regex::new(r"[^\d+]") {
        Ok(re) => re.replace_all(phone.trim(), "").to_string(),
        Err(_) => phone.trim().to_string(),
    }
}

pub fn normalize_name(name: &str) -> String {
    name.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_skill(skill: &str) -> String {
    let lower = skill.trim().to_lowercase();
    let unidecoded = deunicode(&lower);
    let re = Regex::new(r"[_\-\.]+").unwrap_or_else(|_| Regex::new(r"_").unwrap());
    let normalized = re.replace_all(&unidecoded, " ").to_string();
    normalized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

/// Fuzzy match like fuzzywuzzy - score > threshold (0.0-1.0)
pub fn fuzzy_match_skill(input: &str, dictionary: &[String], threshold: f64) -> Option<String> {
    let normalized_input = normalize_skill(input);
    let mut best_score = 0.0_f64;
    let mut best_match: Option<String> = None;

    for standard in dictionary {
        let normalized_standard = normalize_skill(standard);
        let score = normalized_levenshtein(&normalized_input, &normalized_standard);
        if score > threshold && score > best_score {
            best_score = score;
            best_match = Some(standard.clone());
        }
        if normalized_input.contains(&normalized_standard)
            || normalized_standard.contains(&normalized_input)
        {
            let contain_score = 0.85_f64;
            if contain_score > best_score && contain_score >= threshold {
                best_score = contain_score;
                best_match = Some(standard.clone());
            }
        }
    }
    best_match
}

pub fn standard_skills() -> Vec<String> {
    vec![
        "react".to_string(),
        "inventory_management".to_string(),
        "sales_pipeline".to_string(),
        "project_management".to_string(),
        "data_analysis".to_string(),
        "leadership".to_string(),
        "communication".to_string(),
    ]
}
