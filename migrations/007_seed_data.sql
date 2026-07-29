-- Sprint 5: Seed Data for Testing & Demo
-- Depends on: 003_mcp_registry.sql, 004_position_generation.sql,
--              005_candidate_matching.sql, 006_dashboard_analytics.sql

-- ============================================================
-- 0. Prerequisite Tables (referenced but not in our migrations)
-- ============================================================

-- Organizations
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    industry_code VARCHAR(50),
    company_size VARCHAR(50),
    stage VARCHAR(50),
    business_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Business Representatives
CREATE TABLE IF NOT EXISTS business_representatives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL UNIQUE,
    role_title VARCHAR(255),
    relation_to_position VARCHAR(50) CHECK (relation_to_position IN (
        'business_owner', 'direct_manager', 'senior_manager', 'advisor', 'external'
    )),
    personality_influence_enabled BOOLEAN DEFAULT false,
    requested_influence_weight DECIMAL(3,2) DEFAULT 0.00
        CHECK (requested_influence_weight BETWEEN 0 AND 1),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Business Analyses (referenced by FK)
CREATE TABLE IF NOT EXISTS business_analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by_rep_id UUID NOT NULL REFERENCES business_representatives(id),
    title VARCHAR(255),
    status VARCHAR(30) DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 1. Seed Organizations
-- ============================================================
INSERT INTO organizations (id, name, industry_code, company_size, stage, business_data) VALUES
('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'Atlas Pour', 'retail', '11-50', 'growth',
 '{"sector": "home_goods", "annual_revenue": "medium", "country": "IR"}'),
('b2c3d4e5-f6a7-8901-bcde-f12345678901', 'TechFlow', 'software', '1-10', 'startup',
 '{"tech_stack": ["rust", "react"], "funding_stage": "seed", "country": "IR"}');

-- ============================================================
-- 2. Seed Business Representatives
-- ============================================================
INSERT INTO business_representatives
(id, organization_id, email, role_title, relation_to_position, personality_influence_enabled, requested_influence_weight)
VALUES
('c3d4e5f6-a7b8-9012-cdef-123456789012', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
 'ceo@atlaspour.com', 'CEO & Founder', 'business_owner', true, 0.20),
('d4e5f6a7-b8c9-0123-defa-234567890123', 'b2c3d4e5-f6a7-8901-bcde-f12345678901',
 'cto@techflow.io', 'CTO', 'direct_manager', true, 0.25);

-- ============================================================
-- 3. Seed Business Analyses
-- ============================================================
INSERT INTO business_analyses (id, organization_id, created_by_rep_id, title, status) VALUES
('aa11bb22-cc33-dd44-ee55-ff6677889900', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
 'c3d4e5f6-a7b8-9012-cdef-123456789012', 'Atlas Pour - Inventory Gap Analysis', 'completed'),
('bb22cc33-dd44-ee55-ff66-778899001122', 'b2c3d4e5-f6a7-8901-bcde-f12345678901',
 'd4e5f6a7-b8c9-0123-defa-234567890123', 'TechFlow - Backend Lead Requirement', 'completed');

-- ============================================================
-- 4. Seed MCP: Industry (Retail)
-- ============================================================
INSERT INTO mcp_contexts
(id, mcp_type, scope, code, title, version, status, content, content_hash,
 reusable, cacheable, deterministic, description, evidence, source_refs,
 source_quality_score, schema_version)
VALUES
('e5f6a7b8-c9d0-1234-efab-345678901234', 'industry', 'global', 'retail',
 'صنعت خرده‌فروشی', '1.0.0', 'active',
 '{
   "industry_code": "retail",
   "common_processes": ["inventory_management", "sales", "customer_service", "procurement"],
   "common_roles": ["store_manager", "sales_associate", "inventory_controller", "cashier"],
   "common_kpis": ["sales_conversion", "inventory_turnover", "customer_satisfaction", "shrinkage_rate"],
   "common_bottlenecks": ["stock_mismatch", "seasonal_demand", "cash_flow_management"],
   "description_fa": "صنعت خرده‌فروشی شامل فروش کالا به مصرف‌کننده نهایی است"
 }',
 '1327b14d46737bb24b553d78258e7c215e19c154c0c55623fe1872605cb93bb7',
 true, true, true,
 'استانداردهای صنعت خرده‌فروشی', '{}', '[]', 0.85, '0.1'),

