# CI/CD Fix - 2026-07-29

## Problems Found
1. `release.yml` Line 41: `VERSION` undefined -> Invalid workflow file
   - Error: Unrecognized named-value: 'VERSION'
   - Fix: Use `inputs.version` + bash logic for title
2. `genflow.yml` branches: only [main-platform, v2-island-architecture] but default branch is `main` -> CI never triggers
   - Fix: Add `main` to branches
3. `docker/build-push-action@v6` -> v6 didn't exist at time, use v5 + setup-buildx
4. `cd.yml` uses `secrets[format(...)]` which is not allowed
   - Fix: Split into staging/production jobs with static secret names

## Data Cleaning Integration (from PDF)
New island: `islands/data-cleaning`
- missing.rs: isnull()/fillna()/groupby mean
- duplicates.rs: duplicated() + drop_duplicates(keep='last')
- inconsistency.rs: lower() + strip() + fuzzywuzzy (strsim crate)

Usage in candidate-matching:
```rust
use genflow_data_cleaning::{normalize_email, fuzzy_match_skill};
let clean_email = normalize_email(&input_email);
let skill = fuzzy_match_skill("React.js", &standard_skills(), 0.85);
```

## Authorization Matrix
Section 7 test now in `gateway/tests/authorization_matrix.rs`
Run: `cargo test authorization_matrix -- --nocapture`
Live: `cargo test -- --ignored`
