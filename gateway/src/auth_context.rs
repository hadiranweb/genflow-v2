//! Request authentication context for tenant-scoped Gateway operations.
//!
//! The Gateway authenticates transport credentials once and handlers receive a
//! domain-relevant tenant identity rather than parsing HTTP headers themselves.

use crate::error_response::ApiError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::{header::AUTHORIZATION, request::Parts};
use genflow_shared_infra::{AccessRole, AppError, AuthClaims, Permission};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TenantAuth(AuthClaims);

impl TenantAuth {
    pub fn organization_id(&self) -> Uuid {
        self.0.org_id
    }

    pub fn user_id(&self) -> Uuid {
        self.0.sub
    }

    /// Verify that a client-supplied organization reference agrees with the
    /// signed tenant claim. This prevents a request body or path parameter from
    /// becoming an authority boundary.
    pub fn require_organization(&self, organization_id: Uuid) -> Result<(), ApiError> {
        if self.organization_id() == organization_id {
            Ok(())
        } else {
            Err(ApiError(AppError::Authorization(
                "Organization does not match the authenticated tenant".to_string(),
            )))
        }
    }

    pub fn require_permission(&self, permission: Permission) -> Result<(), ApiError> {
        let role = AccessRole::from_claim(&self.0.role).map_err(ApiError)?;
        if role.allows(permission) {
            Ok(())
        } else {
            Err(ApiError(AppError::Authorization(format!(
                "Role {role:?} is not allowed to perform {permission:?}"
            ))))
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for TenantAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ApiError(AppError::Auth("Missing bearer token".to_string())))?;
        let token = authorization
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError(AppError::Auth("Invalid bearer token".to_string())))?;

        state
            .jwt_auth
            .validate_token(token)
            .map(Self)
            .map_err(ApiError)
    }
}