('f6a7b8c9-d0e1-2345-fabc-456789012345', 'industry', 'global', 'software',
 'صنعت نرم‌افزار', '1.0.0', 'active',
 '{
   "industry_code": "software",
   "common_processes": ["product_development", "qa_testing", "devops", "customer_success"],
   "common_roles": ["backend_developer", "frontend_developer", "product_manager", "qa_engineer"],
   "common_kpis": ["velocity", "bug_escape_rate", "uptime", "nps"],
   "common_bottlenecks": ["technical_debt", "talent_shortage", "scaling_challenges"]
 }',
 '9e50e55101ed423d0680efc2c0f4bb1f00dfae5707d93636a105de6368923fac',
 true, true, true,
 'استانداردهای صنعت نرم‌افزار', '{}', '[]', 0.80, '0.1');

-- ============================================================
-- 5. Seed MCP: Business Process (Inventory Management)
-- ============================================================
INSERT INTO mcp_contexts
(id, mcp_type, scope, code, title, version, status, industry_code, content, content_hash,
 reusable, cacheable, deterministic, description, evidence, source_refs,
 source_quality_score, schema_version)
VALUES
('a7b8c9d0-e1f2-3456-abcd-567890123456', 'business_process', 'industry', 'inventory_management',
 'مدیریت موجودی', '1.0.0', 'active', 'retail',
 '{
   "process_code": "inventory_management",
   "inputs": ["purchase_orders", "sales_data", "returns"],
   "outputs": ["stock_levels", "reorder_alerts", "inventory_valuation"],
   "bottlenecks": ["manual_counting", "system_lag", "supplier_delays"],
   "related_roles": ["warehouse_manager", "inventory_controller", "procurement_officer"],
   "automation_potential": "high"
 }',
 'e45593067704f92d3bb163f243eac20a650f1f18ac86b80a50c4b7ad4e17cc59',
 true, true, true,
 'فرایند مدیریت موجودی در خرده‌فروشی', '{}', '[]', 0.75, '0.1');

-- ============================================================
-- 6. Seed MCP: Standard Position (Warehouse Manager)
-- ============================================================
INSERT INTO mcp_contexts
(id, mcp_type, scope, code, title, version, status, industry_code, content, content_hash,
 reusable, cacheable, deterministic, description, evidence, source_refs,
 source_quality_score, schema_version)
VALUES
('b8c9d0e1-f2a3-4567-bcde-678901234567', 'standard_position', 'industry', 'warehouse_manager',
 'مدیر انبار', '1.0.0', 'active', 'retail',
 '{
   "position_code": "warehouse_manager",
   "title_fa": "مدیر انبار",
   "ksao": {
     "knowledge": ["inventory_systems", "warehouse_layout", "safety_regulations"],
     "skills": ["stock_control", "team_leadership", "reporting", "problem_solving"],
     "abilities": ["attention_to_detail", "physical_stamina", "multitasking"],
     "other": ["trustworthiness", "ownership_mentality"]
   },
   "typical_kpis": ["inventory_accuracy", "order_fulfillment_speed", "safety_incidents"],
   "experience_years": {"min": 2, "ideal": 5},
   "team_size": "3-10"
 }',
 '8f33f1ee41e9dfdd97971c31ce48caff4323a693c8bf96cb37776893004b0d3d',
 true, true, true,
 'پوزیشن استاندارد مدیر انبار در خرده‌فروشی', '{}', '[]', 0.90, '0.1');

