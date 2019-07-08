//#![crate_name = "doc"]

use plexrbac::persistence::models::*;
use plexrbac::common::Constants;
use chrono::{NaiveDateTime, Utc};
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// This module defines common domain model
///
//////////////////////////////////////////////////////////////////////////////////////////////


/// SecurityRealm defines abstraction for security realm that encompasses roles/claims
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityRealm {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SecurityRealm {
    /// Creates instance from persistent realm
    pub fn from(realm: &PSecurityRealm) -> SecurityRealm {
        SecurityRealm::new(realm.id.as_str(), realm.description.clone())
    }

    /// Creates instance of persistent realm
    pub fn to(&self) -> PSecurityRealm {
        PSecurityRealm::new(self.id.as_str(), self.description.clone())
    }

    /// Creates new instance of realm
    pub fn new(id: &str, description: Option<String>) -> SecurityRealm {
        SecurityRealm{
            id: id.to_string(),
            description: description.clone()
        }
    }
}

impl std::fmt::Display for SecurityRealm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}


/// Organization represents org that principal users belong to
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    #[serde(skip_deserializing)]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub groups: HashMap<String, Group>,
    #[serde(skip_serializing, skip_deserializing)]
    pub roles: HashMap<String, Role>,
    #[serde(skip_serializing, skip_deserializing)]
    pub claims: Vec<ClaimClaimable>,
    #[serde(skip_serializing, skip_deserializing)]
    pub resources: Vec<Resource>,
    #[serde(skip_serializing, skip_deserializing)]
    pub license_policy: Option<LicensePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(skip_deserializing)]
    pub created_by: Option<String>,
    #[serde(skip_deserializing)]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(skip_deserializing)]
    pub updated_by: Option<String>,
    #[serde(skip_deserializing)]
    pub updated_at: Option<NaiveDateTime>,
}

impl std::fmt::Display for Organization {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buf = String::from("");
        for (_, group) in &self.groups {
            buf.push_str(format!("\tgroup: {}\n", group).as_str());
        }
        for (_, role) in &self.roles {
            buf.push_str(format!("{}", role).as_str());
        }
        for claim in &self.claims {
            buf.push_str(format!("{}", claim).as_str());
        }
        write!(f, "org: {}\n{}", self.name, buf)
    }
}

impl Organization {
    /// Creates instance from persistent organization
    pub fn from(org: &POrganization) -> Organization {
        Organization {
            id: org.id.clone(),
            parent_id: org.parent_id.clone(),
            name: org.name.clone(),
            url: org.url.clone(),
            description: org.description.clone(),
            groups: HashMap::new(),
            roles: HashMap::new(),
            claims: vec![],
            resources: vec![],
            license_policy: None,
            created_at: org.created_at.clone(),
            created_by: org.created_by.clone(),
            updated_at:org.updated_at.clone(), 
            updated_by: org.updated_by.clone(),
        }
    }

    /// Creates instance of persistent organization
    pub fn to(&self) -> POrganization {
        POrganization::new(self.id.as_str(), self.parent_id.clone(), self.name.as_str(), self.url.as_str(), self.description.clone())
    }

