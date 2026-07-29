-- ============================================================
-- Migration 002: Utility & Reference Tables
-- ============================================================

-- ============================================================
-- 1. Industry Reference Codes
-- ============================================================
CREATE TABLE industry_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    title_fa VARCHAR(255) NOT NULL,
    title_en VARCHAR(255),
    description TEXT,
    common_processes JSONB DEFAULT '[]',
    common_roles JSONB DEFAULT '[]',
    common_kpis JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 2. Process Reference Codes
-- ============================================================
CREATE TABLE process_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) UNIQUE NOT NULL,
    title_fa VARCHAR(255) NOT NULL,
    title_en VARCHAR(255),
    industry_code VARCHAR(50) REFERENCES industry_codes(code),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 3. Position Reference Codes
-- ============================================================
CREATE TABLE position_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) UNIQUE NOT NULL,
    title_fa VARCHAR(255) NOT NULL,
    title_en VARCHAR(255),
    industry_code VARCHAR(50) REFERENCES industry_codes(code),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