-- ============================================================
-- 7. Seed MCP: Standard Position (Backend Lead)
-- ============================================================
INSERT INTO mcp_contexts
(id, mcp_type, scope, code, title, version, status, industry_code, content, content_hash,
 reusable, cacheable, deterministic, description, evidence, source_refs,
 source_quality_score, schema_version)
VALUES
('c9d0e1f2-a3b4-5678-cdef-789012345670', 'standard_position', 'industry', 'backend_lead',
 'تیم‌لید بک‌اند', '1.0.0', 'active', 'software',
 '{
   "position_code": "backend_lead",
   "title_fa": "تیم‌لید بک‌اند",
   "ksao": {
     "knowledge": ["system_design", "databases", "security", "distributed_systems"],
     "skills": ["rust", "code_review", " mentoring", "architecture_decision"],
     "abilities": ["strategic_thinking", "communication", "technical_depth"],
     "other": ["ownership", "growth_mindset"]
   },
   "typical_kpis": ["team_velocity", "system_uptime", "code_quality_score"],
   "experience_years": {"min": 4, "ideal": 7},
   "team_size": "3-8"
 }',
 '6137e30d05cbcdc22ebca542139dcd5e48647fb6ec9763fe06e5a0c11c0d5120',
 true, true, true,
 'پوزیشن استاندارد تیم‌لید بک‌اند در نرم‌افزار', '{}', '[]', 0.85, '0.1');

-- ============================================================
-- 8. Seed Prompt Fragments
-- ============================================================
INSERT INTO mcp_prompt_fragments
(id, mcp_context_id, fragment_key, fragment_role, content, token_estimate, content_hash, locale)
VALUES
('c9d0e1f2-a3b4-5678-cdef-789012345678', 'e5f6a7b8-c9d0-1234-efab-345678901234',
 'retail_summary', 'industry_summary',
 'صنعت خرده‌فروشی نیازمند مدیریت دقیق موجودی و خدمات مشتری است.', 20,
 '8f71a964ee713ea1af88c75c4e0dc58796664d20e93599c3607d19f7f5361759', 'fa-IR'),

('d0e1f2a3-b4c5-6789-defa-890123456789', 'b8c9d0e1-f2a3-4567-bcde-678901234567',
 'wm_requirements', 'position_requirements',
 'مدیر انبار باید در سیستم‌های نرم‌افزاری موجودی مسلط باشد و تیم را رهبری کند.', 25,
 '3ede13d8fa352ea2bcb2a4e3dfb7650ea5bab44792b31461d389c4898913c6ef', 'fa-IR');

-- ============================================================
-- 9. Seed Platform Policies (Fairness & Privacy)
-- ============================================================
INSERT INTO mcp_contexts
(id, mcp_type, scope, code, title, version, status, content, content_hash,
 reusable, cacheable, deterministic, description, evidence, source_refs,
 source_quality_score, schema_version)
VALUES
('e1f2a3b4-c5d6-7890-efab-901234567890', 'platform_policy', 'global', 'fairness_guardrails',
 'قوانین انصاف و بی‌طرفی', '1.0.0', 'active',
 '{
   "rules": [
     "Representative personality cannot reject candidates directly",
     "Culture fit renamed to work style alignment",
     "All recommendations are advisory, not decisions",
     "Hiring decisions require human review and diverse panel"
   ],
   "principles": {
     "fairness": "Every candidate evaluated on same criteria",
     "transparency": "Candidates receive constructive feedback",
     "privacy": "Raw personality scores never shared with employers"
   },
   "description_fa": "قوانین انصاف پلتفرم GenFlow"
 }',
 'e41555e638e8ee20255806b294a79e0d6a094efc72fc42f3f76267f810ba101e',
 true, true, true,
 'قوانین انصاف و بی‌طرفی پلتفرم', '{}', '[]', 1.00, '0.1'),

