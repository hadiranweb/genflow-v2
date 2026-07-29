-- Tenant Context Boundaries
--
-- RLS belongs to the table that owns data, not to the gateway that happens to
-- call it. This migration keeps the existing monolith schema, but makes its
-- bounded contexts explicit: position generation, matching and analytics rows
-- are tenant-owned; candidate identity remains a cross-tenant subject linked by
-- invitations and therefore is intentionally not reclassified as tenant-owned.

-- Helper used by all policies. `missing_ok = true` returns NULL when the
-- transaction has no tenant context, which denies access rather than exposing
-- data from another organization.
CREATE OR REPLACE FUNCTION current_tenant_organization_id()
RETURNS UUID
LANGUAGE SQL
STABLE
AS $$
    SELECT NULLIF(current_setting('app.current_org_id', true), '')::UUID
$$;

-- Direct organization-owned roots.
ALTER TABLE business_representatives ENABLE ROW LEVEL SECURITY;
ALTER TABLE job_positions ENABLE ROW LEVEL SECURITY;
ALTER TABLE business_analyses ENABLE ROW LEVEL SECURITY;
ALTER TABLE activity_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_metrics ENABLE ROW LEVEL SECURITY;

-- Derived tables inherit their tenant from their owning root.
ALTER TABLE business_needs ENABLE ROW LEVEL SECURITY;
ALTER TABLE position_generation_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE position_graphs ENABLE ROW LEVEL SECURITY;
ALTER TABLE position_requirements ENABLE ROW LEVEL SECURITY;
ALTER TABLE generation_warnings ENABLE ROW LEVEL SECURITY;
ALTER TABLE position_invites ENABLE ROW LEVEL SECURITY;
ALTER TABLE job_matches ENABLE ROW LEVEL SECURITY;
ALTER TABLE match_risk_flags ENABLE ROW LEVEL SECURITY;
ALTER TABLE match_reports ENABLE ROW LEVEL SECURITY;
ALTER TABLE position_pipeline_stats ENABLE ROW LEVEL SECURITY;

-- Replace the original broad policies with explicit read/write rules. `WITH
-- CHECK` is required for INSERT and UPDATE; `USING` alone only expresses row
-- visibility and is not a complete tenant-write guarantee.
DROP POLICY IF EXISTS organization_isolation ON job_positions;
CREATE POLICY tenant_job_positions ON job_positions
    FOR ALL
    USING (organization_id = current_tenant_organization_id())
    WITH CHECK (organization_id = current_tenant_organization_id());

DROP POLICY IF EXISTS organization_isolation ON business_analyses;
CREATE POLICY tenant_business_analyses ON business_analyses
    FOR ALL
    USING (organization_id = current_tenant_organization_id())
    WITH CHECK (organization_id = current_tenant_organization_id());

DROP POLICY IF EXISTS organization_isolation ON job_matches;
CREATE POLICY tenant_job_matches ON job_matches
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = job_matches.position_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = job_matches.position_id
          AND position.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_business_representatives ON business_representatives
    FOR ALL
    USING (organization_id = current_tenant_organization_id())
    WITH CHECK (organization_id = current_tenant_organization_id());

CREATE POLICY tenant_activity_logs ON activity_logs
    FOR ALL
    USING (organization_id = current_tenant_organization_id())
    WITH CHECK (organization_id = current_tenant_organization_id());

CREATE POLICY tenant_organization_metrics ON organization_metrics
    FOR ALL
    USING (organization_id = current_tenant_organization_id())
    WITH CHECK (organization_id = current_tenant_organization_id());

CREATE POLICY tenant_business_needs ON business_needs
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM business_analyses analysis
        WHERE analysis.id = business_needs.business_analysis_id
          AND analysis.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM business_analyses analysis
        WHERE analysis.id = business_needs.business_analysis_id
          AND analysis.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_position_generation_runs ON position_generation_runs
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM business_analyses analysis
        WHERE analysis.id = position_generation_runs.business_analysis_id
          AND analysis.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM business_analyses analysis
        WHERE analysis.id = position_generation_runs.business_analysis_id
          AND analysis.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_position_graphs ON position_graphs
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_graphs.job_position_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_graphs.job_position_id
          AND position.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_position_requirements ON position_requirements
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_requirements.job_position_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_requirements.job_position_id
          AND position.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_generation_warnings ON generation_warnings
    FOR ALL
    USING (EXISTS (
        SELECT 1
        FROM position_generation_runs run
        JOIN business_analyses analysis ON analysis.id = run.business_analysis_id
        WHERE run.id = generation_warnings.generation_run_id
          AND analysis.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1
        FROM position_generation_runs run
        JOIN business_analyses analysis ON analysis.id = run.business_analysis_id
        WHERE run.id = generation_warnings.generation_run_id
          AND analysis.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_position_invites ON position_invites
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_invites.position_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_invites.position_id
          AND position.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_match_risk_flags ON match_risk_flags
    FOR ALL
    USING (EXISTS (
        SELECT 1
        FROM job_matches job_match
        JOIN job_positions position ON position.id = job_match.position_id
        WHERE job_match.id = match_risk_flags.job_match_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1
        FROM job_matches job_match
        JOIN job_positions position ON position.id = job_match.position_id
        WHERE job_match.id = match_risk_flags.job_match_id
          AND position.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_match_reports ON match_reports
    FOR ALL
    USING (EXISTS (
        SELECT 1
        FROM job_matches job_match
        JOIN job_positions position ON position.id = job_match.position_id
        WHERE job_match.id = match_reports.job_match_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1
        FROM job_matches job_match
        JOIN job_positions position ON position.id = job_match.position_id
        WHERE job_match.id = match_reports.job_match_id
          AND position.organization_id = current_tenant_organization_id()
    ));

CREATE POLICY tenant_position_pipeline_stats ON position_pipeline_stats
    FOR ALL
    USING (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_pipeline_stats.position_id
          AND position.organization_id = current_tenant_organization_id()
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM job_positions position
        WHERE position.id = position_pipeline_stats.position_id
          AND position.organization_id = current_tenant_organization_id()
    ));
