//! genflow-position-generation — Position Generation Island
//!
//! Business analysis → Need discovery → Graph → Calibration → Position generation

pub mod services;

pub use services::{
    BusinessAnalysisEngine, BusinessNeedDiscovery, PositionGenerationEngine, PositionGraphBuilder,
    RepresentativeCalibrator,
};
