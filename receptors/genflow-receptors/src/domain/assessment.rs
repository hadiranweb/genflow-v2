//! Assessment Domain — Candidate assessment profiles
//!
//! Big Five, RIASEC, and skill mapping — inputs to the 5-axis matching engine.

use crate::domain::score::Score;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// پروفایل کامل کاندیدا (برای matching engine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateProfile {
    pub candidate_id: Uuid,
    pub big_five: Option<BigFiveScores>,
    pub riasec: Option<RiasecScores>,
    pub skills: HashMap<String, f32>,
    pub experience_years: Option<u32>,
}

impl CandidateProfile {
    /// دریافت نمره skill با keyword matching
    pub fn get_skill_score(&self, description: &str) -> Option<f32> {
        if let Some(score) = self.skills.get(description) {
            return Some(*score);
        }

        for (key, value) in &self.skills {
            if key.contains(description) || description.contains(key) {
                return Some(*value);
            }
        }

        None
    }
}

/// Big Five Personality Assessment Scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigFiveScores {
    pub openness: Score,
    pub conscientiousness: Score,
    pub extraversion: Score,
    pub agreeableness: Score,
    pub neuroticism: Score,
}

impl BigFiveScores {
    pub fn average(&self) -> f32 {
        let scores = [
            self.openness.value(),
            self.conscientiousness.value(),
            self.extraversion.value(),
            self.agreeableness.value(),
            self.neuroticism.value(),
        ];
        scores.iter().sum::<f32>() / scores.len() as f32
    }

    pub fn is_valid(&self) -> bool {
        Score::new(self.openness.value()).is_some()
            && Score::new(self.conscientiousness.value()).is_some()
            && Score::new(self.extraversion.value()).is_some()
            && Score::new(self.agreeableness.value()).is_some()
            && Score::new(self.neuroticism.value()).is_some()
    }
}

/// RIASEC Interest Assessment Scores (Holland Codes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiasecScores {
    pub realistic: Score,
    pub investigative: Score,
    pub artistic: Score,
    pub social: Score,
    pub enterprising: Score,
    pub conventional: Score,
}

impl RiasecScores {
    pub fn top_codes(&self) -> Vec<(String, f32)> {
        let codes: Vec<(String, f32)> = [
            ("R", self.realistic.value()),
            ("I", self.investigative.value()),
            ("A", self.artistic.value()),
            ("S", self.social.value()),
            ("E", self.enterprising.value()),
            ("C", self.conventional.value()),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
        let mut sorted = codes;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(3).collect()
    }
}

/// Assessment Method Codes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssessmentMethod {
    BigFive,
    Riasec,
    Values,
    SituationalJudgment,
    Custom(String),
}

impl AssessmentMethod {
    pub fn code(&self) -> &str {
        match self {
            Self::BigFive => "big_five",
            Self::Riasec => "riasec",
            Self::Values => "values",
            Self::SituationalJudgment => "situational_judgment",
            Self::Custom(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_skill_lookup() {
        let profile = CandidateProfile {
            candidate_id: Uuid::new_v4(),
            big_five: None,
            riasec: None,
            skills: HashMap::from([
                ("inventory_management".to_string(), 80.0),
                ("sales_pipeline".to_string(), 65.0),
            ]),
            experience_years: Some(5),
        };

        assert_eq!(profile.get_skill_score("inventory_management"), Some(80.0));
        assert_eq!(profile.get_skill_score("inventory"), Some(80.0));
        assert_eq!(profile.get_skill_score("unknown_skill"), None);
    }

    #[test]
    fn test_big_five_average() {
        let bf = BigFiveScores {
            openness: Score::new(70.0).unwrap(),
            conscientiousness: Score::new(85.0).unwrap(),
            extraversion: Score::new(60.0).unwrap(),
            agreeableness: Score::new(75.0).unwrap(),
            neuroticism: Score::new(40.0).unwrap(),
        };
        assert_eq!(bf.average(), 66.0);
    }

    #[test]
    fn test_riasec_top_codes() {
        let riasec = RiasecScores {
            realistic: Score::new(80.0).unwrap(),
            investigative: Score::new(70.0).unwrap(),
            artistic: Score::new(30.0).unwrap(),
            social: Score::new(60.0).unwrap(),
            enterprising: Score::new(90.0).unwrap(),
            conventional: Score::new(50.0).unwrap(),
        };
        let top = riasec.top_codes();
        assert_eq!(top[0].0, "E");
        assert_eq!(top[1].0, "R");
    }
}
