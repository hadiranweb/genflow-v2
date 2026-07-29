-- ============================================================
-- Sprint 1.6 - Phase 1: MCP Registry Foundation
-- Business Domain MCP Registry & Reuse Architecture
-- ============================================================

-- 0. Extensions
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 1. رجیستری اصلی MCP (Master Context Protocol)
CREATE TABLE mcp_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- === طبقه‌بندی ===
    mcp_type VARCHAR(50) NOT NULL CHECK (mcp_type IN (
        'platform_policy',      -- قوانین سراسری پلتفرم
        'industry',             -- استانداردهای صنعت
        'business_process',     -- فرایندهای کسب‌وکار
        'standard_position',    -- پوزیشن‌های استاندارد
        'organization_context', -- بافت سازمان خاص
        'case_temporary'        -- موقت برای یک تحلیل
    )),
    
    scope VARCHAR(50) NOT NULL CHECK (scope IN (
        'global',    -- سراسری
        'industry',  -- سطح صنعت
        'tenant',    -- سطح سازمان
        'case'       -- سطح کیس
    )),
    
    -- === شناسایی ===
    code VARCHAR(150) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- === نسخه‌بندی ===
    version VARCHAR(50) NOT NULL DEFAULT '0.1.0',
    status VARCHAR(30) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'review_ready', 'approved', 'active', 'deprecated', 'archived')),
    
    -- === محتوا و Hash ===
    content JSONB NOT NULL DEFAULT '{}',
    content_hash VARCHAR(128) NOT NULL,
    
    -- === متادیتا ===
    evidence JSONB NOT NULL DEFAULT '{}',
    source_refs JSONB NOT NULL DEFAULT '[]',
    source_quality_score DECIMAL(3,2)   
        CHECK (source_quality_score IS NULL OR source_quality_score BETWEEN 0 AND 1),
    
    -- === مالکیت و محدوده ===
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    industry_code VARCHAR(50),
    process_code VARCHAR(100),
    position_code VARCHAR(100),
    case_id UUID,
    
    -- === کنترل چرخه حیات ===
    reusable BOOLEAN NOT NULL DEFAULT true,
    cacheable BOOLEAN NOT NULL DEFAULT true,
    deterministic BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ,
    
    policy_version VARCHAR(50),
    schema_version VARCHAR(50) DEFAULT '0.1',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- === Constraints کلیدی ===
    
    -- یکتایی کامل
    UNIQUE(mcp_type, scope, code, version),
    
    -- سازگاری scope با type
    CONSTRAINT valid_mcp_scope CHECK (
        (mcp_type = 'platform_policy' AND scope = 'global')
        OR (mcp_type = 'industry' AND scope IN ('global', 'industry'))
        OR (mcp_type = 'business_process' AND scope IN ('global', 'industry'))
        OR (mcp_type = 'standard_position' AND scope IN ('global', 'industry'))
        OR (mcp_type = 'organization_context' AND scope = 'tenant')
        OR (mcp_type = 'case_temporary' AND scope = 'case')
    ),
    
    -- case_temporary نباید reusable باشد
    CONSTRAINT case_mcp_not_reusable CHECK (
        mcp_type <> 'case_temporary' OR reusable = false
    )
);

-- Index برای جستجوی سریع
CREATE INDEX idx_mcp_type_scope ON mcp_contexts(mcp_type, scope);
CREATE INDEX idx_mcp_code ON mcp_contexts(code);
CREATE INDEX idx_mcp_org ON mcp_contexts(organization_id) WHERE organization_id IS NOT NULL;
CREATE INDEX idx_mcp_industry ON mcp_contexts(industry_code) WHERE industry_code IS NOT NULL;
CREATE INDEX idx_mcp_status_active ON mcp_contexts(status) WHERE status = 'active';
CREATE INDEX idx_mcp_expires ON mcp_contexts(expires_at) WHERE expires_at IS NOT NULL;

-- فقط یک نسخه active برای هر MCP
CREATE UNIQUE INDEX uniq_active_mcp_context
ON mcp_contexts(mcp_type, scope, code)
WHERE status = 'active';

-- 2. پیوند MCPها (Composition & Extension)
CREATE TABLE mcp_context_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_mcp_id UUID NOT NULL REFERENCES mcp_contexts(id) ON DELETE CASCADE,
    child_mcp_id UUID NOT NULL REFERENCES mcp_contexts(id) ON DELETE CASCADE,
    
    link_type VARCHAR(50) NOT NULL CHECK (link_type IN (
        'uses',         -- از child استفاده می‌کند
        'extends',      -- child را گسترش می‌دهد
        'overrides',    -- child را بازنویسی می‌کند
        'composes',     -- بخشی از child است
        'derived_from'  -- از child مشتق شده
    )),
    
    weight DECIMAL(5,2) DEFAULT 1.00 CHECK (weight BETWEEN 0 AND 1),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- لینک خود به خود ممنوع
    CONSTRAINT no_self_link CHECK (parent_mcp_id <> child_mcp_id),
    
    UNIQUE(parent_mcp_id, child_mcp_id, link_type)
);

CREATE INDEX idx_mcp_links_parent ON mcp_context_links(parent_mcp_id);
CREATE INDEX idx_mcp_links_child ON mcp_context_links(child_mcp_id);

