// GenFlow Database Contracts
// These types mirror the Rust backend domain models for frontend consumption.

export interface Organization {
  id: string;
  name: string;
  industry?: string;
  tenant_id: string;
  created_at: string;
}

export interface Position {
  id: string;
  title: string;
  description: string;
  department?: string;
  experience_required?: number;
  location?: string;
  organization_id: string;
  status: "draft" | "active" | "filled" | "closed";
  created_at: string;
  updated_at: string;
}

export interface Candidate {
  id: string;
  name: string;
  email?: string;
  position_id: string;
  match_score?: number;
  status: "pending" | "shortlisted" | "interviewed" | "rejected" | "hired";
  created_at: string;
}

export interface MCPRecord {
  id: string;
  key: string;
  value: unknown;
  context: Record<string, unknown>;
  created_at: string;
}
