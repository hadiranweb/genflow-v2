-- Sprint 7: Enterprise Security & Multi-Tenancy  
  
-- ============================================================  
-- 1. Row Level Security (RLS) Policies  
-- ============================================================  
-- فعال‌سازی RLS برای جداول حساس  
ALTER TABLE job_positions ENABLE ROW LEVEL SECURITY;  
ALTER TABLE candidates ENABLE ROW LEVEL SECURITY;  
ALTER TABLE job_matches ENABLE ROW LEVEL SECURITY;  
ALTER TABLE business_analyses ENABLE ROW LEVEL SECURITY;  
  
-- Policy: کاربران فقط داده‌های سازمان خود را ببینند  
CREATE POLICY organization_isolation ON job_positions  
    USING (organization_id = current_setting('app.current_org_id', true)::UUID);  
  
CREATE POLICY organization_isolation ON candidates  
    USING (EXISTS (  
        SELECT 1 FROM position_invites i  
        JOIN job_positions p ON i.position_id = p.id  
        WHERE i.candidate_id = candidates.id  
        AND p.organization_id = current_setting('app.current_org_id', true)::UUID  
    ));  

CREATE POLICY organization_isolation ON business_analyses
    USING (organization_id = current_setting('app.current_org_id', true)::UUID);

CREATE POLICY organization_isolation ON job_matches
    USING (EXISTS (
        SELECT 1 FROM job_positions p
        WHERE p.id = job_matches.position_id
        AND p.organization_id = current_setting('app.current_org_id', true)::UUID
    ));
  
-- ============================================================  
-- 2. API Keys (برای دسترسی برنامه‌ای)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS api_keys (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    organization_id UUID NOT NULL REFERENCES organizations(id),  
    created_by_rep_id UUID NOT NULL REFERENCES business_representatives(id),  
      
    key_hash VARCHAR(255) NOT NULL UNIQUE, -- bcrypt hash  
    key_prefix VARCHAR(20) NOT NULL, -- gf_live_...  
      
    name VARCHAR(100) NOT NULL, -- توضیح کلید  
    scopes JSONB NOT NULL DEFAULT '["read"]', -- ["read", "write", "admin"]  
      
    rate_limit_per_minute INTEGER DEFAULT 60,  
    last_used_at TIMESTAMPTZ,  
    expires_at TIMESTAMPTZ,  
      
    is_active BOOLEAN DEFAULT true,  
    revoked_at TIMESTAMPTZ,  
    revoked_reason TEXT,  
      
    created_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
CREATE INDEX IF NOT EXISTS idx_api_keys_org ON api_keys(organization_id, is_active);  
  
-- ============================================================  
-- 3. Audit Log Detail (برای compliance سازمانی)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS audit_log_details (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    audit_log_id UUID NOT NULL REFERENCES audit_logs(id),  
      
    -- جزئیات تغییرات  
    before_value JSONB,  
    after_value JSONB,  
    change_reason TEXT,  
      
    -- اعتبارسنجی  
    verified_by_user_id UUID REFERENCES business_representatives(id),  
    verified_at TIMESTAMPTZ,  
      
    created_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
-- ============================================================  
-- 4. Data Retention Policy (GDPR compliance)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS data_retention_policies (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id),  
      
    candidate_data_days INTEGER DEFAULT 730, -- 2 سال  
    assessment_raw_data_days INTEGER DEFAULT 365, -- 1 سال  
    match_reports_days INTEGER DEFAULT 2555, -- 7 سال  
      
    auto_delete_enabled BOOLEAN DEFAULT false,  
    last_purged_at TIMESTAMPTZ,  
      
    created_at TIMESTAMPTZ DEFAULT NOW(),  
    updated_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
-- ============================================================  
-- 5. Tenant Configuration (تنظیمات اختصاصی هر سازمان)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS tenant_configs (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id),  
      
    -- ویژگی‌های فعال  
    features_enabled JSONB DEFAULT '["position_generation", "candidate_matching", "basic_reports"]',  
      
    -- محدودیت‌ها  
    max_positions INTEGER DEFAULT 10,  
    max_candidates_per_month INTEGER DEFAULT 100,  
    max_users INTEGER DEFAULT 5,  
      
    -- تنظیمات سفارشی  
    custom_branding JSONB,  
    email_templates JSONB,  
    custom_fields JSONB,  
      
    -- LLM config  
    llm_model_preference VARCHAR(50) DEFAULT 'gpt-3.5-turbo',  
    llm_auto_enhance BOOLEAN DEFAULT true,  
      
    created_at TIMESTAMPTZ DEFAULT NOW(),  
    updated_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
-- ============================================================  
-- 6. Mobile Sessions (برای API موبایل)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS mobile_sessions (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    user_id UUID NOT NULL REFERENCES business_representatives(id),  
      
    device_id VARCHAR(255) NOT NULL,  
    device_type VARCHAR(50) CHECK (device_type IN ('ios', 'android')),  
    push_token VARCHAR(255),  
      
    access_token_hash VARCHAR(255) NOT NULL,  
    refresh_token_hash VARCHAR(255) NOT NULL,  
      
    is_active BOOLEAN DEFAULT true,  
    last_active_at TIMESTAMPTZ DEFAULT NOW(),  
      
    created_at TIMESTAMPTZ DEFAULT NOW(),  
    expires_at TIMESTAMPTZ NOT NULL  
);  
  
CREATE INDEX IF NOT EXISTS idx_mobile_sessions_user ON mobile_sessions(user_id, is_active);  
