//#![crate_name = "doc"]

extern crate uuid as uuu;
use chrono::{NaiveDateTime, NaiveDate, Utc};
use self::uuu::Uuid;
use super::schema::*;

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// Defines Database Domain Model
///
//////////////////////////////////////////////////////////////////////////////////////////////


/// PSecurityRealm defines abstraction for security realm that encompasses roles/claims
///
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "rbac_realms"]
pub struct PSecurityRealm {
    pub id: String,
    pub description: String,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PSecurityRealm {
    pub fn new(id: &str, description: &str) -> PSecurityRealm {
        PSecurityRealm{
            id: id.to_string(),
            description: description.to_string(),
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// POrganization represents org that principal users belong to
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Identifiable, Serialize, Deserialize)]
#[table_name = "rbac_organizations"]
pub struct POrganization {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl POrganization {
    pub fn new(id: &str, parent_id: Option<String>, name: &str, url: &str, description: Option<String>) -> POrganization {
        POrganization {
            id: id.to_string(),
            parent_id: parent_id,
            name: name.to_string(),
            url: url.to_string(),
            description: description,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PPrincipal represents user of the organization and belongs to an organization
#[derive(Debug, Clone, PartialEq, Queryable, Identifiable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_principals"]
#[belongs_to(POrganization, foreign_key="organization_id")]
pub struct PPrincipal {
    pub id: String,
    pub organization_id: String,
    pub username: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PPrincipal {
    pub fn new(id: &str, organization_id: &str, username: &str, description: Option<String>) -> PPrincipal {
        PPrincipal {
            id: id.to_string(),
            username: username.to_string(),
            organization_id: organization_id.to_string(),
            description: description,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// An organization can have one or more groups, where each group is associated with Principal or users. A user can be associated
/// with multiple groups and each group can inherit from another group.
#[derive(Debug, Clone, PartialEq, Queryable, Identifiable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_groups"]
#[belongs_to(POrganization, foreign_key="organization_id")]
#[belongs_to(PGroup, foreign_key="parent_id")]
pub struct PGroup {
    pub id: String,
    pub parent_id: Option<String>,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PGroup {
    pub fn new(id: &str, organization_id: &str, name: &str, description: Option<String>, parent_id: Option<String>) -> PGroup {
        PGroup {
            id: id.to_string(),
            parent_id: parent_id,
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            description: description,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PGroupPrincipal defines Many-to-Many association between groups and principals, a group has
/// multiple principals/users and each principal/user can be associated with multiple groups.
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_group_principals"]
#[belongs_to(PGroup, foreign_key="group_id")]
#[belongs_to(PPrincipal, foreign_key="principal_id")]
pub struct PGroupPrincipal {
    pub group_id: String,
    pub principal_id: String,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PGroupPrincipal {
    pub fn new(group_id: &str, principal_id: &str) -> PGroupPrincipal {
        PGroupPrincipal {
            group_id: group_id.to_string(),
            principal_id: principal_id.to_string(),
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PResource represents target object that needs to be secured within a security realm
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_resources"]
#[belongs_to(PSecurityRealm, foreign_key="realm_id")]
pub struct PResource {
    pub id: String,
    pub realm_id: String,
    pub resource_name: String,
    pub description: Option<String>,
    pub allowable_actions: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PResource {
    pub fn new(id: &str, realm_id: &str, resource_name: &str, description: Option<String>, allowable_actions: Option<String>) -> PResource {
        PResource {
            id: id.to_string(),
            realm_id: realm_id.to_string(),
            resource_name: resource_name.to_string(),
            description: description,
            allowable_actions: allowable_actions,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PResourceInstance represents an instance of target object in case number of objects need constraints
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_resource_instances"]
#[belongs_to(PResource, foreign_key="resource_id")]
pub struct PResourceInstance {
    pub id: String,
    pub resource_id: String,
    pub license_policy_id : String,
    pub scope: String,
    pub ref_id: String,
    pub status: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PResourceInstance {
    pub fn new(id: &str, resource_id: &str, license_policy_id: &str, scope: &str, ref_id: &str, status: &str, description: Option<String>) -> PResourceInstance {
        PResourceInstance {
            id: id.to_string(),
            resource_id: resource_id.to_string(),
            license_policy_id: license_policy_id.to_string(),
            scope: scope.to_string(),
            ref_id: ref_id.to_string(),
            status: status.to_string(),
            description: description,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PResourceQuota represents max quota for number of instances of target object
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_resource_quotas"]
#[belongs_to(PResource, foreign_key="resource_id")]
#[belongs_to(PLicensePolicy, foreign_key="license_policy_id")]
pub struct PResourceQuota {
    pub id: String,
    pub resource_id: String,
    pub license_policy_id: String,
    pub scope: String,
    pub max_value: i32,
    pub effective_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PResourceQuota {
    pub fn new(id: &str, resource_id: &str, license_policy_id: &str, scope: &str, max_value: i32, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> PResourceQuota {
        PResourceQuota {
            id: id.to_string(),
            resource_id: resource_id.to_string(),
            license_policy_id: license_policy_id.to_string(),
            scope: scope.to_string(),
            max_value: max_value,
            effective_at: effective_at,
            expired_at: expired_at,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PRole defines abstraction for defining claims/capabilities/permissions to a group of users
#[derive(Debug, Clone, PartialEq, Queryable, Identifiable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_roles"]
#[belongs_to(POrganization, foreign_key="organization_id")]
#[belongs_to(PSecurityRealm, foreign_key="realm_id")]
#[belongs_to(PRole, foreign_key="parent_id")]
pub struct PRole {
    pub id: String,
    pub parent_id: Option<String>,
    pub realm_id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PRole {
    pub fn new(id: &str, realm_id: &str, organization_id: &str, name: &str, description: Option<String>, parent_id: Option<String>) -> PRole {
        PRole {
            id: id.to_string(),
            parent_id: parent_id,
            organization_id: organization_id.to_string(),
            realm_id: realm_id.to_string(),
            name: name.to_string(),
            description: description,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PRoleRoleable defines Many-to-Many association between role and roleable, the roleable can be
/// principal/user or group.
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_role_roleables"]
#[belongs_to(PRole, foreign_key="role_id")]
pub struct PRoleRoleable {
    pub role_id: String,
    pub roleable_id: String,
    pub roleable_type: String,
    pub role_constraints: String,
    pub effective_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PRoleRoleable {
    pub fn new(role_id: &str, roleable_id: &str, roleable_type: &str, role_constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> PRoleRoleable {
        PRoleRoleable {
            role_id: role_id.to_string(),
            roleable_id: roleable_id.to_string(),
            roleable_type: roleable_type.to_string(),
            role_constraints: role_constraints.to_string(),
            effective_at: effective_at,
            expired_at: expired_at,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PClaim defines mapping of target resource that needs protection and action that can be performed
/// on those resources.
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_claims"]
#[belongs_to(PResource, foreign_key="resource_id")]
pub struct PClaim {
    pub id: String,
    pub realm_id: String,
    pub resource_id: String,
    pub action: String,
    pub effect: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PClaim {
    pub fn new(id: &str, realm_id: &str, resource_id: &str, action: &str, effect: &str, description: Option<String>) -> PClaim {
        PClaim {
            id: id.to_string(),
            realm_id: realm_id.to_string(),
            resource_id: resource_id.to_string(),
            action: action.to_string(),
            effect: effect.to_string(),
            description: description,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// ClaimClaimable defines association between Claim and Claimable (Principal/Group/LicensePolicy)
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_claim_claimables"]
#[belongs_to(PClaim, foreign_key="claim_id")]
pub struct PClaimClaimable {
    pub claim_id: String,
    pub claimable_id: String,
    pub claimable_type: String,
    pub scope: String,
    pub claim_constraints: String,
    pub effective_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PClaimClaimable {
    pub fn new(claim_id: &str, claimable_id: &str, claimable_type: &str, scope: &str, claim_constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> PClaimClaimable {
        PClaimClaimable {
            claim_id: claim_id.to_string(),
            claimable_id: claimable_id.to_string(),
            claimable_type: claimable_type.to_string(),
            scope: scope.to_string(),
            claim_constraints: claim_constraints.to_string(),
            effective_at: effective_at,
            expired_at: expired_at,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

/// PAuditRecord stores a log for any action on RBAC system.
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "rbac_audit_records"]
pub struct PAuditRecord {
    pub id: String,
    pub message: String,
    pub action: Option<String>,
    pub context: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
}

impl PAuditRecord {
    pub fn new(message: &str, action: Option<String>, context: Option<String>) -> PAuditRecord {
        PAuditRecord {
            id: Uuid::new_v4().to_hyphenated().to_string(),
            message: message.to_string(),
            action: action,
            context: context,
            created_at: Utc::now().naive_utc(),
            created_by: None,
        }
    }
}

/// PLicensePolicy defines what an organization can access
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]
#[table_name = "rbac_license_policies"]
#[belongs_to(POrganization, foreign_key="organization_id")]
pub struct PLicensePolicy {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub effective_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_by: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl PLicensePolicy {
    pub fn new(id: &str, organization_id: &str, name: &str, description: Option<String>, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> PLicensePolicy {
        PLicensePolicy {
            id: id.to_string(),
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            description: description,
            effective_at: effective_at,
            expired_at: expired_at,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use self::uuu::Uuid;
    use plexrbac::persistence::models::*;

    #[test]
    fn test_create_realm() {
        let r = PSecurityRealm::new("test", "");
        assert_eq!("test", r.id);
    }

    #[test]
    fn test_create_org() {
        let o = POrganization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "test", "url", None);
        assert_eq!("test", o.name);
    }

    #[test]
    fn test_create_principal() {
        let g = PPrincipal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "22", "test", None);
        assert_eq!("test", g.username);
    }

    #[test]
    fn test_create_group() {
        let g = PGroup::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "test", None, None);
        assert_eq!("test", g.name);
    }

    #[test]
    fn test_create_role() {
        let r = PRole::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "test", None, None);
        assert_eq!("test", r.name);
    }

    #[test]
    fn test_create_role_roleable() {
        let r = PRoleRoleable::new("11", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        assert_eq!("11".to_string(), r.role_id);
    }
}
