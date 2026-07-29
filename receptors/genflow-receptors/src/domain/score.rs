//! Score Type — Validated numeric score in range 0–100
//!
//! Used for confidence, dimension requirements, matching scores, and metrics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Score value (0.0 – 100.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Score(pub f32);

impl Score {
    /// Create a new Score. Returns `None` if value is outside [0, 100].
    pub fn new(value: f32) -> Option<Self> {
        if (0.0..=100.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Create a Score without validation (use with caution)
    pub fn new_unchecked(value: f32) -> Self {
        Self(value.clamp(0.0, 100.0))
    }

    /// Zero score
    pub fn zero() -> Self {
        Self(0.0)
    }

    /// Maximum score
    pub fn max() -> Self {
        Self(100.0)
    }

    /// Get the raw f32 value
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Is this score considered high (>= 80)?
    pub fn is_high(&self) -> bool {
        self.0 >= 80.0
    }

    /// Is this score considered medium (40–80)?
    pub fn is_medium(&self) -> bool {
        self.0 >= 40.0 && self.0 < 80.0
    }

    /// Is this score considered low (< 40)?
    pub fn is_low(&self) -> bool {
        self.0 < 40.0
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

impl Default for Score {
    fn default() -> Self {
        Self(50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_validation() {
        assert!(Score::new(75.0).is_some());
        assert!(Score::new(0.0).is_some());
        assert!(Score::new(100.0).is_some());
        assert!(Score::new(-1.0).is_none());
        assert!(Score::new(101.0).is_none());
    }

    #[test]
    fn test_score_levels() {
        let high = Score::new(85.0).unwrap();
        let medium = Score::new(60.0).unwrap();
        let low = Score::new(25.0).unwrap();

        assert!(high.is_high());
        assert!(!high.is_medium());
        assert!(medium.is_medium());
        assert!(low.is_low());
    }
}