('f2a3b4c5-d6e7-8901-fabc-012345678901', 'platform_policy', 'global', 'privacy_policy',
 'قوانین حریم خصوصی', '1.0.0', 'active',
 '{
   "rules": [
     "Candidate raw assessment data encrypted at rest",
     "Only behavioral insights shared (not raw scores)",
     "Candidate consent required before sharing results",
     "Data retention: 90 days after last activity",
     "GDPR compliance: right to access, correct, delete"
   ],
   "description_fa": "قوانین حریم خصوصی GenFlow"
 }',
 '7e60f244c60a846e5c1473cd65cc95a458c6578979ce7a3255993ec4aba376f1',
 true, true, true,
 'قوانین حریم خصوصی و GDPR', '{}', '[]', 1.00, '0.1');

-- ============================================================
-- 10. Seed MCP Context Links (composition relationships)
-- ============================================================
INSERT INTO mcp_context_links (parent_mcp_id, child_mcp_id, link_type, weight) VALUES
-- Retail industry → Inventory Management process
('e5f6a7b8-c9d0-1234-efab-345678901234', 'a7b8c9d0-e1f2-3456-abcd-567890123456', 'composes', 0.80),
-- Inventory Management → Warehouse Manager position
('a7b8c9d0-e1f2-3456-abcd-567890123456', 'b8c9d0e1-f2a3-4567-bcde-678901234567', 'uses', 0.90),
-- Software industry → Backend Lead position
('f6a7b8c9-d0e1-2345-fabc-456789012345', 'c9d0e1f2-a3b4-5678-cdef-789012345670', 'composes', 0.85),
-- All positions → fairness policy
('b8c9d0e1-f2a3-4567-bcde-678901234567', 'e1f2a3b4-c5d6-7890-efab-901234567890', 'uses', 1.00),
('c9d0e1f2-a3b4-5678-cdef-789012345670', 'e1f2a3b4-c5d6-7890-efab-901234567890', 'uses', 1.00);

-- ============================================================
-- 11. Seed Job Position (Atlas Pour - Warehouse Manager)
-- ============================================================
INSERT INTO job_positions
(id, organization_id, created_by_rep_id, position_code, title, description,
 generation_method, status, generation_evidence, standards_used, advisory_disclaimer)
VALUES
('p1a2b3c4-d5e6-f789-abcd-ef1234567890', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
 'c3d4e5f6-a7b8-9012-cdef-123456789012',
 'GF-a1b2c3d-0001', 'مدیر انبار - Atlas Pour',
 'مدیر انبار برای مدیریت موجودی و تیم انبار در Atlas Pour',
 'business_analysis', 'active',
 '{"generation_method": "business_analysis", "business_needs_used": ["NEED-W-001"], "standards_used": ["warehouse_manager"]}',
 '[{"mcp_id": "b8c9d0e1-f2a3-4567-bcde-678901234567", "code": "warehouse_manager"}]',
 'این خروجی صرفاً کمک‌تصمیم است و جایگزین ارزیابی تخصصی انسانی نیست.');

-- ============================================================
-- 12. Seed Position Requirements (Warehouse Manager axes)
-- ============================================================
INSERT INTO position_requirements
(id, job_position_id, axis_code, requirement_type, description, importance,
 is_mandatory, source_type, source_ref, rationale)
VALUES
-- Capability axis
('r0010000-0000-0000-0000-000000000001', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890',
 'capability', 'skill', 'مهارت کنترل موجودی', 'critical', true,
 'industry_standard', 'warehouse_manager', 'استاندارد صنعت خرده‌فروشی'),

('r0010000-0000-0000-0000-000000000002', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890',
 'capability', 'skill', 'رهبری تیم', 'important', false,
 'industry_standard', 'warehouse_manager', 'نیاز به مدیریت تیم ۳-۱۰ نفر'),

-- Output KPI axis
('r0010000-0000-0000-0000-000000000003', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890',
 'output_kpi', 'ability', 'دقت موجودی بالای ۹۵%', 'critical', true,
 'industry_standard', 'inventory_accuracy', 'KPI استاندارد انبار'),

