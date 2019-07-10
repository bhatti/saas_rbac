//#![crate_name = "doc"]

use plexrbac::common::SecurityContext;
use plexrbac::common::ActionType;

use rocket::outcome::Outcome::*;
use rocket::request::{self, Request, FromRequest};

////////////////////////////////////////////////////////////////////////////////
/// PermissionRequest defines parameters to check for security access
///
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRequest {
    pub action: String,
    pub resource_name: String,
    pub resource_scope: String,
    pub context: SecurityContext,
}

impl<'a, 'r> FromRequest<'a, 'r> for PermissionRequest {
    type Error = (); // experimental exclamation;
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if let Success(ctx) = SecurityContext::from_request(req) {
            let action: String = req.get_query_value("action").and_then(|r| r.ok()).unwrap_or("".into());
            let resource: String = req.get_query_value("resource").and_then(|r| r.ok()).unwrap_or("".into());
            let scope: String = req.get_query_value("scope").and_then(|r| r.ok()).unwrap_or("".into());
            let req = PermissionRequest {
                action: action.clone(),
                resource_name: resource.clone(),
                resource_scope: scope.clone(),
                context: ctx,
            };
            Success(req)
        } else {
            let realm = req.headers().get_one("X-Realm").unwrap_or_else(||"");
            let principal = req.headers().get_one("X-Principal").unwrap_or_else(||"");
            Success(PermissionRequest::with(realm, principal, "", "", ""))
        }
    }
}

impl PermissionRequest {
    /// Creates new instance of security context
    pub fn new(realm_id: &str, principal_id: &str, action: ActionType, resource_name: &str, resource_scope: &str) -> PermissionRequest {
        PermissionRequest {
            action: action.to_string(),
            resource_name: resource_name.to_string(),
            resource_scope: resource_scope.to_string(),
            context: SecurityContext::new(realm_id, principal_id),
        }
    }
    //
    pub fn with(realm_id: &str, principal_id: &str, action: &str, resource_name: &str, resource_scope: &str) -> PermissionRequest {
        PermissionRequest {
            action: action.to_string(),
            resource_name: resource_name.to_string(),
            resource_scope: resource_scope.to_string(),
            context: SecurityContext::new(realm_id, principal_id),
        }
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::security::request::PermissionRequest;
    use plexrbac::common::*;

    #[test]
    fn test_create_new() {
        let req = PermissionRequest::new("1", "2", ActionType::READ, "App", "com.plexobject");
        assert_eq!("READ", req.action);
    }

    #[test]
    fn test_create_with() {
        let req = PermissionRequest::with("1", "2", "READ", "App", "com.plexobject");
        assert_eq!("READ", req.action);
    }
}
