## GenFlow v2 — 5-Axis Matching Algorithm

### Overview

The matching engine calculates compatibility between a position graph and a candidate profile across 5 dimensions:

| Axis | Weight (default) | What it measures |
|------|------------------|------------------|
| **Capability** | 0.25 | Knowledge, skills, abilities vs position requirements |
| **Output KPI** | 0.25 | Expected results and measurable outcomes |
| **Business Gap** | 0.20 | How well the candidate closes the business gap |
| **Work Style** | 0.20 | Personality/behavioral alignment (Big Five influence) |
| **Growth Motivation** | 0.10 | Learning drive, career trajectory alignment |

### Composite Index Calculation

```
composite = Σ(axis_match_percentage × axis_weight) / Σ(axis_weight)
```

Weights are configurable per position and can be calibrated by representative influence (only affects Work Style axis).

### Human Review Thresholds

- **Automatic**: composite ≥ 60 AND no `ActionRequired` risk flags
- **Review required**: composite < 60 OR any `ActionRequired` flag

### Risk Flags

| Flag | Severity | Condition |
|------|----------|-----------|
| `work_style_low` | Attention | Work Style alignment < 40 |
| `stress_sensitivity` | Info | Candidate neuroticism score is high (≥ 80) |

### Representative Calibration

Representative influence **only modifies the Work Style axis weight** — it never changes hard requirements or other axes:

```
calibration_shift = policy.effective_weight × 0.10
axis.weight = original_weight + calibration_shift
```

Maximum effective weights by relation:
- Owner: 0.30 (0.35 with personality)
- SeniorManager: 0.20
- Manager: 0.15
- Advisor: 0.10
- External: 0.05

Personality data can only be used by Owner, SeniorManager, and Manager relations.
