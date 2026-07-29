//! Representative Context & Influence Policy
//!
//! Determines how business representatives influence position generation.
//! Representative calibration only affects Work Style axis — never hard requirements.

use serde::{Deserialize, Serialize};

/// نوع رابطه نماینده با سازمان
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepresentativeRelation {
    Owner,         // صاحب کسب‌وکار
    SeniorManager, // مدیر ارشد
    Manager,       // مدیر میانی
    Advisor,       // مشاور
    External,      // خارجی
}

impl RepresentativeRelation {
    /// حداکثر وزن مجاز برای این رابطه
    pub fn max_allowed_weight(&self) -> f32 {
        match self {
            Self::Owner => 0.30,
            Self::SeniorManager => 0.20,
            Self::Manager => 0.15,
            Self::Advisor => 0.10,
            Self::External => 0.05,
        }
    }

    /// آیا این رابطه اجازه استفاده از personality data دارد؟
    pub fn can_use_personality(&self) -> bool {
        matches!(self, Self::Owner | Self::SeniorManager | Self::Manager)
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::SeniorManager => "senior_manager",
            Self::Manager => "manager",
            Self::Advisor => "advisor",
            Self::External => "external",
        }
    }
}

/// Policy influence نماینده
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentativeInfluencePolicy {
    use_personality: bool,
    relation: RepresentativeRelation,
    requested_weight: f32,
    effective_weight: f32,
}

impl RepresentativeInfluencePolicy {
    /// ساخت Policy با validation
    pub fn new(
        use_personality: bool,
        relation: RepresentativeRelation,
        requested_weight: f32,
    ) -> Result<Self, PolicyError> {
        if use_personality && !relation.can_use_personality() {
            return Err(PolicyError::PersonalityNotAllowed);
        }

        if !(0.0..=1.0).contains(&requested_weight) {
            return Err(PolicyError::InvalidWeight(requested_weight));
        }

        let base = requested_weight.min(relation.max_allowed_weight());
        let personality_bonus = if use_personality && relation.can_use_personality() {
            0.05
        } else {
            0.0
        };
        let effective = (base + personality_bonus).min(0.35);

        Ok(Self {
            use_personality,
            relation,
            requested_weight,
            effective_weight: effective,
        })
    }

    pub fn effective_weight(&self) -> f32 {
        self.effective_weight
    }

    pub fn uses_personality(&self) -> bool {
        self.use_personality
    }

    pub fn relation(&self) -> RepresentativeRelation {
        self.relation
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyError {
    PersonalityNotAllowed,
    InvalidWeight(f32),
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PersonalityNotAllowed => {
                write!(f, "Personality data not allowed for this relation type")
            }
            Self::InvalidWeight(w) => write!(f, "Invalid weight: {w}"),
        }
    }
}

impl std::error::Error for PolicyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_max_weight() {
        assert_eq!(RepresentativeRelation::Owner.max_allowed_weight(), 0.30);
        assert_eq!(RepresentativeRelation::External.max_allowed_weight(), 0.05);
    }

    #[test]
    fn test_effective_weight_owner() {
        let policy =
            RepresentativeInfluencePolicy::new(true, RepresentativeRelation::Owner, 0.25).unwrap();
        assert_eq!(policy.effective_weight(), 0.30);
    }

    #[test]
    fn test_effective_weight_external() {
        let policy =
            RepresentativeInfluencePolicy::new(false, RepresentativeRelation::External, 0.10)
                .unwrap();
        assert_eq!(policy.effective_weight(), 0.05);
    }

    #[test]
    fn test_personality_not_allowed() {
        let result =
            RepresentativeInfluencePolicy::new(true, RepresentativeRelation::External, 0.05);
        assert!(result.is_err());
    }
}