    pub fn new(id: &str, parent_id: Option<String>, name: &str, url: &str, description: Option<String>) -> Organization {
        Organization {
            id: id.to_string(),
            parent_id: parent_id,
            name: name.to_string(),
            url: url.to_string(),
            description: description,
            groups: HashMap::new(),
            roles: HashMap::new(),
            claims: vec![],
            resources: vec![],
            license_policy: None,
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

/// Principal represents user of the organization and belongs to an organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Principal {
    #[serde(skip_deserializing)]
    pub id: String,
    pub organization_id: String,
    pub username: String,
    pub description: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub groups: HashMap<String, Group>,
    #[serde(skip_serializing, skip_deserializing)]
    pub roles: HashMap<String, Role>,
    #[serde(skip_serializing, skip_deserializing)]
    pub claims: Vec<ClaimClaimable>,
    #[serde(skip_serializing, skip_deserializing)]
    pub resources: Vec<Resource>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buf = String::from("");
        for (_, group) in &self.groups {
            buf.push_str(format!("\tgroup: {}\n", group).as_str());
        }
        for (_, role) in &self.roles {
            buf.push_str(format!("{}", role).as_str());
        }
        for claim in &self.claims {
            buf.push_str(format!("{}", claim).as_str());
        }
        for resource in &self.resources {
            buf.push_str(format!("\n\t\t{}", resource).as_str());
        }
        write!(f, "user: {}\n{}", self.username, buf)
    }
}

impl Principal {
    /// Creates instance from persistent principal
    pub fn from(principal: &PPrincipal) -> Principal {
        Principal {
            id: principal.id.clone(),
            username: principal.username.clone(),
            organization_id: principal.organization_id.clone(),
            description: principal.description.clone(),
            groups: HashMap::new(),
            roles: HashMap::new(),
            claims: vec![],
            resources: vec![],
            created_at: principal.created_at.clone(),
            created_by: principal.created_by.clone(),
            updated_at: principal.updated_at.clone(),
            updated_by: principal.updated_by.clone(),
        }
    }

    /// Creates instance of persistent principal
    pub fn to(&self) -> PPrincipal {
        PPrincipal ::new(self.id.as_str(), self.organization_id.as_str(), self.username.as_str(), self.description.clone())
    }

    pub fn new(id: &str, organization_id: &str, username: &str, description: Option<String>) -> Principal {
        Principal {
            id: id.to_string(),
            username: username.to_string(),
            organization_id: organization_id.to_string(),
            description: description,
            groups: HashMap::new(),
            roles: HashMap::new(),
            claims: vec![],
            resources: vec![],
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }

}

/// An organization can have one or more groups, where each group is associated with Principal or users. A user can be associated
/// with multiple groups and each group can inherit from another group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    #[serde(skip_deserializing)]
    pub id: String,
    pub parent_id: Option<String>,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub roles: HashMap<String, Role>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buf = String::from("");
        for (_, role) in &self.roles {
            buf.push_str(format!("{}", role).as_str());
        }
        write!(f, "group: {}\n{}", self.name, buf)
    }
}

impl Group {
    /// Creates instance from persistent group
    pub fn from(group: &PGroup) -> Group {
        Group {
            id: group.id.clone(),
            parent_id: group.parent_id.clone(),
            organization_id: group.organization_id.clone(),
            name: group.name.clone(),
            description: group.description.clone(),
            roles: HashMap::new(),
            created_at: group.created_at.clone(),
            created_by: group.created_by.clone(),
            updated_at: group.updated_at.clone(),
            updated_by: group.updated_by.clone(),
        }
    }

    /// Creates instance of persistent group
    pub fn to(&self) -> PGroup {
        PGroup::new(self.id.as_str(), self.organization_id.as_str(), self.name.as_str(), self.description.clone(), self.parent_id.clone())
    }

    pub fn new(id: &str, organization_id: &str, name: &str, description: Option<String>, parent_id: Option<String>) -> Group {
        Group {
            id: id.to_string(),
            parent_id: parent_id,
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            description: description,
            roles: HashMap::new(),
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

/// Resource represents target object that needs to be secured within a security realm
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    #[serde(skip_deserializing)]
    pub id: String,
    pub realm_id: String,
    pub resource_name: String,
    pub description: Option<String>,
    pub allowable_actions: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub instances: HashMap<String, ResourceInstance>,
    #[serde(skip_serializing, skip_deserializing)]
    pub quotas: HashMap<String, ResourceQuota>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "resource: {}", self.resource_name)
    }
}

impl Resource {
    /// Creates instance from persistent resource
    pub fn from(resource: &PResource) -> Resource {
        Resource {
            id: resource.id.clone(),
            realm_id: resource.realm_id.clone(),
            resource_name: resource.resource_name.clone(),
            description: resource.description.clone(),
            allowable_actions: resource.allowable_actions.clone(),
            instances: HashMap::new(),
            quotas: HashMap::new(),
            created_at: resource.created_at.clone(),
            created_by: resource.created_by.clone(),
            updated_at: resource.updated_at.clone(),
            updated_by: resource.updated_by.clone()
        }
    }