-- 3. تاریخچه تغییرات (Audit Trail)
CREATE TABLE mcp_context_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mcp_context_id UUID NOT NULL REFERENCES mcp_contexts(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    
    change_type VARCHAR(50) CHECK (change_type IN (
        'created', 'updated', 'reviewed', 'approved', 'deprecated', 'archived'
    )),
    
    change_summary TEXT,
    changed_by_user_id UUID REFERENCES business_representatives(id),
    
    previous_content_hash VARCHAR(128),
    new_content_hash VARCHAR(128),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_mcp_revisions_context ON mcp_context_revisions(mcp_context_id);
CREATE INDEX idx_mcp_revisions_date ON mcp_context_revisions(created_at DESC);

-- 4. استفاده MCP در تحلیل (Evidence Trail)
CREATE TABLE business_analysis_mcp_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_analysis_id UUID NOT NULL REFERENCES business_analyses(id) ON DELETE CASCADE,
    mcp_context_id UUID NOT NULL REFERENCES mcp_contexts(id),
    
    usage_role VARCHAR(50) NOT NULL CHECK (usage_role IN (
        'industry_standard',
        'process_standard',
        'position_standard',
        'organization_context',
        'case_context',
        'policy_guardrail'
    )),
    
    -- متریک‌های پرفورمنس
    prompt_included BOOLEAN DEFAULT false,
    rust_evaluated BOOLEAN DEFAULT true,
    cache_hit BOOLEAN DEFAULT false,
    token_savings_estimate INTEGER DEFAULT 0,
    
    usage_evidence JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- یک MCP نباید با یک نقش دوبار در یک تحلیل استفاده شود
    UNIQUE(business_analysis_id, mcp_context_id, usage_role)
);

CREATE INDEX idx_mcp_usage_analysis ON business_analysis_mcp_usage(business_analysis_id);
CREATE INDEX idx_mcp_usage_context ON business_analysis_mcp_usage(mcp_context_id);

-- 5. Prompt Fragments (برای کاهش توکن LLM)
CREATE TABLE mcp_prompt_fragments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mcp_context_id UUID NOT NULL REFERENCES mcp_contexts(id) ON DELETE CASCADE,
    
    fragment_key VARCHAR(150) NOT NULL,
    fragment_role VARCHAR(50) NOT NULL CHECK (fragment_role IN (
        'industry_summary',
        'common_processes',
        'common_roles',
        'standard_kpis',
        'common_bottlenecks',
        'position_requirements',
        'prompt_instruction',
        'compliance_warning'
    )),
    
    content TEXT NOT NULL,
    token_estimate INTEGER DEFAULT 0,
    content_hash VARCHAR(128) NOT NULL,
    
    locale VARCHAR(20) DEFAULT 'fa-IR',
    active BOOLEAN DEFAULT true,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(mcp_context_id, fragment_key, locale)
);

CREATE INDEX idx_mcp_fragments_context ON mcp_prompt_fragments(mcp_context_id, fragment_role, active);
CREATE INDEX idx_mcp_fragments_locale ON mcp_prompt_fragments(locale, active);

-- 6. متادیتای کش (Cache Metadata)
CREATE TABLE mcp_cache_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cache_key VARCHAR(255) UNIQUE NOT NULL,
    mcp_context_id UUID REFERENCES mcp_contexts(id) ON DELETE SET NULL,
    
    cached_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    
    hit_rate DECIMAL(5,2)
);

CREATE INDEX idx_cache_metadata_expires ON mcp_cache_metadata(expires_at);
CREATE INDEX idx_cache_metadata_key ON mcp_cache_metadata(cache_key);

-- 7. Trigger برای updated_at
CREATE OR REPLACE FUNCTION update_mcp_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER mcp_contexts_updated_at
BEFORE UPDATE ON mcp_contexts
FOR EACH ROW EXECUTE FUNCTION update_mcp_updated_at();

-- 8. Helper Function: content hash generator
CREATE OR REPLACE FUNCTION calculate_content_hash(content_json JSONB)
RETURNS VARCHAR(128) AS $$
BEGIN
    RETURN encode(digest(content_json::text, 'sha256'), 'hex');
END;
$$ LANGUAGE plpgsql IMMUTABLE;

COMMENT ON FUNCTION calculate_content_hash IS 
'محاسبه SHA-256 hash برای JSONB content (برای cache invalidation)';

-- 9. Comments
COMMENT ON TABLE mcp_contexts IS 'رجیستری اصلی MCPهای قابل استفاده مجدد (Industry, Process, Position, Organization Context, Case)';
COMMENT ON TABLE mcp_context_links IS 'روابط بین MCPها (composition, extension, override)';
COMMENT ON TABLE mcp_context_revisions IS 'تاریخچه تغییرات MCP برای audit trail';
COMMENT ON TABLE business_analysis_mcp_usage IS 'ثبت استفاده از MCP در تحلیل‌های کسب‌وکار (evidence + performance metrics)';
COMMENT ON TABLE mcp_prompt_fragments IS 'قطعات فشرده MCP برای prompt (کاهش توکن LLM)';
COMMENT ON TABLE mcp_cache_metadata IS 'متادیتای کش Redis برای tracking performance';

COMMENT ON COLUMN mcp_contexts.content_hash IS 'SHA-256 hash محتوای MCP برای cache invalidation';
COMMENT ON COLUMN mcp_contexts.reusable IS 'آیا این MCP قابل استفاده مجدد در سازمان‌های دیگر است؟';
COMMENT ON COLUMN mcp_contexts.cacheable IS 'آیا این MCP باید در Redis کش شود؟';
COMMENT ON CONSTRAINT case_mcp_not_reusable ON mcp_contexts IS 'case_temporary همیشه reusable=false';
COMMENT ON CONSTRAINT valid_mcp_scope ON mcp_contexts IS 'scope باید با mcp_type سازگار باشد';