-- Work Style axis
('r0010000-0000-0000-0000-000000000004', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890',
 'work_style', 'personality_trait', 'نظم و مسئولیت‌پذیری', 'important', false,
 'representative_context', 'calibrated', 'کالیبره شده توسط نماینده'),

-- Growth Motivation axis
('r0010000-0000-0000-0000-000000000005', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890',
 'growth_motivation', 'experience', 'حداقل ۲ سال تجربه انبار', 'nice_to_have', false,
 'industry_standard', 'warehouse_manager', 'تجربه پیشنهادی');

-- ============================================================
-- 13. Seed Position Graph (5 axes)
-- ============================================================
INSERT INTO position_graphs
(id, job_position_id, graph_version,
 capability_axis, output_kpi_axis, business_gap_axis,
 work_style_axis, growth_motivation_axis, calibration_applied)
VALUES
('g0010000-0000-0000-0000-000000000001', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890', '1.0.0',
 '{"weight": 0.25, "description": "مهارت‌ها و توانایی‌های فنی", "dimensions": [{"code": "cap_stock_control", "description": "کنترل موجودی"}]}',
 '{"weight": 0.25, "description": "شاخص‌های کلیدی عملکرد", "dimensions": []}',
 '{"weight": 0.20, "description": "پر کردن شکاف‌های کسب‌وکار", "dimensions": [{"code": "gap_need_w001", "description": "ضعف داخلی: مدیریت موجودی"}]}',
 '{"weight": 0.20, "description": "سبک کار و همکاری", "dimensions": []}',
 '{"weight": 0.10, "description": "انگیزه و رشد", "dimensions": []}',
 true);

-- ============================================================
-- 14. Seed Candidate (demo candidate for matching)
-- ============================================================
INSERT INTO candidates (id, email, full_name, analysis_status) VALUES
('x1a2b3c4-d5e6-f789-abcd-ef123456789a', 'ali.rezaei@example.com', 'علیرضا رضایی', 'completed'),
('x2b3c4d5-e6f7-a890-bcde-f123456789ab', 'sara.mohammadi@example.com', 'سارا محمدی', 'in_progress');

-- ============================================================
-- 15. Seed Assessment Session (Big Five for demo candidate)
-- ============================================================
INSERT INTO assessment_sessions
(id, subject_candidate_id, method_code, method_version, status, result_summary)
VALUES
('as000001-0000-0000-0000-000000000001', 'x1a2b3c4-d5e6-f789-abcd-ef123456789a',
 'big_five', '1.0.0', 'completed',
 '{"openness": 72, "conscientiousness": 85, "extraversion": 55, "agreeableness": 78, "neuroticism": 30}');

-- ============================================================
-- 16. Seed Job Match (demo match result)
-- ============================================================
INSERT INTO job_matches
(id, position_id, candidate_id,
 capability_match_score, output_kpi_match_score, business_gap_match_score,
 work_style_alignment_score, growth_motivation_match_score,
 composite_match_index, confidence_score, method_version, human_review_required, status)
VALUES
('m0010000-0000-0000-0000-000000000001',
 'p1a2b3c4-d5e6-f789-abcd-ef1234567890', 'x1a2b3c4-d5e6-f789-abcd-ef123456789a',
 82.50, 75.00, 70.00, 85.00, 60.00,
 75.25, 80.00, '1.0.0', true, 'pending_review');

-- ============================================================
-- 17. Seed Match Risk Flags
-- ============================================================
INSERT INTO match_risk_flags (job_match_id, flag_code, severity, description, mitigation_suggestion) VALUES
('m0010000-0000-0000-0000-000000000001', 'development_plan_advised', 'info',
 'کاندیدا در برخی حوزه‌ها نیاز به توسعه دارد',
 'برنامه توسعه فردی در ۳ ماه اول توصیه می‌شود');

-- ============================================================
-- 18. Seed Match Reports (dual: employer + candidate)
-- ============================================================
INSERT INTO match_reports
(id, job_match_id, report_type, title, summary,
 key_findings, strengths, development_areas, recommendations,
 disclaimers, privacy_level)
