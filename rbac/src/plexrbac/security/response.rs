//////////////////////////////////////////////////////////////////////////////////////////////
///
/// This module defines response object from security check
///

use plexrbac::common::Constants;

/// PermissionResponse
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionResponse {
    Allow,
    Deny
}

impl PermissionResponse {
    pub fn from(value: String) -> PermissionResponse {
        if value == Constants::Allow.to_string() {
            PermissionResponse::Allow
        } else {
            PermissionResponse::Deny
        }
    }
}

