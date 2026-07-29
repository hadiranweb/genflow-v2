//! genflow-synaptic-hub — Dual-layer event bus
//!
//! Inspired by pema-platform-v2's "Synaptic Hub" concept:
//! Central event convergence and pattern matching.
//!
//! ## Architecture
//! - **Layer 1**: tokio mpsc channels — in-process, zero-cost, ultra-fast
//! - **Layer 2**: Redis pub/sub — cross-container, production-grade
//!
//! Events flow from Islands → Synaptic Hub → Receptors (other Islands)

pub mod bus;
pub mod convergence;
pub mod router;

pub use bus::SynapticBus;
pub use convergence::ConvergenceTracker;
pub use router::EventRouter;
