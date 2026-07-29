## GenFlow v2 — Synaptic Hub Event Flow

### Dual-Layer Architecture

The Synaptic Hub uses two complementary layers:

| Layer | Transport | Scope | Cost |
|-------|-----------|-------|------|
| **Layer 1** | tokio mpsc broadcast | In-process | Zero (no serialization) |
| **Layer 2** | Redis pub/sub | Cross-container | Network + serialization |

Events are published to **both layers** simultaneously. Subscribers choose which layer to listen on based on their deployment topology.

### Event Types

| Event | Source | Targets |
|-------|--------|---------|
| `mcp.resolved` | MCP Registry | Position Generation, Dashboard |
| `position.generated` | Position Generation | Candidate Matching, Dashboard |
| `position.analysis_completed` | Position Generation | Candidate Matching, Dashboard |
| `candidate.invited` | Candidate Matching | Dashboard |
| `match.calculated` | Candidate Matching | Dashboard |
| `report.generated` | Candidate Matching | Dashboard |
| `dashboard.metrics_updated` | Dashboard | Gateway |
| `dashboard.alert_triggered` | Dashboard | Gateway |

### Convergence Patterns

Convergence Tracker detects when correlated events from different islands arrive for the same organization:

| Pattern | Required Events | Action |
|---------|----------------|--------|
| `position_pipeline_init` | `mcp.resolved` + `position.generated` | Trigger candidate pipeline setup |
| `match_complete_notification` | `match.calculated` + `report.generated` | Notify dashboard |

### Usage

```rust
// Publish event
bus.publish(envelope).await?;

// Publish domain event (auto-wraps in envelope)
bus.publish_event(&mcp_event).await?;

// Subscribe to internal layer
let receiver = bus.subscribe_internal();
while let Ok(event) = receiver.recv().await {
    // Handle event
}
```
