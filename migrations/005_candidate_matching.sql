-- Sprint 3: Candidate Invitation & Matching Engine

-- ============================================================
-- 1. Candidates (مستقل از دعوت)
-- ============================================================
CREATE TABLE candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID, -- بعد از ثبت‌نام کامل
    
    email VARCHAR(255),
    phone VARCHAR(50),
    full_name VARCHAR(255),
    
    -- Privacy: داده‌های حساس فقط در assessment_sessions
    analysis_status VARCHAR(30) DEFAULT 'pending'
        CHECK (analysis_status IN ('pending', 'invited', 'registered', 'in_progress', 'completed')),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT candidate_has_identity CHECK (
        user_id IS NOT NULL OR email IS NOT NULL OR phone IS NOT NULL
    )
);

-- ============================================================
-- 2. Position Invites (دعوت‌نامه)
-- ============================================================
CREATE TABLE position_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id UUID NOT NULL REFERENCES job_positions(id) ON DELETE CASCADE,
    invited_by_rep_id UUID NOT NULL REFERENCES business_representatives(id),
    candidate_id UUID REFERENCES candidates(id), -- nullable تا قبل از ثبت‌نام
    
    invite_code VARCHAR(100) UNIQUE NOT NULL, -- GF-XXXX-XXXX
    email VARCHAR(255),
    phone VARCHAR(50),
    
    status VARCHAR(30) DEFAULT 'created'
        CHECK (status IN ('created', 'sent', 'viewed', 'accepted', 'expired', 'revoked')),
    
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    sent_via VARCHAR(20) CHECK (sent_via IN ('email', 'sms', 'link')),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_invites_code ON position_invites(invite_code);
CREATE INDEX idx_invites_position ON position_invites(position_id, status);

-- ============================================================
-- 3. Assessment Sessions (عمومی برای Big Five, RIASEC, etc.)
-- ============================================================
CREATE TABLE assessment_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- دقیقاً یک subject
    subject_business_rep_id UUID REFERENCES business_representatives(id) ON DELETE CASCADE,
    subject_candidate_id UUID REFERENCES candidates(id) ON DELETE CASCADE,
    
    method_code VARCHAR(50) NOT NULL, -- 'big_five', 'riasec', 'values', etc.
    method_version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    
    status VARCHAR(30) NOT NULL DEFAULT 'started'
        CHECK (status IN ('started', 'completed', 'expired', 'cancelled')),
    
    result_summary JSONB DEFAULT '{}', -- نمرات نهایی (مثلاً {openness: 75, ...})
    raw_responses JSONB DEFAULT '{}', -- پاسخ‌های خام (encrypted در production)
    
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT exactly_one_subject CHECK (
        (subject_business_rep_id IS NOT NULL AND subject_candidate_id IS NULL) OR
        (subject_business_rep_id IS NULL AND subject_candidate_id IS NOT NULL)
    )
);

CREATE INDEX idx_assessments_candidate ON assessment_sessions(subject_candidate_id, method_code);

-- ============================================================
-- 4. Job Matches (تطابق‌یابی ۵ محوره)
-- ============================================================
CREATE TABLE job_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id UUID NOT NULL REFERENCES job_positions(id),
    candidate_id UUID NOT NULL REFERENCES candidates(id),
    
    -- ۵ محور تطابق (0-100)
    capability_match_score DECIMAL(5,2) CHECK (capability_match_score BETWEEN 0 AND 100),
    output_kpi_match_score DECIMAL(5,2),
    business_gap_match_score DECIMAL(5,2),
    work_style_alignment_score DECIMAL(5,2), -- نه culture_fit
    growth_motivation_match_score DECIMAL(5,2),
    
    -- Score ترکیبی (وزن‌دار)
    composite_match_index DECIMAL(5,2) CHECK (composite_match_index BETWEEN 0 AND 100),
    
    -- Confidence & Advisory
    confidence_score DECIMAL(5,2),
    method_version VARCHAR(20) DEFAULT '1.0.0',
    advisory_disclaimer TEXT DEFAULT 'این ارزیابی صرفاً مشاوره‌ای است.',
    human_review_required BOOLEAN DEFAULT true,
    
    -- وضعیت (خنثی)
    status VARCHAR(30) DEFAULT 'pending_review'
        CHECK (status IN ('pending_review', 'under_review', 'shortlisted', 'not_selected', 'selected', 'withdrawn')),
    
    -- Audit تصمیم
    decision_made_by_user_id UUID,
    decision_made_at TIMESTAMPTZ,
    decision_note TEXT,
    
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    
    UNIQUE(position_id, candidate_id)
);

CREATE INDEX idx_matches_position ON job_matches(position_id, composite_match_index DESC);
CREATE INDEX idx_matches_candidate ON job_matches(candidate_id);

-- ============================================================
-- 5. Match Risk Flags (غیرانگ‌زننده)
-- ============================================================
CREATE TABLE match_risk_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_match_id UUID NOT NULL REFERENCES job_matches(id) ON DELETE CASCADE,
    
    flag_code VARCHAR(50) NOT NULL CHECK (flag_code IN (
        'stress_support_needed',
        'role_transition_support_needed',
        'collaboration_style_gap',
        'expectation_clarification_recommended',
        'development_plan_advised'
    )),
    
    severity VARCHAR(20) CHECK (severity IN ('info', 'attention', 'action_required')),
    description TEXT NOT NULL,
    mitigation_suggestion TEXT NOT NULL,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 6. Match Reports (دوطرفه - Privacy Aware)
-- ============================================================
CREATE TABLE match_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_match_id UUID NOT NULL REFERENCES job_matches(id),
    report_type VARCHAR(20) CHECK (report_type IN ('for_employer', 'for_candidate')),
    
    title VARCHAR(255) NOT NULL,
    summary TEXT,
    
    -- Content ساختاریافته
    key_findings JSONB DEFAULT '[]',
    strengths JSONB DEFAULT '[]', -- نقاط قوت (نه raw scores)
    development_areas JSONB DEFAULT '[]', -- Areas for growth
    recommendations JSONB DEFAULT '[]',
    
    -- Compliance
    disclaimers JSONB NOT NULL DEFAULT '[]',
    privacy_level VARCHAR(20) CHECK (privacy_level IN ('summary_only', 'detailed_alignment', 'full_profile')),
    
    pdf_url VARCHAR(500),
    generated_at TIMESTAMPTZ DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 7. Consent & Audit (GDPR)
-- ============================================================
CREATE TABLE consent_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_type VARCHAR(30) NOT NULL CHECK (subject_type IN ('business_rep', 'candidate')),
    subject_id UUID NOT NULL,
    consent_type VARCHAR(50) NOT NULL,
    granted BOOLEAN NOT NULL,
    ip_hash VARCHAR(64), -- hash شده
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID NOT NULL,
    target_user_id UUID,
    action VARCHAR(50) NOT NULL, -- 'view_match', 'download_pdf'
    resource_type VARCHAR(50),
    resource_id UUID,
    ip_hash VARCHAR(64),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
