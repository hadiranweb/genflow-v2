-- Sprint 4: Dashboard & Analytics Schema

-- ============================================================
-- 1. Activity Logs (برای timeline داشبورد)
-- ============================================================
CREATE TABLE activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    actor_id UUID NOT NULL REFERENCES business_representatives(id),
    
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('position', 'candidate', 'match', 'invite')),
    entity_id UUID NOT NULL,
    
    action VARCHAR(50) NOT NULL CHECK (action IN (
        'position_created', 'position_activated', 'position_filled',
        'candidate_invited', 'candidate_registered', 'assessment_completed',
        'match_calculated', 'candidate_shortlisted', 'candidate_rejected', 'candidate_hired',
        'report_downloaded', 'invite_sent'
    )),
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_activity_org ON activity_logs(organization_id, created_at DESC);
CREATE INDEX idx_activity_entity ON activity_logs(entity_type, entity_id);

-- ============================================================
-- 2. Dashboard Metrics (Cache شده برای پرفورمنس)
-- ============================================================
CREATE TABLE organization_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id),
    
    -- Position Metrics
    total_positions INTEGER DEFAULT 0,
    active_positions INTEGER DEFAULT 0,
    filled_positions INTEGER DEFAULT 0,
    draft_positions INTEGER DEFAULT 0,
    
    -- Candidate Metrics
    total_candidates_invited INTEGER DEFAULT 0,
    total_candidates_completed INTEGER DEFAULT 0,
    candidates_in_pipeline INTEGER DEFAULT 0,
    
    -- Match Metrics
    average_match_score DECIMAL(5,2),
    total_matches_calculated INTEGER DEFAULT 0,
    shortlisted_candidates INTEGER DEFAULT 0,
    
    -- Time Metrics
    average_time_to_hire_days DECIMAL(5,2),
    
    -- Calculated at
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 3. Position Pipeline View (Materialized یا Cache)
-- ============================================================
CREATE TABLE position_pipeline_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id UUID NOT NULL UNIQUE REFERENCES job_positions(id) ON DELETE CASCADE,
    
    invited_count INTEGER DEFAULT 0,
    registered_count INTEGER DEFAULT 0,
    in_progress_count INTEGER DEFAULT 0,
    completed_count INTEGER DEFAULT 0,
    shortlisted_count INTEGER DEFAULT 0,
    hired_count INTEGER DEFAULT 0,
    rejected_count INTEGER DEFAULT 0,
    
    top_match_score DECIMAL(5,2),
    top_match_candidate_id UUID,
    
    last_updated TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 4. Notifications (برای real-time alerts)
-- ============================================================
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id UUID NOT NULL REFERENCES business_representatives(id),
    
    type VARCHAR(50) NOT NULL CHECK (type IN (
        'match_ready', 'candidate_completed', 'position_expiring', 
        'high_match_found', 'system_alert'
    )),
    
    title VARCHAR(255) NOT NULL,
    message TEXT,
    entity_type VARCHAR(50),
    entity_id UUID,
    
    is_read BOOLEAN DEFAULT false,
    read_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notifications_recipient ON notifications(recipient_id, is_read, created_at DESC);

-- ============================================================
-- 5. WebSocket Sessions (برای real-time tracking)
-- ============================================================
CREATE TABLE websocket_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES business_representatives(id),
    session_token VARCHAR(255) NOT NULL UNIQUE,
    connected_at TIMESTAMPTZ DEFAULT NOW(),
    last_ping TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true
);