VALUES
('rp000001-0000-0000-0000-000000000001', 'm0010000-0000-0000-0000-000000000001',
 'for_employer', 'تحلیل تطابق - مدیر انبار Atlas Pour',
 'کاندیدا ۷۵% تطابق با نقش مدیر انبار دارد',
 '["تطابق توانایی: ۸۲%", "تطابق سبک کار: ۸۵%"]',
 '["تطابق بالای توانایی فنی", "سازگاری خوب با سبک کار تیم"]',
 '["growth_motivation: نیاز به توسعه بیشتر"]',
 '["مصاحبه فنی درباره مهارت‌های کلیدی", "بررسی تجربه کاری"]',
 '["این گزارش صرفاً مشاوره‌ای است", "تصمیم نهایی باید با مصاحبه تخصصی"]',
 'detailed_alignment'),

('rp000002-0000-0000-0000-000000000002', 'm0010000-0000-0000-0000-000000000001',
 'for_candidate', 'تحلیل تطابق شما با مدیر انبار - Atlas Pour',
 'شما ۷۵% با این نقش تطابق دارید',
 '["نقاط قوت شما برای این نقش", "مواردی که می‌توانید تقویت کنید"]',
 '["تجربه شما در مدیریت موجودی"]',
 '["می‌توانید در زمینه رشد شغلی بیشتر یاد بگیرید"]',
 '["در مصاحبه روی مهارت‌های فنی تمرکز کنید"]',
 '["این تحلیل بر اساس ارزیابی‌های شماست"]',
 'summary_only');

-- ============================================================
-- 19. Seed Organization Metrics
-- ============================================================
INSERT INTO organization_metrics
(id, organization_id, total_positions, active_positions, filled_positions,
 total_candidates_invited, total_candidates_completed, average_match_score)
VALUES
('om000001-0000-0000-0000-000000000001', 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
 1, 1, 0, 2, 1, 75.25);

-- ============================================================
-- 20. Seed Position Pipeline Stats
-- ============================================================
INSERT INTO position_pipeline_stats
(id, position_id, invited_count, registered_count, completed_count, shortlisted_count, top_match_score)
VALUES
('ps000001-0000-0000-0000-000000000001', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890',
 2, 1, 1, 0, 75.25);

-- ============================================================
-- 21. Seed Notifications
-- ============================================================
INSERT INTO notifications (recipient_id, type, title, message, entity_type, entity_id, is_read) VALUES
('c3d4e5f6-a7b8-9012-cdef-123456789012', 'match_ready',
 'تطابق جدید آماده است',
 'کاندیدای علیرضا رضایی با امتیاز ۷۵% تطابق یافت', 'match',
 'm0010000-0000-0000-0000-000000000001', false);

-- ============================================================
-- 22. Seed Activity Logs
-- ============================================================
INSERT INTO activity_logs (organization_id, actor_id, entity_type, entity_id, action, metadata) VALUES
('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'c3d4e5f6-a7b8-9012-cdef-123456789012',
 'position', 'p1a2b3c4-d5e6-f789-abcd-ef1234567890', 'position_created', '{"title": "مدیر انبار"}'),
('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 'c3d4e5f6-a7b8-9012-cdef-123456789012',
 'match', 'm0010000-0000-0000-0000-000000000001', 'match_calculated', '{"score": 75.25}');

-- ============================================================
-- 23. Seed Consent Events
-- ============================================================
INSERT INTO consent_events (subject_type, subject_id, consent_type, granted, ip_hash) VALUES
('candidate', 'x1a2b3c4-d5e6-f789-abcd-ef123456789a', 'assessment_consent', true, 'sha256_hash_placeholder'),
('candidate', 'x1a2b3c4-d5e6-f789-abcd-ef123456789a', 'report_sharing_consent', true, 'sha256_hash_placeholder');

-- ============================================================
-- END OF SEED DATA
-- ============================================================
