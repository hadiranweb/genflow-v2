#![allow(clippy::all)]
#![allow(unused)]
//! Missing Data handling

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum FillStrategy {
    Mean,
    Median,
    Mode,
    ForwardFill,
    Constant(f32),
}

pub fn handle_missing_score(
    score: Option<f32>,
    fallback_mean: Option<f32>,
    strategy: FillStrategy,
) -> Option<f32> {
    match score {
        Some(s) if s.is_finite() => Some(s),
        _ => match strategy {
            FillStrategy::Mean => fallback_mean,
            FillStrategy::Constant(c) => Some(c),
            _ => fallback_mean,
        },
    }
}

pub fn group_impute(
    candidate_group_key: &str,
    group_means: &HashMap<String, f32>,
    overall_mean: f32,
) -> f32 {
    group_means
        .get(candidate_group_key)
        .copied()
        .unwrap_or(overall_mean)
}

pub fn is_missing<T>(opt: &Option<T>) -> bool {
    opt.is_none()
}

pub fn is_present<T>(opt: &Option<T>) -> bool {
    opt.is_some()
}
