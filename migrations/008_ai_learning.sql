-- Sprint 6: AI Enhancement & Machine Learning Schema  
  
-- ============================================================  
-- 1. LLM Usage Logs (برای monitoring هزینه و کیفیت)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS llm_usage_logs (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    request_id UUID NOT NULL,  
      
    model_name VARCHAR(100) NOT NULL, -- 'gpt-4', 'gpt-3.5-turbo', 'local-llama'  
    prompt_tokens INTEGER NOT NULL,  
    completion_tokens INTEGER NOT NULL,  
    total_tokens INTEGER NOT NULL,  
    estimated_cost_usd DECIMAL(10, 4),  
      
    purpose VARCHAR(50) CHECK (purpose IN (  
        'position_description', 'report_summary', 'interview_questions',   
        'mcp_enrichment', 'candidate_feedback'  
    )),  
      
    latency_ms INTEGER,  
    success BOOLEAN DEFAULT true,  
    error_message TEXT,  
      
    created_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
CREATE INDEX IF NOT EXISTS idx_llm_usage_purpose ON llm_usage_logs(purpose, created_at);  
  
-- ============================================================  
-- 2. Hiring Decisions (برای یادگیری الگوریتم)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS hiring_decisions (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    job_match_id UUID NOT NULL UNIQUE REFERENCES job_matches(id),  
      
    decision_type VARCHAR(20) NOT NULL CHECK (decision_type IN ('hired', 'rejected', 'withdrawn')),  
    decided_by_user_id UUID NOT NULL REFERENCES business_representatives(id),  
    decided_at TIMESTAMPTZ NOT NULL,  
      
    -- دلایل واقعی (برای training)  
    primary_reason VARCHAR(100) CHECK (primary_reason IN (  
        'skill_fit', 'culture_fit', 'experience_level', 'salary_mismatch',  
        'availability', 'interview_performance', 'reference_check', 'other'  
    )),  
      
    notes TEXT,  
    would_reconsider BOOLEAN DEFAULT false,  
      
    -- Outcome tracking (3-6-12 months)  
    performance_rating_3m INTEGER CHECK (performance_rating_3m BETWEEN 1 AND 5),  
    performance_rating_6m INTEGER CHECK (performance_rating_6m BETWEEN 1 AND 5),  
    still_employed BOOLEAN,  
      
    created_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
-- ============================================================  
-- 3. Match Feedback (feedback از طرف کارفرما و کاندیدا)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS match_feedback (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    job_match_id UUID NOT NULL REFERENCES job_matches(id),  
    feedback_from VARCHAR(20) CHECK (feedback_from IN ('employer', 'candidate')),  
      
    -- امتیاز دقت تطابق (1-5)  
    accuracy_rating INTEGER CHECK (accuracy_rating BETWEEN 1 AND 5),  
    -- آیا پیش‌بینی درست بود؟  
    prediction_accurate BOOLEAN,  
      
    -- کدام محور اشتباه پیش‌بینی شد؟  
    mispredicted_axes JSONB DEFAULT '[]',  
      
    comments TEXT,  
    created_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
-- ============================================================  
-- 4. Adaptive Weights History (تاریخچه وزن‌های تطبیقی)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS adaptive_weights_history (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    organization_id UUID NOT NULL REFERENCES organizations(id),  
      
    -- وزن‌های یادگرفته شده برای این سازمان  
    capability_weight DECIMAL(3,2) NOT NULL,  
    output_kpi_weight DECIMAL(3,2) NOT NULL,  
    business_gap_weight DECIMAL(3,2) NOT NULL,  
    work_style_weight DECIMAL(3,2) NOT NULL,  
    growth_motivation_weight DECIMAL(3,2) NOT NULL,  
      
    -- بر اساس چه داده‌هایی؟  
    training_data_count INTEGER NOT NULL,  
    accuracy_on_training DECIMAL(5,2),  
      
    valid_from TIMESTAMPTZ DEFAULT NOW(),  
    valid_until TIMESTAMPTZ,  
      
    created_at TIMESTAMPTZ DEFAULT NOW()  
);  
  
-- ============================================================  
-- 5. Prompt Templates (قالب‌های بهینه شده)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS optimized_prompts (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    purpose VARCHAR(50) NOT NULL CHECK (purpose IN (  
        'position_description', 'employer_report', 'candidate_report', 'interview_guide'  
    )),  
      
    version VARCHAR(20) NOT NULL,  
    template_text TEXT NOT NULL,  
      
    -- متریک‌های کیفیت  
    avg_quality_rating DECIMAL(3,2),  
    usage_count INTEGER DEFAULT 0,  
      
    is_active BOOLEAN DEFAULT false,  
    created_at TIMESTAMPTZ DEFAULT NOW(),  
      
    UNIQUE(purpose, version)  
);  
  
-- ============================================================  
-- 6. Feature Vectors (برای ML)  
-- ============================================================  
CREATE TABLE IF NOT EXISTS candidate_feature_vectors (  
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  
    candidate_id UUID NOT NULL UNIQUE REFERENCES candidates(id),  
      
    -- بردار ویژگی‌های استخراج شده از assessments  
    personality_vector JSONB, -- Big Five normalized  
    skill_vector JSONB, -- encoded skills  
    experience_vector JSONB, -- years, industries  
      
    -- Embedding برای similarity search (در آینده)  
    embedding_vector BYTEA,  
      
    updated_at TIMESTAMPTZ DEFAULT NOW()  
);  
