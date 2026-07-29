//! Business Need Discovery — Identifies business needs from analysis input

use genflow_receptors::BusinessAnalysisRequest;
use genflow_receptors::{BusinessNeed, BusinessNeedType, NeedUrgency};

pub struct BusinessNeedDiscovery;

impl Default for BusinessNeedDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl BusinessNeedDiscovery {
    pub fn new() -> Self {
        Self
    }

    /// Discover business needs from the analysis request
    pub fn discover(&self, request: &BusinessAnalysisRequest) -> Vec<BusinessNeed> {
        let mut needs = Vec::new();

        match &request.input_mode {
            genflow_receptors::BusinessInputMode::Swot {
                weaknesses,
                opportunities,
                ..
            } => {
                for weakness in weaknesses {
                    needs.push(BusinessNeed::new(
                        BusinessNeedType::CapabilityGap,
                        weakness,
                        NeedUrgency::ShortTerm,
                    ));
                }
                for opportunity in opportunities {
                    needs.push(BusinessNeed::new(
                        BusinessNeedType::GrowthOpportunity,
                        opportunity,
                        NeedUrgency::MediumTerm,
                    ));
                }
            }
            genflow_receptors::BusinessInputMode::GapAnalysis { pain_points, .. } => {
                for pain in pain_points {
                    needs.push(BusinessNeed::new(
                        BusinessNeedType::ProcessBottleneck,
                        pain,
                        NeedUrgency::Immediate,
                    ));
                }
            }
            genflow_receptors::BusinessInputMode::DirectRequest {
                requested_title,
                reason,
                ..
            } => {
                needs.push(BusinessNeed::new(
                    BusinessNeedType::DirectPositionRequest,
                    format!("{requested_title} — {reason}"),
                    NeedUrgency::Immediate,
                ));
            }
        }

        // Sort by priority
        needs.sort_by_key(|n| n.priority_score());

        tracing::info!(needs_count = needs.len(), "Business needs discovered");
        needs
    }
}
