//! Missing Data handling - equivalent to pandas isnull(), fillna(), ffill, groupby().transform('mean')

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum FillStrategy {
    Mean,
    Median,
    Mode,
    ForwardFill,
    Constant(f32),
}

/// Handle missing score (like pd.NA) - if None, fill with strategy
/// Inspired by df['LDL'].fillna(df.groupby(...).transform('mean'))
pub fn handle_missing_score(score: Option<f32>, fallback_mean: Option<f32>, strategy: FillStrategy) -> Option<f32> {
    match score {
        Some(s) if s.is_finite() => Some(s),
        _ => match strategy {
            FillStrategy::Mean => fallback_mean,
            FillStrategy::Constant(c) => Some(c),
            _ => fallback_mean, // Simplified - real would need series
        },
    }
}

/// Group imputation - avg by (position_id, exp_years) group like PDF example
/// PDF: df.groupby(['AgeGroups','Smoke'])['LDL'].transform('mean')
pub fn group_impute(
    candidate_group_key: &str,
    group_means: &HashMap<String, f32>,
    overall_mean: f32,
) -> f32 {
    group_means.get(candidate_group_key).copied().unwrap_or(overall_mean)
}

/// isnull() equivalent in Rust -> Option::is_none()
pub fn is_missing<T>(opt: &Option<T>) -> bool {
    opt.is_none()
}

/// notnull() equivalent
pub fn is_present<T>(opt: &Option<T>) -> bool {
    opt.is_some()
}
