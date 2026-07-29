//! Position Graph Builder — Constructs the 5-axis position graph

use genflow_receptors::{
    AxisCode, AxisWeights, DimensionRequirement, PositionGraph, PositionGraphAxis, Score,
};
use uuid::Uuid;

pub struct PositionGraphBuilder;

impl Default for PositionGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionGraphBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build a position graph from needs and weights
    pub fn build(&self, position_id: Uuid, weights: &AxisWeights) -> PositionGraph {
        let axes = vec![
            self.build_axis(AxisCode::Capability, weights.capability, position_id),
            self.build_axis(AxisCode::OutputKpi, weights.output_kpi, position_id),
            self.build_axis(AxisCode::BusinessGap, weights.business_gap, position_id),
            self.build_axis(AxisCode::WorkStyle, weights.work_style, position_id),
            self.build_axis(
                AxisCode::GrowthMotivation,
                weights.growth_motivation,
                position_id,
            ),
        ];

        PositionGraph {
            position_id,
            version: "1.0".to_string(),
            axes,
            calibration_notes: None,
        }
    }

    fn build_axis(&self, code: AxisCode, weight: f32, _position_id: Uuid) -> PositionGraphAxis {
        let description = match code {
            AxisCode::Capability => "دانش، مهارت و توانایی‌های مورد نیاز",
            AxisCode::OutputKpi => "نتایج و KPI‌های مورد انتظار",
            AxisCode::BusinessGap => "فاصله بین وضعیت فعلی و مطلوب",
            AxisCode::WorkStyle => "سبک کار و نحوه همکاری",
            AxisCode::GrowthMotivation => "انگیزه رشد و توسعه",
        };

        PositionGraphAxis {
            code,
            weight,
            description: description.to_string(),
            dimensions: vec![DimensionRequirement {
                code: format!("{}_primary", code.as_str()),
                description: format!("Primary dimension for {}", code.as_str()),
                min: Some(Score::new(40.0).unwrap()),
                ideal: Some(Score::new(70.0).unwrap()),
                max: Some(Score::new(95.0).unwrap()),
                is_mandatory: true,
            }],
            calibration_applied: false,
        }
    }
}
