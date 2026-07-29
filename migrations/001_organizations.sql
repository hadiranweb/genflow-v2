-- ============================================================
-- Migration 001: Organizations & Business Representatives
-- Prerequisite tables for all other migrations
-- ============================================================

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ============================================================
-- 1. Organizations
-- ============================================================
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    industry_code VARCHAR(50),
    company_size VARCHAR(50), -- '1-10', '11-50', '51-200', '200+'
    stage VARCHAR(50),        -- 'startup', 'growth', 'mature', 'enterprise'
    business_data JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_org_industry ON organizations(industry_code);

-- ============================================================
-- 2. Business Representatives
-- ============================================================
CREATE TABLE business_representatives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    email VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    role_title VARCHAR(255),
    
    -- Representative influence fields
    relation_to_position VARCHAR(50) CHECK (relation_to_position IN (
        'business_owner', 'senior_manager', 'direct_manager', 'hr_manager',
        'external_advisor', 'consultant'
    )),
    personality_influence_enabled BOOLEAN DEFAULT false,
    requested_influence_weight DECIMAL(3,2) DEFAULT 0.10
        CHECK (requested_influence_weight BETWEEN 0 AND 1),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_rep_org ON business_representatives(organization_id);
CREATE UNIQUE INDEX uniq_rep_email ON business_representative(email);

-- ============================================================
-- 3. Business Analyses
-- ============================================================
CREATE TABLE business_analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by_rep_id UUID NOT NULL REFERENCES business_representatives(id),
    
    title VARCHAR(255),
    analysis_type VARCHAR(50) CHECK (analysis_type IN (
        'swot', 'gap_analysis', 'direct_request'
    )),
    status VARCHAR(30) DEFAULT 'draft' CHECK (status IN (
        'draft', 'in_progress', 'completed', 'archived'
    )),
    
    input_data JSONB DEFAULT '{}',
    result_summary JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_analysis_org ON business_analyses(organization_id);
