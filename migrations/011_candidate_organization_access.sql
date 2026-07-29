-- Candidate Identity and Organization Access Boundary
--
-- Candidates are people, not copies of a person per tenant. Their relationship
-- to an organization is established through an invitation/position lifecycle.
-- This table makes that relationship explicit without forcing an
-- organization_id onto the shared candidate identity aggregate.

CREATE TABLE candidate_organization_access (
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    first_position_id UUID REFERENCES job_positions(id) ON DELETE SET NULL,
    first_invite_id UUID REFERENCES position_invites(id) ON DELETE SET NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (candidate_id, organization_id)
);

CREATE INDEX idx_candidate_org_access_organization
    ON candidate_organization_access(organization_id, candidate_id);

-- Preserve knowledge already represented by accepted invitations before new
-- acceptance flows begin writing the explicit access relation.
INSERT INTO candidate_organization_access (
    candidate_id,
    organization_id,
    first_position_id,
    first_invite_id
)
SELECT
    invitation.candidate_id,
    position.organization_id,
    invitation.position_id,
    invitation.id
FROM position_invites invitation
JOIN job_positions position ON position.id = invitation.position_id
WHERE invitation.candidate_id IS NOT NULL
ON CONFLICT (candidate_id, organization_id) DO NOTHING;

-- This relation is tenant-owned even though the candidate identity itself is
-- cross-tenant. Candidate profile RLS can migrate to this relation in a later
-- compatibility-safe step after invitation acceptance has been exercised in
-- production with a non-owner database role.
ALTER TABLE candidate_organization_access ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_candidate_organization_access ON candidate_organization_access
    FOR ALL
    USING (organization_id = current_tenant_organization_id())
    WITH CHECK (organization_id = current_tenant_organization_id());