    /// Creates instance of persistent resource
    pub fn to(&self) -> PResource {
        PResource::new(self.id.as_str(), self.realm_id.as_str(), self.resource_name.as_str(), self.description.clone(), self.allowable_actions.clone())
    }

    pub fn new(id: &str, realm_id: &str, resource_name: &str, description: Option<String>, allowable_actions: Option<String>) -> Resource {
        Resource {
            id: id.to_string(),
            realm_id: realm_id.to_string(),
            resource_name: resource_name.to_string(),
            description: description,
            allowable_actions: allowable_actions,
            instances: HashMap::new(),
            quotas: HashMap::new(),
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

/// ResourceInstance represents an instance of target object in case number of objects need constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceInstance {
    #[serde(skip_deserializing)]
    pub id: String,
    #[serde(skip_deserializing)]
    pub resource_id: String,
    #[serde(skip_deserializing)]
    pub license_policy_id: String,
    pub scope: String,
    pub ref_id: String,
    pub status: String,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl ResourceInstance {
    /// Creates instance from persistent resource instance
    pub fn from(instance: &PResourceInstance) -> ResourceInstance {
        ResourceInstance {
            id: instance.id.clone(),
            resource_id: instance.resource_id.clone(),
            scope: instance.scope.clone(),
            license_policy_id: instance.license_policy_id.clone(),
            ref_id: instance.ref_id.clone(),
            status: instance.status.clone(),
            description: instance.description.clone(),
            created_at: instance.created_at.clone(),
            created_by: instance.created_by.clone(),
            updated_at: instance.updated_at.clone(),
            updated_by: instance.updated_by.clone()
        }
    }

    /// Creates instance of persistent resource instance
    pub fn to(&self) -> PResourceInstance {
        PResourceInstance::new(self.id.as_str(), self.resource_id.as_str(), self.license_policy_id.as_str(), self.scope.as_str(), self.ref_id.as_str(), self.status.as_str(), self.description.clone())
    }

    pub fn new(id: &str, resource_id: &str, license_policy_id: &str, scope: &str, ref_id: &str, status: &str, description: Option<String>) -> ResourceInstance {
        ResourceInstance {
            id: id.to_string(),
            resource_id: resource_id.to_string(),
            license_policy_id: license_policy_id.to_string(),
            scope: scope.to_string(),
            ref_id: ref_id.to_string(),
            status: status.to_string(),
            description: description,
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

/// ResourceQuota represents max quota for number of instances of target object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceQuota {
    #[serde(skip_deserializing)]
    pub id: String,
    #[serde(skip_deserializing)]
    pub resource_id: String,
    pub scope: String,
    #[serde(skip_deserializing)]
    pub license_policy_id: String,
    pub max_value: i32,
    pub effective_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl ResourceQuota {
    /// Creates quota from persistent resource quota
    pub fn from(quota: &PResourceQuota) -> ResourceQuota {
        ResourceQuota {
            id: quota.id.clone(),
            resource_id: quota.resource_id.clone(),
            license_policy_id: quota.license_policy_id.clone(),
            scope: quota.scope.clone(),
            max_value: quota.max_value.clone(),
            effective_at: quota.effective_at.clone(),
            expired_at: quota.expired_at.clone(),
            created_at: quota.created_at.clone(),
            created_by: quota.created_by.clone(),
            updated_at: quota.updated_at.clone(),
            updated_by: quota.updated_by.clone()
        }
    }

    /// Creates quota of persistent resource quota
    pub fn to(&self) -> PResourceQuota {
        PResourceQuota::new(self.id.as_str(), self.resource_id.as_str(), self.license_policy_id.as_str(), self.scope.as_str(), self.max_value, self.effective_at.clone(), self.expired_at.clone())
    }

    pub fn new(id: &str, resource_id: &str, license_policy_id: &str, scope: &str, max_value: i32, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> ResourceQuota {
        ResourceQuota {
            id: id.to_string(),
            resource_id: resource_id.to_string(),
            license_policy_id: license_policy_id.to_string(),
            scope: scope.to_string(),
            max_value: max_value,
            effective_at: effective_at,
            expired_at: expired_at,
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

/// RoleRoleable defines mapping of role and roleable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoleRoleable {
    Group(Role, String, String), // id, constraints
    Principal(Role, String, String), // id, constraints
}

/// Role defines abstraction for defining claims/capabilities/permissions to a group of users
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    #[serde(skip_deserializing)]
    pub id: String,
    pub parent_id: Option<String>,
    pub realm_id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub claims: Vec<ClaimClaimable>,    // All claims mapped to role
    pub constraints: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buf = String::from("");
        for claim in &self.claims {
            buf.push_str(format!("{}", claim).as_str());
        }
        write!(f, "\trole: {}\n{}", self.name, buf)
    }
}

impl Role {
    /// Creates instance from persistent role
    pub fn from(role: &PRole) -> Role {
        Role {
            id: role.id.clone(),
            parent_id: role.parent_id.clone(),
            organization_id: role.organization_id.clone(),
            realm_id: role.realm_id.clone(),
            name: role.name.clone(),
            description: role.description.clone(),
            claims: vec![],
            constraints: None,
            created_at: role.created_at.clone(),
            created_by: role.created_by.clone(),
            updated_at: role.updated_at.clone(),
            updated_by: role.updated_by.clone(),
        }
    }

    /// Creates instance of persistent role
    pub fn to(&self) -> PRole {
        PRole::new(self.id.as_str(), self.realm_id.as_str(), self.organization_id.as_str(), self.name.as_str(), self.description.clone(), self.parent_id.clone())
    }

    pub fn new(id: &str, realm_id: &str, organization_id: &str, name: &str, description: Option<String>, parent_id: Option<String>) -> Role {
        Role {
            id: id.to_string(),
            parent_id: parent_id,
            organization_id: organization_id.to_string(),
            realm_id: realm_id.to_string(),
            name: name.to_string(),
            description: description,
            claims: vec![],
            constraints: None,
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

/// ClaimClaimable defines mapping of claim and claimable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaimClaimable {
    Realm(Claim, String), // realm
    LicensePolicy(Claim, String, String, String), // realm, scope, constraints
    Role(Claim, String, String, String, String), // realm, role-id, scope, constraints
    Principal(Claim, String, String, String, String), // realm, principal-id, scope, constraints
}

impl std::fmt::Display for ClaimClaimable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClaimClaimable::Realm(claim, realm) => write!(f, "\n\trealm-claim: {}\t\trealm: {}", claim, realm),
            ClaimClaimable::LicensePolicy(claim, _realm, scope, constraints) => write!(f, "\n\tlicense-claim: {}\t\tscope: {}, constraints: {}", claim, scope, constraints),
            ClaimClaimable::Role(claim, _, _realm, scope, constraints) => write!(f, "\n\trole-claim: {}\t\tscope: {}, constraints: {}", claim, scope, constraints),
            ClaimClaimable::Principal(claim, _, _realm, scope, constraints) => write!(f, "\n\tprincipal-claim: {}\t\tscope: {}, constraints: {}", claim, scope, constraints),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimResource {
    pub claim: Claim,
    pub scope: String,
    pub constraints: String,
    pub resource: Resource,
}

impl ClaimResource {
    pub fn new(claim: Claim, scope: String, constraints: String, resource: Resource) -> ClaimResource {
        ClaimResource { claim: claim, scope: scope, constraints: constraints, resource: resource}
    }
}

/// Claim defines mapping of target resource that needs protection and action that can be performed
/// on those resources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    #[serde(skip_deserializing)]
    pub id: String,
    pub realm_id: String,
    #[serde(skip_deserializing)]
    pub resource_id: String,
    pub action: String,
    pub effect: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl std::fmt::Display for Claim {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "claim: {} - {}\n", self.action, self.resource_id)
    }
}

impl Claim {
    /// Creates instance from persistent claim
    pub fn from(claim: &PClaim) -> Claim {
        Claim {
            id: claim.id.clone(),
            realm_id: claim.realm_id.clone(),
            resource_id: claim.resource_id.clone(),
            action: claim.action.clone(),
            effect: claim.effect.clone(),
            description: claim.description.clone(),
            created_at: claim.created_at.clone(),
            created_by: claim.created_by.clone(),
            updated_at: claim.updated_at.clone(),
            updated_by: claim.updated_by.clone(),
        }
    }

    /// Creates instance of persistent claim
    pub fn to(&self) -> PClaim {
        PClaim::new(self.id.as_str(), self.realm_id.as_str(), self.resource_id.as_str(), self.action.as_str(), self.effect().as_str(), self.description.clone())
    }

    pub fn new(id: &str, realm_id: &str, resource_id: &str, action: &str, effect: &str, description: Option<String>) -> Claim {
        Claim {
            id: id.to_string(),
            realm_id: realm_id.to_string(),
            resource_id: resource_id.to_string(),
            action: action.to_string(),
            effect: Some(effect.to_string()),
            description: description,
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }

    pub fn effect(&self) -> String {
        if let Some(effect) = self.effect.clone() {
            if effect.len() > 0 {
                return effect;
            }
        }
        Constants::Allow.to_string()
    }
}

/// LicensePolicy defines what an organization can access
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LicensePolicy {
    #[serde(skip_deserializing)]
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub effective_at: NaiveDateTime,
    pub expired_at: NaiveDateTime,
    #[serde(skip_serializing, skip_deserializing)]
    pub claims: Vec<ClaimClaimable>,    // All claims mapped to organization via license policy
    pub created_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

impl LicensePolicy {
    /// Creates instance from persistent license-policy
    pub fn from(policy: &PLicensePolicy) -> LicensePolicy {
        LicensePolicy {
            id: policy.id.clone(),
            organization_id: policy.organization_id.clone(),
            name: policy.name.clone(),
            description: policy.description.clone(),
            effective_at: policy.effective_at.clone(),
            expired_at: policy.expired_at.clone(),
            claims: vec![],
            created_at: policy.created_at.clone(),
            created_by: policy.created_by.clone(),
            updated_at: policy.updated_at.clone(),
            updated_by: policy.updated_by.clone(),
        }
    }

    /// Creates instance of persistent license-policy
    pub fn to(&self) -> PLicensePolicy {
        PLicensePolicy::new(self.id.as_str(), self.organization_id.as_str(), self.name.as_str(), self.description.clone(), self.effective_at.clone(), self.expired_at.clone())
    }

    pub fn new(id: &str, organization_id: &str, name: &str, description: Option<String>, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> LicensePolicy {
        LicensePolicy {
            id: id.to_string(),
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            description: description,
            effective_at: effective_at,
            expired_at: expired_at,
            claims: vec![],
            created_at: Some(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: None
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use self::uuu::Uuid;
    use plexrbac::domain::models::*;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_create_realm() {
        let r = SecurityRealm::new("test", None);
        assert_eq!("test", r.id);
    }

    #[test]
    fn test_create_org() {
        let o = Organization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "test", "url", None);
        assert_eq!("test", o.name);
    }

    #[test]
    fn test_create_principal() {
        let g = Principal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "22", "test", None);
        assert_eq!("test", g.username);
    }

    #[test]
    fn test_create_group() {
        let g = Group::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "test", None, None);
        assert_eq!("test", g.name);
    }

    #[test]
    fn test_create_role() {
        let r = Role::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "test", None, None);
        assert_eq!("test", r.name);
    }

    #[test]
    fn test_create_resource() {
        let r = Resource::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "app", None, None);
        assert_eq!("app", r.resource_name);
    }

    #[test]
    fn test_create_resource_instance() {
        let r = ResourceInstance::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", "ref", "INFLIGHT", None);
        assert_eq!("22", r.license_policy_id);
    }

    #[test]
    fn test_create_resource_quota() {
        let r = ResourceQuota::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        assert_eq!("22", r.license_policy_id);
    }

    #[test]
    fn test_create_claim() {
        let r = Claim::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "action", "allow", None);
        assert_eq!("action", r.action);
    }

    #[test]
    fn test_create_license_policy() {
        let license_policy = PLicensePolicy::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "mylicense_policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        assert_eq!("mylicense_policy", license_policy.name);
    }
}
