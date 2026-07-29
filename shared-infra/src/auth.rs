//! JWT Authentication — Real implementation (no placeholder Uuid::new_v4())

use crate::config::JwtConfig;
use crate::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims — authentication payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: Uuid,    // user_id
    pub org_id: Uuid, // organization_id
    pub role: String, // role (admin, analyst, representative)
    pub iss: String,  // issuer
    pub exp: i64,     // expiration timestamp
    pub iat: i64,     // issued at timestamp
}

/// Roles accepted by the current Gateway contract.
///
/// Parsing roles at the boundary prevents arbitrary JWT strings from becoming
/// implicit permissions in individual handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRole {
    Admin,
    Analyst,
    Representative,
}

impl AccessRole {
    pub fn from_claim(role: &str) -> Result<Self, AppError> {
        match role {
            "admin" => Ok(Self::Admin),
            "analyst" => Ok(Self::Analyst),
            "representative" => Ok(Self::Representative),
            _ => Err(AppError::Authorization(format!("Unsupported role: {role}"))),
        }
    }

    pub fn allows(self, permission: Permission) -> bool {
        match self {
            Self::Admin => true,
            Self::Analyst => matches!(
                permission,
                Permission::GeneratePosition
                    | Permission::ReadPosition
                    | Permission::ResolveMcp
                    | Permission::ReadMcp
                    | Permission::ReadDashboard
                    | Permission::CalculateMatch
                    | Permission::GenerateReport
            ),
            Self::Representative => matches!(
                permission,
                Permission::ReadPosition
                    | Permission::ReadMcp
                    | Permission::ReadDashboard
                    | Permission::CalculateMatch
                    | Permission::CreateInvitation
                    | Permission::GenerateReport
                    | Permission::RecordDecision
            ),
        }
    }
}

/// Commands exposed by the current Gateway. Permissions model intent rather
/// than HTTP routes, so the policy remains stable as API transport evolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    GeneratePosition,
    ReadPosition,
    ResolveMcp,
    ReadMcp,
    ReadDashboard,
    CalculateMatch,
    CreateInvitation,
    GenerateReport,
    RecordDecision,
}

/// JWT authentication handler
pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: JwtConfig,
}

impl JwtAuth {
    pub fn new(config: JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            config,
        }
    }

    /// Generate a JWT token for a user
    pub fn generate_token(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        role: &str,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = AuthClaims {
            sub: user_id,
            org_id,
            role: role.to_string(),
            iss: self.config.issuer.clone(),
            exp: (now + Duration::hours(self.config.expiration_hours as i64)).timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Auth(format!("Token generation failed: {e}")))
    }

    /// Validate a JWT token and return claims
    pub fn validate_token(&self, token: &str) -> Result<AuthClaims, AppError> {
        let mut validation = Validation::default();
        validation.set_issuer(std::slice::from_ref(&self.config.issuer));

        decode(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Auth(format!("Token validation failed: {e}")))
    }
}

#[cfg(test)]
mod authorization_tests {
    use super::{AccessRole, Permission};

    #[test]
    fn analyst_can_generate_positions_but_cannot_record_hiring_decisions() {
        assert!(AccessRole::Analyst.allows(Permission::GeneratePosition));
        assert!(!AccessRole::Analyst.allows(Permission::RecordDecision));
    }

    #[test]
    fn representative_can_run_candidate_workflow_but_not_generate_positions() {
        assert!(AccessRole::Representative.allows(Permission::CreateInvitation));
        assert!(AccessRole::Representative.allows(Permission::RecordDecision));
        assert!(!AccessRole::Representative.allows(Permission::GeneratePosition));
    }

    #[test]
    fn unknown_roles_are_rejected_explicitly() {
        assert!(AccessRole::from_claim("candidate").is_err());
    }
}
