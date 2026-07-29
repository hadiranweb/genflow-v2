-- Sprint 2: Position Generation Engine Schema
-- Depends on: 003_mcp_registry.sql

-- ============================================================
-- 1. Business Needs (کشف شده از تحلیل)
-- ============================================================
CREATE TABLE business_needs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_analysis_id UUID NOT NULL REFERENCES business_analyses(id) ON DELETE CASCADE,
    
    need_id VARCHAR(100) NOT NULL, -- مثلاً NEED-001
    need_type VARCHAR(50) NOT NULL CHECK (need_type IN (
        'capability_gap', 'process_bottleneck', 'growth_opportunity', 
        'risk_mitigation', 'direct_position_request'
    )),
    
    description TEXT NOT NULL,
    related_process VARCHAR(100),
    related_capabilities JSONB DEFAULT '[]',
    urgency VARCHAR(20) CHECK (urgency IN ('immediate', 'short_term', 'medium_term')),
    
    evidence_refs JSONB DEFAULT '[]',
    source_mcp_id UUID REFERENCES mcp_contexts(id), -- اگر از MCP آمده
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_needs_analysis ON business_needs(business_analysis_id, need_type);

-- ============================================================
-- 2. Position Generation Runs (Audit Trail کامل)
-- ============================================================
CREATE TABLE position_generation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_analysis_id UUID NOT NULL REFERENCES business_analyses(id) ON DELETE CASCADE,
    
    -- ورودی‌ها
    input_mode VARCHAR(50) NOT NULL,
    mcp_bundle_snapshot JSONB NOT NULL DEFAULT '{}',
    
    -- خروجی‌های intermediate
    discovered_needs_count INTEGER DEFAULT 0,
    generated_hypotheses JSONB DEFAULT '[]',
    selected_hypothesis_title VARCHAR(255),
    standard_position_match JSONB DEFAULT '{}',
    
    -- Representative Calibration (فقط context)
    rep_calibration_applied BOOLEAN DEFAULT false,
    rep_effective_weight DECIMAL(3,2),
    rep_calibration_notes TEXT,
    
    -- LLM Usage (برای description نهایی)
    used_llm BOOLEAN DEFAULT false,
    llm_tokens_used INTEGER DEFAULT 0,
    llm_reason TEXT,
    
    status VARCHAR(30) DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed')),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- ============================================================
-- 3. Job Positions (تولید نهایی)
-- ============================================================
CREATE TABLE job_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    created_by_rep_id UUID NOT NULL REFERENCES business_representatives(id),
    generation_run_id UUID REFERENCES position_generation_runs(id),
    
    position_code VARCHAR(50) UNIQUE NOT NULL, -- GEN-FLOW-ORG-001
    title VARCHAR(255) NOT NULL,
    description TEXT,
    
    generation_method VARCHAR(50) CHECK (generation_method IN (
        'business_analysis', 'direct_request', 'gap_driven'
    )),
    
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'paused', 'filled', 'archived')),
    
    -- Evidence & Audit
    generation_evidence JSONB DEFAULT '{}',
    standards_used JSONB DEFAULT '[]',
    advisory_disclaimer TEXT DEFAULT 'این خروجی صرفاً کمک‌تصمیم است و جایگزین ارزیابی تخصصی انسانی نیست.',
    
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 4. Position Graphs (۵ محور تطابق)
-- ============================================================
CREATE TABLE position_graphs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_position_id UUID NOT NULL UNIQUE REFERENCES job_positions(id) ON DELETE CASCADE,
    graph_version VARCHAR(20) DEFAULT '1.0.0',
    
    -- ۵ محور با weight و calibration
    capability_axis JSONB NOT NULL DEFAULT '{"weight": 0.25, "dimensions": []}',
    output_kpi_axis JSONB NOT NULL DEFAULT '{"weight": 0.25, "dimensions": []}',
    business_gap_axis JSONB NOT NULL DEFAULT '{"weight": 0.20, "dimensions": []}',
    work_style_axis JSONB NOT NULL DEFAULT '{"weight": 0.20, "dimensions": []}',
    growth_motivation_axis JSONB NOT NULL DEFAULT '{"weight": 0.10, "dimensions": []}',
    
    calibration_applied BOOLEAN DEFAULT false,
    calibration_notes TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 5. Position Requirements (نیازمندی‌های تفکیکی)
-- ============================================================
CREATE TABLE position_requirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_position_id UUID NOT NULL REFERENCES job_positions(id) ON DELETE CASCADE,
    
    axis_code VARCHAR(50) NOT NULL CHECK (axis_code IN (
        'capability', 'output_kpi', 'business_gap', 'work_style', 'growth_motivation'
    )),
    
    requirement_type VARCHAR(50) CHECK (requirement_type IN (
        'knowledge', 'skill', 'ability', 'personality_trait', 'experience', 'certification'
    )),
    
    description TEXT NOT NULL,
    importance VARCHAR(20) CHECK (importance IN ('critical', 'important', 'nice_to_have')),
    
    -- برای dimension-based requirements
    min_score INTEGER CHECK (min_score BETWEEN 0 AND 100),
    ideal_score INTEGER CHECK (ideal_score BETWEEN 0 AND 100),
    max_score INTEGER CHECK (max_score BETWEEN 0 AND 100),
    
    is_mandatory BOOLEAN DEFAULT false,
    source_type VARCHAR(50) CHECK (source_type IN (
        'business_need', 'industry_standard', 'process_standard', 
        'representative_context', 'generated'
    )),
    source_ref VARCHAR(100), -- reference به need_id یا standard
    
    rationale TEXT NOT NULL,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_reqs_position ON position_requirements(job_position_id, axis_code);

-- ============================================================
-- 6. Generation Warnings (هشدارهای غیرانگ‌زننده)
-- ============================================================
CREATE TABLE generation_warnings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    generation_run_id UUID NOT NULL REFERENCES position_generation_runs(id) ON DELETE CASCADE,
    
    warning_code VARCHAR(100) NOT NULL CHECK (warning_code IN (
        'high_representative_influence',
        'missing_industry_standard',
        'ambiguous_business_need',
        'low_confidence_match',
        'potential_bias_detected'
    )),
    
    severity VARCHAR(20) CHECK (severity IN ('info', 'warning', 'attention')),
    message TEXT NOT NULL,
    mitigation_suggestion TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);
