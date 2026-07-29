//! Representative Calibrator — Adjusts work style axis based on representative influence

use genflow_receptors::{
    AxisCode, PositionGraph, RepresentativeInfluencePolicy, RepresentativeRelation,
};

pub struct RepresentativeCalibrator;

impl Default for RepresentativeCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

impl RepresentativeCalibrator {
    pub fn new() -> Self {
        Self
    }

    /// Apply representative calibration to the work style axis
    /// Only affects Work Style — never hard requirements
    pub fn calibrate(
        &self,
        graph: &mut PositionGraph,
        relation: RepresentativeRelation,
        requested_weight: f32,
        use_personality: bool,
    ) -> Result<(), genflow_receptors::PolicyError> {
        let policy =
            RepresentativeInfluencePolicy::new(use_personality, relation, requested_weight)?;

        // Only modify the Work Style axis
        for axis in &mut graph.axes {
            if axis.code == AxisCode::WorkStyle {
                // Adjust weight: representative can shift work_style weight up to effective_weight
                axis.calibration_applied = true;
                // Don't change dimensions — only the axis weight changes slightly
                let original_weight = axis.weight;
                let calibration_shift = policy.effective_weight() * 0.10;
                axis.weight = original_weight + calibration_shift;

                tracing::info!(
                    axis = "work_style",
                    original = %original_weight,
                    calibrated = %axis.weight,
                    effective_weight = %policy.effective_weight(),
                    "Work style axis calibrated"
                );
            }
        }

        // Record calibration notes
        graph.calibration_notes = Some(format!(
            "Representative calibration: relation={}, effective_weight={:.3}, personality={}",
            relation.as_db_str(),
            policy.effective_weight(),
            policy.uses_personality(),
        ));

        Ok(())
    }
}
