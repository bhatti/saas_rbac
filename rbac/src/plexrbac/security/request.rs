//#![crate_name = "doc"]
    
use plexrbac::common::SecurityContext;
use plexrbac::common::ActionType;

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
