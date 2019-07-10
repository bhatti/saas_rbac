//#![crate_name = "doc"]

use plexrbac::domain::models::*;
use plexrbac::common::Constants;
use plexrbac::common::Status;
use plexrbac::common::SecurityContext;
use chrono::{NaiveDate, Utc};
use log::{info, warn};
use std::collections::HashMap;
use plexrbac::common::RbacError;

//////////////////////////////////////////////////////////////////////////////////////////////
/// PersistenceManager defines high-level methods for accessing rbac entities
///
/// TODO validate time range for effective/expired within license
pub struct PersistenceManager<'a> {
    pub realm_repository: super::realm_repository::SecurityRealmRepository<'a>,
    pub org_repository: super::org_repository::OrganizationRepository<'a>,
    pub group_repository: super::group_repository::GroupRepository<'a>,
    pub principal_repository: super::principal_repository::PrincipalRepository<'a>,
    pub group_principal_repository: super::group_principal_repository::GroupPrincipalRepository<'a>,
    pub resource_repository: super::resource_repository::ResourceRepository<'a>,
    pub resource_instance_repository: super::resource_instance_repository::ResourceInstanceRepository<'a>,
    pub resource_quota_repository: super::resource_quota_repository::ResourceQuotaRepository<'a>,
    pub role_repository: super::role_repository::RoleRepository<'a>,
    pub role_roleable_repository: super::role_roleable_repository::RoleRoleableRepository<'a>,
    pub claim_repository: super::claim_repository::ClaimRepository<'a>,
    pub claim_claimable_repository: super::claim_claimable_repository::ClaimClaimableRepository<'a>,
    pub license_policy_repository: super::license_policy_repository::LicensePolicyRepository<'a>,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> PersistenceManager<'a> {
    ////////////////////////////////// REALM OPERATIONS //////////////////////////////
    /// Creates or updates security realm
    pub fn new_realm_with(&self, ctx: &SecurityContext, name: &str) -> Result<SecurityRealm, RbacError> {
        self.realm_repository.create(ctx, &SecurityRealm::new(name, None))
    }

    ////////////////////////////////// LICENSE POLICY CRUD OPERATIONS //////////////////////////////
    /// Adds license-policy
    pub fn new_license_policy(&self, ctx: &SecurityContext, org: &Organization) -> Result<LicensePolicy, RbacError> {
        self.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)))
    }

    ////////////////////////////////// RESOURCE CRUD OPERATIONS //////////////////////////////
    /// Creates resource
    pub fn new_resource_with(&self, ctx: &SecurityContext, realm: &SecurityRealm, resource_name: &str) -> Result<Resource, RbacError> {
        self.resource_repository.create(ctx, &Resource::new("", realm.id.as_str(), resource_name, None, Some("(CREATE|READ|UPDATE|DELETE)".to_string())))
    }

    ////////////////////////////////// RESOURCE INSTANCE CRUD OPERATIONS //////////////////////////////
    /// Creates resource_instance
    pub fn new_resource_instance_with(&self, ctx: &SecurityContext, resource: &Resource, principal: &Principal, scope: &str, ref_id: &str, status: Status) -> Result<ResourceInstance, RbacError> {
        let mut instance = ResourceInstance::new("", resource.id.as_str(), "", scope, ref_id, status.to_string().as_str(), None);
        self.new_resource_instance(ctx, principal.id.as_str(), &mut instance)
    }

    pub fn new_resource_instance(&self, ctx: &SecurityContext, principal_id: &str, instance: &mut ResourceInstance) -> Result<ResourceInstance, RbacError> {
        if let Some(principal) = self.principal_repository.get(ctx, principal_id) {
            if let Some(policy) = self.license_policy_repository.get_by_org(&ctx, principal.organization_id.as_str()).first() {
                instance.license_policy_id = policy.id.clone();
                if let Some(quota) = self.resource_quota_repository.get_by_resource_scope(instance.resource_id.as_str(), instance.scope.as_str()).first() {
                    let count = self.resource_instance_repository.count_by_resource(instance.resource_id.as_str(), instance.scope.as_str(), Status::COMPLETED.to_string().as_str()) + self.resource_instance_repository.count_recent_by_resource(instance.resource_id.as_str(), instance.scope.as_str(), Status::INFLIGHT.to_string().as_str());
                    if count >= quota.max_value as i64 {
                        self.audit(ctx, format!("Reached quota limit for resource instance {:?}  -- {:?}", instance, quota), "CREATE");
                        return Err(RbacError::QuotaExceeded(format!("Reached quota limit for resource instance {:?} -- {:?}", instance, quota)));
                    }
                } else {
                    self.audit(ctx, format!("Reached quota limit for resource instance {:?}", instance), "CREATE");
                    return Err(RbacError::QuotaExceeded(format!("Reached limit for {:?} -- quota not found", instance)));
                }
                //
                self.resource_instance_repository._create(ctx, instance)
            } else {
                Err(RbacError::Persistence(format!("License policy not found for principal {:?} while adding resource instance {:?}", principal, instance)))
            }
        } else {
            Err(RbacError::NotFound(format!("Principal not found {:?} while adding resource instance {:?}", principal_id, instance)))
        }
    }

    ////////////////////////////////// RESOURCE QUOTA CRUD OPERATIONS //////////////////////////////
    /// Creates resource_quota
    pub fn new_resource_quota_with(&self, ctx: &SecurityContext, resource: &Resource, principal: &Principal, scope: &str, max_value: i32) -> Result<ResourceQuota, RbacError> {
        let mut quota = ResourceQuota::new("", resource.id.as_str(), "", scope, max_value, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        self.new_resource_quota(ctx, principal.id.as_str(), &mut quota)
    }

    pub fn new_resource_quota(&self, ctx: &SecurityContext, principal_id: &str, quota: &mut ResourceQuota) -> Result<ResourceQuota, RbacError> {
        if let Some(principal) = self.principal_repository.get(ctx, principal_id) {
            if let Some(policy) = self.license_policy_repository.get_by_org(ctx, principal.organization_id.as_str()).first() {
                quota.license_policy_id = policy.id.clone();
                self.resource_quota_repository.create(ctx, quota)
            } else {
                Err(RbacError::NotFound(format!("License policy not found for {:?} while adding resource quota {:?}", principal_id, quota)))
            }
        } else {
            Err(RbacError::NotFound(format!("Principal not found {:?} while adding resource quota {:?}", principal_id, quota)))
        }
    }

    ////////////////////////////////// ORGANIZATION OPERATIONS //////////////////////////////
    /// Creates or updates organization
    pub fn new_org_with(&self, ctx: &SecurityContext, name: &str) -> Result<Organization, RbacError> {
        self.org_repository.create(ctx, &Organization::new("", None, name, "", None))
    }

    /// Retrieves organization by org-id from the database
    pub fn get_org(&self, ctx: &SecurityContext, realm_id: &str, organization_id: &str) -> Option<Organization> {
        if let Some(mut org) = self.org_repository.get(ctx, organization_id) {
            self.populate_org(ctx, realm_id, &mut org);
            Some(org)
        } else {
            None
        }
    }

    ////////////////////////////////// Group CRUD OPERATIONS //////////////////////////////
    /// Creates group with parent
    pub fn new_group_with_parent(&self, ctx: &SecurityContext, org: &Organization, parent: &Group, name: &str) -> Result<Group, RbacError> {
        self.group_repository.create(ctx, &Group::new("".into(), org.id.as_str(), name, None, Some(parent.id.clone())))
    }

    /// Creates group
    pub fn new_group_with(&self, ctx: &SecurityContext, org: &Organization, name: &str) -> Result<Group, RbacError> {
        self.group_repository.create(ctx, &Group::new("".into(), org.id.as_str(), name, None, None))
    }


    ////////////////////////////////// Role CRUD OPERATIONS //////////////////////////////
    /// Creates role with parent
    pub fn new_role_with_parent(&self, ctx: &SecurityContext, realm: &SecurityRealm, org: &Organization, parent: &Role, name: &str) -> Result<Role, RbacError> {
        self.role_repository.create(ctx, &Role::new("".into(), realm.id.as_str(), org.id.as_str(), name, None, Some(parent.id.clone())))
    }

    /// Creates role
    pub fn new_role_with(&self, ctx: &SecurityContext, realm: &SecurityRealm, org: &Organization, name: &str) -> Result<Role, RbacError> {
        self.role_repository.create(ctx, &Role::new("".into(), realm.id.as_str(), org.id.as_str(), name, None, None))
    }

    ////////////////////////////////// Principal CRUD OPERATIONS //////////////////////////////
    /// Creates principal
    pub fn new_principal_with(&self, ctx: &SecurityContext, org: &Organization, username: &str) -> Result<Principal, RbacError> {
        self.principal_repository.create(ctx, &Principal::new("", org.id.as_str(), username, None))
    }

    /// Retrieves principal by user-id from the database
    pub fn get_principal(&self, ctx: &SecurityContext, realm_id: &str, principal_id: &str) -> Option<Principal> {
        if let Some(mut principal) = self.principal_repository.get(&ctx, principal_id) {
            self.populate_principal(ctx, realm_id, &mut principal);
            Some(principal)
        } else {
            None
        }
    }


    ////////////////////////////////// CLAIM CRUD OPERATIONS //////////////////////////////
    /// Creates claim
    pub fn new_claim_with(&self, ctx: &SecurityContext, realm: &SecurityRealm, resource: &Resource, action: &str) -> Result<Claim, RbacError> {
        self.claim_repository.create(ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), action, Constants::Allow.to_string().as_str(), None))
    }


    /// Adds principal group
    pub fn map_principal_to_group(&self, ctx: &SecurityContext, principal: &Principal, group: &Group) -> Result<usize, RbacError> {
        self.group_principal_repository.add_principal_to_group(ctx, group.id.as_str(), principal.id.as_str())
    }

    /// Removes principal from group
    pub fn unmap_principal_from_group(&self, ctx: &SecurityContext, principal: &Principal, group: &Group) -> Result<usize, RbacError> {
        self.group_principal_repository.delete_principal_from_group(ctx, group.id.as_str(), principal.id.as_str())
    }

    /// Adds role to principal
    pub fn map_principal_to_role(&self, ctx: &SecurityContext, principal: &Principal, role: &Role) -> Result<usize, RbacError> {
        self.role_roleable_repository.add_principal_to_role(ctx, role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Removes principal from role
    pub fn unmap_principal_from_role(&self, ctx: &SecurityContext, principal: &Principal, role: &Role) -> Result<usize, RbacError> {
        self.role_roleable_repository.delete_principal_from_role(ctx, role.id.as_str(), principal.id.as_str())
    }

    /// Adds group to role
    pub fn map_group_to_role(&self, ctx: &SecurityContext, group: &Group, role: &Role, constraints: &str) -> Result<usize, RbacError> {
        self.role_roleable_repository.add_group_to_role(ctx, role.id.as_str(), group.id.as_str(), constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Removes group from role
    pub fn unmap_group_from_role(&self, ctx: &SecurityContext, group: &Group, role: &Role) -> Result<usize, RbacError> {
        self.role_roleable_repository.delete_group_from_role(ctx, role.id.as_str(), group.id.as_str())
    }


    /// Adds role to claim
    pub fn map_role_to_claim(&self, ctx: &SecurityContext, role: &Role, claim: &Claim, scope: &str, constraints: &str) -> Result<usize, RbacError> {
        self.claim_claimable_repository.add_role_to_claim(ctx, role.id.as_str(), claim.id.as_str(), scope, constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Removes role from claim
    pub fn unmap_role_from_claim(&self, ctx: &SecurityContext, role: &Role, claim: &Claim) -> Result<usize, RbacError> {
        self.claim_claimable_repository.delete_role_from_claim(ctx, role.id.as_str(), claim.id.as_str())
    }


    /// Adds principal to claim
    pub fn map_principal_to_claim(&self, ctx: &SecurityContext, principal: &Principal, claim: &Claim, scope: &str, constraints: &str) -> Result<usize, RbacError> {
        self.claim_claimable_repository.add_principal_to_claim(ctx, principal.id.as_str(), claim.id.as_str(), scope, constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Removes principal from claim
    pub fn unmap_principal_from_claim(&self, ctx: &SecurityContext, principal: &Principal, claim: &Claim) -> Result<usize, RbacError> {
        self.claim_claimable_repository.delete_principal_from_claim(ctx, principal.id.as_str(), claim.id.as_str())
    }


    /// Adds license-policy to claim
    pub fn map_license_policy_to_claim(&self, ctx: &SecurityContext, policy: &LicensePolicy, claim: &Claim, scope: &str, constraints: &str) -> Result<usize, RbacError> {
        self.claim_claimable_repository.add_license_policy_to_claim(ctx, policy.id.as_str(), claim.id.as_str(), scope, constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Removes license-policy from claim
    pub fn unmap_license_policy_from_claim(&self, ctx: &SecurityContext, policy: &LicensePolicy, claim: &Claim) -> Result<usize, RbacError> {
        self.claim_claimable_repository.delete_license_policy_from_claim(ctx, policy.id.as_str(), claim.id.as_str())
    }

    /// Returns all resources for given claims - used by security manager
    pub fn get_resources_by_claims(&self, ctx: &SecurityContext, realm_id: &str, principal: &Principal, resource_name: String, scope: String) -> Vec<ClaimResource> {
        // Checking claims against license-policy
        let org_claim_claimables = self.get_claim_claimables_by_org(ctx, realm_id, principal.organization_id.as_str());
        let mut matched = false;
        for org_claim_claimable in &org_claim_claimables {
            match org_claim_claimable {
                ClaimClaimable::LicensePolicy(_, _, claim_scope, _) => {
                    if *claim_scope == scope {
                        matched = true;
                        break;
                    }
                },
                ClaimClaimable::Realm(_, _) => {
                    matched = true;
                    break;
                },
                _ => (),
            };
        }

        if !matched {
            warn!("Access to {} {} for user {}-{} denied because no matching claims by license policy exist", resource_name, scope, principal.username, principal.id);
            return vec![];
        }

        let mut result = vec![];
        let empty = &"".to_string();
        for (_, resource) in self.resource_repository.get_by_realm(ctx, realm_id) {
            for cc in &principal.claims {
                let (claim, claim_scope, claim_constraints) = match cc {
                    ClaimClaimable::Realm(claim, _) => (claim, empty, empty),
                    ClaimClaimable::LicensePolicy(claim, _, scope, constraints) => (claim, scope, constraints),
                    ClaimClaimable::Role(claim, _, _, scope, constraints) => (claim, scope, constraints),
                    ClaimClaimable::Principal(claim, _, _, scope, constraints) => (claim, scope, constraints),
                };
                //
                if claim.resource_id == resource.id && resource.resource_name == resource_name && *claim_scope == scope {
                    result.push(ClaimResource::new(claim.clone(), claim_scope.clone(), claim_constraints.clone(), resource.clone()));
                }
            }
        }
        result
    }


    ////////////////////////////////// PRIVATE METHODS //////////////////////////////
    fn populate_roles(&self, ctx: &SecurityContext, org_roles: &HashMap<String, Role>, role_ids: &Vec<String>, principal: &mut Principal) {
        for role_id in role_ids {
            if let Some(role) = org_roles.get(role_id) {
                principal.roles.insert(role_id.clone(), role.clone());
                if let Some(ref parent_id) = role.parent_id {
                    self.populate_roles(ctx, org_roles, &vec![parent_id.clone()], principal);
                }
            } else {
                self.audit(ctx, format!("Failed to add role with id {} for {}-{} because it's not mapped to org while populating principal", role_id, principal.username, principal.id), "GET");
            }
        }
    }

    fn populate_principal(&self, ctx: &SecurityContext, realm_id: &str, principal: &mut Principal) {
        // populate roles directly map to principal
        let org_roles = self.role_repository.get_by_org(ctx, principal.organization_id.as_str());
        self.populate_roles(ctx, &org_roles, &self.role_roleable_repository.get_role_ids_by_principal(principal.id.as_str()), principal);

        // Checking groups
        let org_groups = self.group_repository.get_by_org(ctx, principal.organization_id.as_str());
        for group_id in &self.group_repository.get_group_ids_by_principal(ctx, principal.id.as_str()) {
            // populate roles indirectly map to group
            self.populate_roles(ctx, &org_roles, &self.role_roleable_repository.get_role_ids_by_group(group_id.clone()), principal);
            //
            // Adding groups
            if let Some(group) = org_groups.get(group_id) {
                principal.groups.insert(group.id.clone(), group.clone());
            } else {
                self.audit(ctx, format!("Failed to find group for id {} for {}-{} while populating principal", group_id, principal.username, principal.id), "GET");
            }
        }

        //
        // Create role-ids lookup
        let mut role_ids = vec![];
        for (_, role) in &principal.roles {
            role_ids.push(role.id.clone());
        }

        // Checking license-policy
        let org_claim_claimables = self.get_claim_claimables_by_org(ctx, realm_id, principal.organization_id.as_str());
        let mut claim_id_scopes = HashMap::new();
        let mut claims_by_id = HashMap::new();
        for org_claim_claimable in &org_claim_claimables {
            match org_claim_claimable {
                ClaimClaimable::LicensePolicy(claim, _, scope, constraints) => {
                    if scope.len() > 0 || constraints.len() > 0 {
                        claim_id_scopes.insert(format!("{}_{}", claim.id, scope), true);
                    }
                    claims_by_id.insert(claim.id.clone(), claim.clone());
                },
                ClaimClaimable::Realm(claim, _) => {
                    claims_by_id.insert(claim.id.clone(), claim.clone());
                }
                _ => (),
            };
        }

        // Find claims mapped to roles
        for cc in &self.claim_claimable_repository.get_by_roles(role_ids) {
            if let Some(claim) = claims_by_id.get(&cc.claim_id) {
                if claim_id_scopes.len() > 0 && (cc.scope.len() > 0 || cc.claim_constraints().len() > 0) && claim_id_scopes.get(&format!("{}_{}", claim.id, cc.scope)) == None {
                    self.audit(ctx, format!("Found different or missing role scope/constraints than what was set in policy principal claim: {:?}, org claim: {:?}, all org claims: {:?}", cc, claim, org_claim_claimables), "GET");
                } else {
                    principal.claims.push(ClaimClaimable::Role(claim.clone(), realm_id.to_string(), cc.claimable_id.clone(), cc.scope.clone(), cc.claim_constraints().clone()));
                }
            } else {
                self.audit(ctx, format!("Failed to find claim for id {} - principal {}-{} while populating principal", cc.claim_id, principal.username, principal.id), "GET");
            }
        }

        // Find claims mapped directly to principal
        for cc in &self.claim_claimable_repository.get_by_principal(principal.id.clone()) {
            if let Some(claim) = claims_by_id.get(&cc.claim_id) {
                if claim_id_scopes.len() > 0 && (cc.scope.len() > 0 || cc.claim_constraints().len() > 0) && claim_id_scopes.get(&format!("{}_{}", claim.id, cc.scope)) == None {
                    self.audit(ctx, format!("Found different or missing principal scope/constraints than what was set in policy {:?} - {:?}", claim, cc), "GET");
                } else {
                    principal.claims.push(ClaimClaimable::Principal(claim.clone(), realm_id.to_string(), cc.claimable_id.clone(), cc.scope.clone(), cc.claim_constraints().clone()));
                }
            } else {
                self.audit(ctx, format!("Failed to find claim for id {} - principal {}-{} while populating principal", cc.claim_id, principal.username, principal.id), "GET");
            }
        }

        // Created resources
        let mut resource_ids = vec![];
        for cc in &principal.claims {
            match cc {
                ClaimClaimable::Role(claim, _, _, _, _) => resource_ids.push(claim.resource_id.clone()),
                ClaimClaimable::Principal(claim, _, _, _, _) => resource_ids.push(claim.resource_id.clone()),
                _ => (),
            };
        }
        principal.resources = self.resource_repository._get_by_ids(resource_ids).iter().map(|r| Resource::from(r)).collect::<Vec<Resource>>();
    }


    fn get_claim_claimables_by_org(&self, ctx: &SecurityContext, realm_id: &str, organization_id: &str) -> Vec<ClaimClaimable> {
        let claims = self.claim_repository.get_claims_by_realm(ctx, realm_id);
        let policies = self.license_policy_repository.get_by_org(ctx, organization_id);
        let mut result = vec![];
        if let Some(license_policy) = policies.first() {
            for cc in &self.claim_claimable_repository.get_by_policy(license_policy.id.as_str()) {
                if let Some(claim) = claims.get(&cc.claim_id) {
                    result.push(ClaimClaimable::LicensePolicy(claim.clone(), realm_id.to_string(), cc.scope.clone(), cc.claim_constraints().clone()));
                } else {
                    self.audit(ctx, format!("Failed to find claim for id {}", cc.claim_id), "GET");
                }
            }
        } else {
            for (_, claim) in claims {
                result.push(ClaimClaimable::Realm(claim.clone(), realm_id.to_string()));
            }
        }
        result
    }

    fn populate_org(&self, ctx: &SecurityContext, realm_id: &str, org: &mut Organization) {
        org.claims = self.get_claim_claimables_by_org(ctx, realm_id, org.id.as_str());
        let mut resource_ids = vec![];
        for cc in &org.claims {
            match cc {
                ClaimClaimable::LicensePolicy(claim, _, _, _) => resource_ids.push(claim.resource_id.clone()),
                ClaimClaimable::Realm(claim, _) => resource_ids.push(claim.resource_id.clone()),
                _ => (),
            };
        }
        org.resources = self.resource_repository._get_by_ids(resource_ids).iter().map(|r| Resource::from(r)).collect::<Vec<Resource>>();
        for role in &self.role_repository._get_by_org(org.id.as_str()) {
            org.roles.insert(role.id.clone(), Role::from(&role));
        }
        for group in &self.group_repository._get_by_org(org.id.as_str()) {
            org.groups.insert(group.id.clone(), Group::from(&group));
        }
    }

    pub fn get_claims_by_policy(&self, ctx: &SecurityContext, realm_id: &str, license_policy_id: &str) -> HashMap<String, Claim> {
        let all_claims = self.claim_repository.get_claims_by_realm(ctx, realm_id);
        let mut matched_claims = HashMap::new();
        for cc in &self.claim_claimable_repository.get_by_policy(license_policy_id) {
            if let Some(claim) = all_claims.get(&cc.claim_id) {
                matched_claims.insert(claim.id.clone(), claim.clone());
            }
        }
        if matched_claims.len() > 0 {
            matched_claims
        } else {
            all_claims
        }
    }



    /// Clear - cleans up database for testing
    pub fn clear(&self) {
        self.claim_claimable_repository.clear();
        self.license_policy_repository.clear();
        self.claim_repository.clear();
        self.role_roleable_repository.clear();
        self.role_repository.clear();
        self.resource_quota_repository.clear();
        self.resource_instance_repository.clear();
        self.resource_repository.clear();
        self.group_principal_repository.clear();
        self.group_repository.clear();
        self.principal_repository.clear();
        self.org_repository.clear();
        self.realm_repository.clear();
    }

    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }
}

#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::common::SecurityContext;
    use plexrbac::security::manager::SecurityManager;
    use plexrbac::security::request::PermissionRequest;
    use plexrbac::security::response::PermissionResponse;
    use plexrbac::domain::models::*;
    use plexrbac::common::*;
    use chrono::{NaiveDate, Utc, Datelike};

    fn init() {
        let _ = env_logger::try_init();
        //env_logger::init();
        //let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_get_full_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = pm.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let org_str = format!("{:?}", org);
        let loaded = pm.get_org(&ctx, realm.id.as_str(), org.id.as_str()).unwrap();
        assert_eq!(org_str, format!("{:?}", loaded));
        pm.license_policy_repository.get(&ctx, org.id.as_str(), license_policy.id.as_str()).unwrap();
    }

    #[test]
    fn test_get_full_principal() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = pm.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        //
        let principal = pm.principal_repository.create(&ctx, &Principal::new("", org.id.as_str(), "myusername-principal", Some("desc".to_string()))).unwrap();
        let principal_str = format!("{:?}", principal);
        let loaded = pm.get_principal(&ctx, realm.id.as_str(), principal.id.as_str()).unwrap();
        assert_eq!(principal_str, format!("{:?}", loaded));
        pm.license_policy_repository.get(&ctx, org.id.as_str(), license_policy.id.as_str()).unwrap();
    }

    #[test]
    fn test_add_delete_principal_to_group() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let group = pm.group_repository.create(&ctx, &Group::new("", org.id.as_str(), "mygroup", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let principal = pm.principal_repository.create(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();
        assert!(pm.group_principal_repository.add_principal_to_group(&ctx, group.id.as_str(), principal.id.as_str()).is_ok());
        assert!(pm.group_principal_repository.delete_principal_from_group(&ctx, group.id.as_str(), principal.id.as_str()).is_ok());
    }

    #[test]
    fn test_add_delete_principal_to_role() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        //
        let role = pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "myrole", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let principal = pm.principal_repository.create(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();

        assert!(pm.role_roleable_repository.add_principal_to_role(&ctx, role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.role_roleable_repository.delete_principal_from_role(&ctx, role.id.as_str(), principal.id.as_str()).is_ok());
    }

    #[test]
    fn test_add_delete_principal_to_claims() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let principal = pm.principal_repository.create(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();
        let resource = pm.new_resource_with(&ctx, &realm, "report").unwrap();
        let claim1 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", Constants::Deny.to_string().as_str(), None)).unwrap();
        let claim2 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", Constants::Allow.to_string().as_str(), None)).unwrap();
        assert!(pm.claim_claimable_repository.add_principal_to_claim(&ctx, principal.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.claim_claimable_repository.add_principal_to_claim(&ctx, principal.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.claim_claimable_repository.delete_principal_from_claim(&ctx, principal.id.as_str(), claim1.id.as_str()).is_ok());
        assert!(pm.claim_claimable_repository.delete_principal_from_claim(&ctx, principal.id.as_str(), claim2.id.as_str()).is_ok());
    }

    #[test]
    fn test_add_delete_role_to_claims() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let role = pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "myrole", None, None)).unwrap();
        let resource = pm.new_resource_with(&ctx, &realm, "report").unwrap();
        let claim1 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", Constants::Deny.to_string().as_str(), None)).unwrap();
        let claim2 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", Constants::Allow.to_string().as_str(), None)).unwrap();
        assert!(pm.claim_claimable_repository.add_role_to_claim(&ctx, role.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.claim_claimable_repository.add_role_to_claim(&ctx, role.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.claim_claimable_repository.delete_role_from_claim(&ctx, role.id.as_str(), claim1.id.as_str()).is_ok());
        assert!(pm.claim_claimable_repository.delete_role_from_claim(&ctx, role.id.as_str(), claim2.id.as_str()).is_ok());
    }

    #[test]
    fn test_get_save_resource_quota() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();

        let resource = pm.new_resource_with(&ctx, &realm, "report").unwrap();
        let loaded = pm.resource_repository.get(&ctx, "myrealm", resource.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", resource), format!("{:?}", loaded));

        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();

        let policy = pm.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let _ = pm.resource_quota_repository.create(&ctx, &ResourceQuota::new("", resource.id.as_str(), policy.id.as_str(), "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let instance = pm.resource_instance_repository._create(&ctx, &ResourceInstance::new("", resource.id.as_str(), "22", "", "refid", "INFLIGHT", Some("blah".to_string()))).unwrap();

        let loaded = pm.resource_instance_repository.get(&ctx, instance.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", instance), format!("{:?}", loaded));

        let quota = pm.resource_quota_repository.create(&ctx, &ResourceQuota::new("", resource.id.as_str(), "22", "", 22, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let loaded = pm.resource_quota_repository.get(&ctx, quota.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", quota), format!("{:?}", loaded));
    }

    #[test]
    fn test_add_delete_claims_to_license_policy() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = pm.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let resource = pm.new_resource_with(&ctx, &realm, "report").unwrap();
        let claim1 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", "allow", None)).unwrap();
        let claim2 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", "allow", None)).unwrap();

        assert!(pm.claim_claimable_repository.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.claim_claimable_repository.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).is_ok());
        assert!(pm.claim_claimable_repository.delete_license_policy_from_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str()).is_ok());
        assert!(pm.claim_claimable_repository.delete_license_policy_from_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str()).is_ok());

        let loaded = pm.license_policy_repository.get(&ctx, org.id.as_str(), license_policy.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", license_policy), format!("{:?}", loaded));
    }

    #[test]
    fn test_populate_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = pm.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "admin", None, None)).unwrap();
        pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "employee", None, None)).unwrap();
        pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "manager", None, None)).unwrap();
        pm.group_repository.create(&ctx, &Group::new("", org.id.as_str(), "default", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        pm.group_repository.create(&ctx, &Group::new("", org.id.as_str(), "devops", Some("desc".to_string()), Some("parent".to_string()))).unwrap();

        let resource = pm.new_resource_with(&ctx, &realm, "report").unwrap();
        let claim1 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", "allow", None)).unwrap();
        let claim2 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", "allow", None)).unwrap();
        pm.claim_claimable_repository.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).unwrap();
        pm.claim_claimable_repository.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).unwrap();
        let loaded = pm.get_org(&ctx, realm.id.as_str(), org.id.as_str()).unwrap();
        assert_eq!(3, loaded.roles.len());
        assert_eq!(2, loaded.groups.len());
        assert_eq!(2, loaded.claims.len());
    }

    #[test]
    fn test_populate_principal() {
        let ctx = SecurityContext::new("myorg", "myid");
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        let realm = pm.realm_repository.create(&ctx, &SecurityRealm::new("myrealm", None)).unwrap();
        let org = pm.org_repository.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = pm.license_policy_repository.create(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let org_employee_role = pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "org_employee", None, None)).unwrap();
        let org_manager_role = pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "org_manager", None, None)).unwrap();

        let _default_group_employee = pm.role_repository.create(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "default_group_employee", None, None)).unwrap();

        let principal = pm.principal_repository.create(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();
        let default_group = pm.group_repository.create(&ctx, &Group::new("", org.id.as_str(), "default", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        pm.group_principal_repository.add_principal_to_group(&ctx, default_group.id.as_str(), principal.id.as_str()).unwrap();

        pm.role_roleable_repository.add_principal_to_role(&ctx, org_manager_role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).unwrap();
        pm.role_roleable_repository.add_group_to_role(&ctx, org_employee_role.id.as_str(), default_group.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).unwrap();

        let resource = pm.new_resource_with(&ctx, &realm, "report").unwrap();
        let claim1 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", "Allow", None)).unwrap();
        let claim2 = pm.claim_repository.create(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", "Allow", None)).unwrap();
        pm.claim_claimable_repository.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).unwrap();
        pm.claim_claimable_repository.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)).unwrap();
        let loaded = pm.get_principal(&ctx, realm.id.as_str(), principal.id.as_str()).unwrap();
        assert_eq!(2, loaded.roles.len());
        assert_eq!(1, loaded.groups.len());
        assert!(pm.role_roleable_repository.delete_group_from_role(&ctx, org_employee_role.id.as_str(), default_group.id.as_str()).is_ok());
    }

    #[test]
    fn test_banking() {
        init();

        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "banking").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "bank-of-flakes").unwrap();

        // Creating Users
        let tom = pm.new_principal_with(&ctx, &org, "tom").unwrap();
        let cassy = pm.new_principal_with(&ctx, &org, "cassy").unwrap();
        let ali = pm.new_principal_with(&ctx, &org, "ali").unwrap();
        let mike = pm.new_principal_with(&ctx, &org, "mike").unwrap();
        let larry = pm.new_principal_with(&ctx, &org, "larry").unwrap();

        // Creating Roles
        let employee = pm.new_role_with(&ctx, &realm, &org, "Employee").unwrap();
        let teller = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Teller").unwrap();
        let csr = pm.new_role_with_parent(&ctx, &realm, &org, &teller, "CSR").unwrap();
        let accountant = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Accountant").unwrap();
        let accountant_manager = pm.new_role_with_parent(&ctx, &realm, &org, &accountant, "AccountingManager").unwrap();
        let loan_officer = pm.new_role_with_parent(&ctx, &realm, &org, &accountant_manager, "LoanOfficer").unwrap();

        // Creating Resources
        let deposit_account = pm.new_resource_with(&ctx, &realm, "DepositAccount").unwrap();
        let loan_account = pm.new_resource_with(&ctx, &realm, "LoanAccount").unwrap();
        let general_ledger = pm.new_resource_with(&ctx, &realm, "GeneralLedger").unwrap();
        let posting_rules = pm.new_resource_with(&ctx, &realm, "GeneralLedgerPostingRules").unwrap();

        // Creating claims for resources
        let cd_deposit = pm.new_claim_with(&ctx, &realm, &deposit_account, "(CREATE|DELETE)").unwrap();
        let ru_deposit = pm.new_claim_with(&ctx, &realm, &deposit_account, "(READ|UPDATE)").unwrap();

        let cd_loan = pm.new_claim_with(&ctx, &realm, &loan_account, "(CREATE|DELETE)").unwrap();
        let ru_loan = pm.new_claim_with(&ctx, &realm, &loan_account, "(READ|UPDATE)").unwrap();

        let rd_ledger = pm.new_claim_with(&ctx, &realm, &general_ledger, "(READ|CREATE|DELETE)").unwrap();
        let r_glpr = pm.new_claim_with(&ctx, &realm, &general_ledger, "(READ)").unwrap();

        let cud_glpr = pm.new_claim_with(&ctx, &realm, &posting_rules, "(CREATE|UPDATE|DELETE)").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &tom, &teller).unwrap();
        pm.map_principal_to_role(&ctx, &cassy, &csr).unwrap();
        pm.map_principal_to_role(&ctx, &ali, &accountant).unwrap();
        pm.map_principal_to_role(&ctx, &mike, &accountant_manager).unwrap();
        pm.map_principal_to_role(&ctx, &larry, &loan_officer).unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &teller, &ru_deposit, "U.S.", r#"employeeRegion == "Midwest""#).unwrap();
        pm.map_role_to_claim(&ctx, &csr, &cd_deposit, "U.S.", r#"employeeRegion == "Midwest""#).unwrap();
        pm.map_role_to_claim(&ctx, &accountant, &rd_ledger, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#).unwrap();
        pm.map_role_to_claim(&ctx, &accountant, &ru_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#).unwrap();
        pm.map_role_to_claim(&ctx, &accountant_manager, &cd_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#).unwrap();
        pm.map_role_to_claim(&ctx, &accountant_manager, &r_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#).unwrap();
        pm.map_role_to_claim(&ctx, &loan_officer, &cud_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#).unwrap();

        let sm = SecurityManager::new(pm);
        // Tom, the teller should be able to READ DepositAccount with scope U.S when employeeRegion
        // == Midwest
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Tom, the teller should not be able to READ DepositAccount with scope U.S when employeeRegion
        // == Northeast
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Northeast".to_string()));
        assert!(sm.check(&req).is_err());

        // Tom, the teller should not be able to DELETE DepositAccount with scope U.S when employeeRegion
        // == Midwest
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert!(sm.check(&req).is_err());

        // Cassy, the CSR should be able to DELETE DepositAccount with scope U.S when employeeRegion
        // == Midwest
        let pm = locator.new_persistence_manager();
        let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Cassy, the CSR should be able to DELETE DepositAccount with scope U.K when employeeRegion
        // == Midwest
        let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.K.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert!(sm.check(&req).is_err());

        // Ali, the Accountant should be able to READ GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::READ, "GeneralLedger", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Ali, the Accountant should not be able to READ GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear is in past
        req.context.add("ledgerYear", ValueWrapper::Int(2000));
        assert!(sm.check(&req).is_err());

        // Ali, the Accountant should not be able to DELETE GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::DELETE, "GeneralLedger", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        assert!(sm.check(&req).is_err());

        // Mike, the Accountant Manager should be able to DELETE GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedger", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());


        // Mike, the Accountant Manager should not be able to post posting-rules of general-ledger with scope U.S 
        // when employeeRegion == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        req.context.add("accountBlance", ValueWrapper::Int(500));
        assert!(sm.check(&req).is_err());

        // Larry, the Loan Officer should be able to post posting-rules of general-ledger with scope U.S 
        // when employeeRegion == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), larry.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        req.context.add("accountBlance", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        pm.unmap_role_from_claim(&ctx, &teller, &ru_deposit).unwrap();
        pm.unmap_role_from_claim(&ctx, &csr, &cd_deposit).unwrap();
        pm.unmap_role_from_claim(&ctx, &accountant, &rd_ledger).unwrap();
        pm.unmap_role_from_claim(&ctx, &accountant, &ru_loan).unwrap();
        pm.unmap_role_from_claim(&ctx, &accountant_manager, &cd_loan).unwrap();
        pm.unmap_role_from_claim(&ctx, &accountant_manager, &r_glpr).unwrap();
        pm.unmap_role_from_claim(&ctx, &loan_officer, &cud_glpr).unwrap();

        pm.unmap_role_from_claim(&ctx, &loan_officer, &cud_glpr).unwrap();
        assert_eq!(7, pm.claim_repository.get_claims_by_realm(&ctx, realm.id.as_str()).len());
        assert_eq!(7, pm.get_claims_by_policy(&ctx, realm.id.as_str(), "99").len());

        pm.unmap_principal_from_role(&ctx, &tom, &teller).unwrap();
        pm.unmap_principal_from_role(&ctx, &cassy, &csr).unwrap();
        pm.unmap_principal_from_role(&ctx, &ali, &accountant).unwrap();
        pm.unmap_principal_from_role(&ctx, &mike, &accountant_manager).unwrap();
        pm.unmap_principal_from_role(&ctx, &larry, &loan_officer).unwrap();
    }

    #[test]
    fn test_expense_report_with_groups() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "hypatia").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "hypatia").unwrap();

        // Creating Groups
        let group_employee = pm.new_group_with(&ctx, &org, "Employee").unwrap();
        let group_manager = pm.new_group_with_parent(&ctx, &org, &group_employee, "Manager").unwrap();

        // Creating Users
        let tom = pm.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = pm.new_principal_with(&ctx, &org, "mike").unwrap();

        // Mapping users to groups
        pm.map_principal_to_group(&ctx, &tom, &group_employee).unwrap();
        pm.map_principal_to_group(&ctx, &mike, &group_employee).unwrap();
        pm.map_principal_to_group(&ctx, &mike, &group_manager).unwrap();

        // Creating Roles
        let employee = pm.new_role_with(&ctx, &realm, &org, "Employee").unwrap();
        let manager = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Manager").unwrap();

        // Creating Resources
        let expense_report = pm.new_resource_with(&ctx, &realm, "ExpenseReport").unwrap();

        // Creating claims for resources
        let submit_report = pm.new_claim_with(&ctx, &realm, &expense_report, "SUBMIT").unwrap();
        let approve_report = pm.new_claim_with(&ctx, &realm, &expense_report, "APPROVE").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_group_to_role(&ctx, &group_employee, &employee, "").unwrap();
        pm.map_group_to_role(&ctx, &group_manager, &manager, "").unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &employee, &submit_report, "U.S.", r#"amount < 10000"#).unwrap();
        pm.map_role_to_claim(&ctx, &manager, &approve_report, "U.S.", r#"amount < 10000"#).unwrap();

        let sm = SecurityManager::new(pm);
        // Tom should be able to submit report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::SUBMIT, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Tom should not be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert!(sm.check(&req).is_err());

        // Mike should be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        let pm = locator.new_persistence_manager();
        pm.unmap_principal_from_group(&ctx, &tom, &group_employee).unwrap();
        pm.unmap_principal_from_group(&ctx, &mike, &group_employee).unwrap();
        pm.unmap_principal_from_group(&ctx, &mike, &group_manager).unwrap();

        pm.unmap_group_from_role(&ctx, &group_employee, &employee).unwrap();
        pm.unmap_group_from_role(&ctx, &group_manager, &manager).unwrap();
    }

    #[test]
    fn test_expense_report_with_direct_claim_to_principal() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "rainier").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "rainier").unwrap();

        // Creating Users
        let tom = pm.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = pm.new_principal_with(&ctx, &org, "mike").unwrap();

        // Creating Resources
        let expense_report = pm.new_resource_with(&ctx, &realm, "ExpenseReport").unwrap();

        // Creating claims for resources
        let submit_report = pm.new_claim_with(&ctx, &realm, &expense_report, "SUBMIT").unwrap();
        let approve_report = pm.new_claim_with(&ctx, &realm, &expense_report, "APPROVE").unwrap();

        // Map claims to roles as follows:
        pm.map_principal_to_claim(&ctx, &tom, &submit_report, "U.S.", r#"amount < 10000"#).unwrap();
        pm.map_principal_to_claim(&ctx, &mike, &approve_report, "U.S.", r#"amount < 10000"#).unwrap();

        let sm = SecurityManager::new(pm);
        // Tom should be able to submit report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::SUBMIT, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Tom should not be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert!(sm.check(&req).is_err());

        // Mike should be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        let pm = locator.new_persistence_manager();
        pm.unmap_principal_from_claim(&ctx, &tom, &submit_report).unwrap();
        pm.unmap_principal_from_claim(&ctx, &mike, &approve_report).unwrap();
    }

    #[test]
    fn test_feature_flag_with_geo_fencing() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "ada").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "ada").unwrap();

        // Creating Users
        let tom = pm.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = pm.new_principal_with(&ctx, &org, "mike").unwrap();

        // Creating Roles
        let customer = pm.new_role_with(&ctx, &realm, &org, "Customer").unwrap();
        let beta_customer = pm.new_role_with_parent(&ctx, &realm, &org, &customer, "BetaCustomer").unwrap();

        // Creating Resources
        let feature = pm.new_resource_with(&ctx, &realm, "Feature").unwrap();

        // Creating claims for resources
        let view = pm.new_claim_with(&ctx, &realm, &feature, "VIEW").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &tom, &customer).unwrap();
        pm.map_principal_to_role(&ctx, &mike, &beta_customer).unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 100"#).unwrap();
        pm.map_role_to_claim(&ctx, &beta_customer, &view, "UI::Flag::AdvancedReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 200"#).unwrap();

        let sm = SecurityManager::new(pm);

        // Tom should be able to view basic report if he lives close to Seattle
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
        req.context.add("customer_lat", ValueWrapper::Float(46.879967));
        req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Tom should not be able to view basic report if he lives far from Seattle
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
        req.context.add("customer_lat", ValueWrapper::Float(37.3230));
        req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
        assert!(sm.check(&req).is_err());

        // Tom should not be able to view advanced report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        req.context.add("customer_lat", ValueWrapper::Float(46.879967));
        req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
        assert!(sm.check(&req).is_err());

        // Mike should be able to view advanced report
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        req.context.add("customer_lat", ValueWrapper::Float(46.879967));
        req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Mike should not be able to view advanced report if he lives far from Seattle
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        req.context.add("customer_lat", ValueWrapper::Float(37.3230));
        req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
        assert!(sm.check(&req).is_err());
    }

    #[test]
    fn test_data_protection() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "dada").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "dada").unwrap();

        // Creating Users
        let tom = pm.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = pm.new_principal_with(&ctx, &org, "mike").unwrap();

        // Creating Roles
        let customer = pm.new_role_with(&ctx, &realm, &org, "Customer").unwrap();
        let beta_customer = pm.new_role_with_parent(&ctx, &realm, &org, &customer, "BetaCustomer").unwrap();

        // Creating Resources
        let data = pm.new_resource_with(&ctx, &realm, "Data").unwrap();

        // Creating claims for resources
        let view = pm.new_claim_with(&ctx, &realm, &data, "VIEW").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &tom, &customer).unwrap();
        pm.map_principal_to_role(&ctx, &mike, &beta_customer).unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &customer, &view, "Report::Summary", "").unwrap();
        pm.map_role_to_claim(&ctx, &beta_customer, &view, "Report::Details", "").unwrap();

        let sm = SecurityManager::new(pm);

        // Tom should be able to view summary
        let req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Data", "Report::Summary");
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Tom should not be able to view details
        let req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Data", "Report::Details");
        assert!(sm.check(&req).is_err());

        // Mike should be able to view details
        let req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Data", "Report::Details");
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());
    }

    #[test]
    fn test_license_policy() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "curie").unwrap();

        // Creating organization
        let freemium_org = pm.new_org_with(&ctx, "Freeloader").unwrap();
        let paid_org = pm.new_org_with(&ctx, "Moneymaker").unwrap();

        // Create license policies
        let freemium_policy = pm.new_license_policy(&ctx, &freemium_org).unwrap();
        let paid_policy = pm.new_license_policy(&ctx, &paid_org).unwrap();

        // Creating Users
        let freemium_frank = pm.new_principal_with(&ctx, &freemium_org, "frank").unwrap();
        let money_matt = pm.new_principal_with(&ctx, &paid_org, "matt").unwrap();

        // Creating Roles
        let customer = pm.new_role_with(&ctx, &realm, &freemium_org, "Customer").unwrap();
        let paid_customer = pm.new_role_with(&ctx, &realm, &paid_org, "PaidCustomer").unwrap();

        // Creating Resources
        let feature = pm.new_resource_with(&ctx, &realm, "Feature").unwrap();

        // Creating claims for resources
        let view = pm.new_claim_with(&ctx, &realm, &feature, "VIEW").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &freemium_frank, &customer).unwrap();
        pm.map_principal_to_role(&ctx, &money_matt, &customer).unwrap();
        pm.map_principal_to_role(&ctx, &money_matt, &paid_customer).unwrap();

        // Map claims to policies as follows:
        pm.map_license_policy_to_claim(&ctx, &freemium_policy, &view, "UI::Flag::BasicReport", "").unwrap();
        pm.map_license_policy_to_claim(&ctx, &paid_policy, &view, "UI::Flag::AdvancedReport", "").unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", "").unwrap();
        pm.map_role_to_claim(&ctx, &paid_customer, &view, "UI::Flag::AdvancedReport", "").unwrap();

        let sm = SecurityManager::new(pm);

        // Frank should be able to view basic report
        let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        let pm = locator.new_persistence_manager();
        // Frank should not be able to view advanced report
        let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        assert!(sm.check(&req).is_err());

        // Matt should be able to view advanced report
        let req = PermissionRequest::new(realm.id.as_str(), money_matt.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        pm.unmap_license_policy_from_claim(&ctx, &freemium_policy, &view).unwrap();
        pm.unmap_license_policy_from_claim(&ctx, &paid_policy, &view).unwrap();
    }

    #[test]
    fn test_app_report() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "SeeEye").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "SeeEye").unwrap();

        // Create license policies
        let policy = pm.new_license_policy(&ctx, &org).unwrap();

        // Creating Users
        let dave = pm.new_principal_with(&ctx, &org, "dave").unwrap();
        let qari = pm.new_principal_with(&ctx, &org, "qari").unwrap();
        let ali = pm.new_principal_with(&ctx, &org, "ali").unwrap();

        // Creating Roles
        let developer = pm.new_role_with(&ctx, &realm, &org, "Developer").unwrap();
        let qa = pm.new_role_with(&ctx, &realm, &org, "QA").unwrap();
        let admin = pm.new_role_with_parent(&ctx, &realm, &org, &developer, "Admin").unwrap();

        // Creating Resources
        let app = pm.new_resource_with(&ctx, &realm, "App").unwrap();

        // Creating claims for resources
        let submit_view = pm.new_claim_with(&ctx, &realm, &app, "(SUBMIT|VIEW)").unwrap();
        let view = pm.new_claim_with(&ctx, &realm, &app, "VIEW").unwrap();
        let create_delete = pm.new_claim_with(&ctx, &realm, &app, "(CREATE|DELETE)").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &dave, &developer).unwrap();
        pm.map_principal_to_role(&ctx, &qari, &qa).unwrap();
        pm.map_principal_to_role(&ctx, &ali, &admin).unwrap();

        // Map claims to policies as follows:
        pm.map_license_policy_to_claim(&ctx, &policy, &submit_view, "com.xyz.app", "appSize < 1000").unwrap();
        pm.map_license_policy_to_claim(&ctx, &policy, &view, "com.xyz.app", "appSize < 1000").unwrap();
        pm.map_license_policy_to_claim(&ctx, &policy, &create_delete, "com.xyz.app", "").unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &developer, &submit_view, "com.xyz.app", "appSize < 1000").unwrap();
        pm.map_role_to_claim(&ctx, &qa, &view, "com.xyz.app", "appSize < 1000").unwrap();
        pm.map_role_to_claim(&ctx, &admin, &create_delete, "com.xyz.app", "").unwrap();

        let sm = SecurityManager::new(pm);

        // Dave should be able to submit app
        let mut req = PermissionRequest::new(realm.id.as_str(), dave.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Qari should be able to view app
        let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::VIEW, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Qari should not be able to create app
        let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert!(sm.check(&req).is_err());

        // Ali should be able to create app
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Ali should be able to submit app
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());

        // Ali should not be able to submit app with large app
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(5000));
        assert!(sm.check(&req).is_err());
    }

    #[test]
    fn test_project() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "JobGrid").unwrap();

        // Creating organization
        let abc_corp = pm.new_org_with(&ctx, "ABC").unwrap();
        let xyz_corp = pm.new_org_with(&ctx, "XYZ").unwrap();

        // Create license policies
        let abc_policy = pm.new_license_policy(&ctx, &abc_corp).unwrap();
        let xyz_policy = pm.new_license_policy(&ctx, &xyz_corp).unwrap();

        // Creating Users
        let abc_dave = pm.new_principal_with(&ctx, &abc_corp, "dave").unwrap();
        let abc_ali = pm.new_principal_with(&ctx, &abc_corp, "ali").unwrap();

        let xyz_dan = pm.new_principal_with(&ctx, &xyz_corp, "dan").unwrap();
        let xyz_ann = pm.new_principal_with(&ctx, &xyz_corp, "ann").unwrap();

        // Creating Roles
        let abc_developer = pm.new_role_with(&ctx, &realm, &abc_corp, "Developer").unwrap();
        let abc_admin = pm.new_role_with_parent(&ctx, &realm, &abc_corp, &abc_developer, "Admin").unwrap();

        let xyz_developer = pm.new_role_with(&ctx, &realm, &xyz_corp, "Developer").unwrap();
        let xyz_admin = pm.new_role_with_parent(&ctx, &realm, &xyz_corp, &xyz_developer, "Admin").unwrap();

        // Creating Resources
        let project = pm.new_resource_with(&ctx, &realm, "Project").unwrap();
        let job = pm.new_resource_with(&ctx, &realm, "Job").unwrap();

        // Creating claims for resources
        let project_create_delete = pm.new_claim_with(&ctx, &realm, &project, "(CREATE|DELETE)").unwrap();
        let project_view = pm.new_claim_with(&ctx, &realm, &project, "VIEW").unwrap();
        let job_view_submit = pm.new_claim_with(&ctx, &realm, &job, "(VIEW|SUBMIT)").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &abc_dave, &abc_developer).unwrap();
        pm.map_principal_to_role(&ctx, &abc_ali, &abc_admin).unwrap();

        pm.map_principal_to_role(&ctx, &xyz_dan, &xyz_developer).unwrap();
        pm.map_principal_to_role(&ctx, &xyz_ann, &xyz_admin).unwrap();

        // Map claims to policies as follows:
        pm.map_license_policy_to_claim(&ctx, &abc_policy, &project_create_delete, "com.abc.app", "").unwrap();
        pm.map_license_policy_to_claim(&ctx, &abc_policy, &project_view, "com.abc.app", "").unwrap();
        pm.map_license_policy_to_claim(&ctx, &abc_policy, &job_view_submit, "com.abc.app", "appSize < 1000").unwrap();

        pm.map_license_policy_to_claim(&ctx, &xyz_policy, &project_create_delete, "com.xyz.app", "").unwrap();
        pm.map_license_policy_to_claim(&ctx, &xyz_policy, &project_view, "com.xyz.app", "").unwrap();
        pm.map_license_policy_to_claim(&ctx, &xyz_policy, &job_view_submit, "com.xyz.app", "appSize < 1000").unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &abc_admin, &project_create_delete, "com.abc.app", "").unwrap();
        pm.map_role_to_claim(&ctx, &abc_developer, &project_view, "com.abc.app", "").unwrap();
        pm.map_role_to_claim(&ctx, &abc_developer, &job_view_submit, "com.abc.app", "appSize < 1000").unwrap();

        pm.map_role_to_claim(&ctx, &xyz_admin, &project_create_delete, "com.xyz.app", "").unwrap();
        pm.map_role_to_claim(&ctx, &xyz_developer, &project_view, "com.xyz.app", "").unwrap();
        pm.map_role_to_claim(&ctx, &xyz_developer, &job_view_submit, "com.xyz.app", "appSize < 1000").unwrap();

        let sm = SecurityManager::new(pm);

        // Ali for ABC should create project
        let mut req = PermissionRequest::new(realm.id.as_str(), abc_ali.id.as_str(), ActionType::CREATE, "Project", "com.abc.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert!(sm.check(&req).is_ok());

        // Dave for ABC should be able to submit job
        let mut req = PermissionRequest::new(realm.id.as_str(), abc_dave.id.as_str(), ActionType::SUBMIT, "Job", "com.abc.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert!(sm.check(&req).is_ok());

    }

    #[test]
    fn test_quota_limits() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let ds = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&ds);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "JobGrid").unwrap();

        // Creating organization
        let abc_corp = pm.new_org_with(&ctx, "ABC").unwrap();
        let xyz_corp = pm.new_org_with(&ctx, "XYZ").unwrap();

        // Create license policies
        let _abc_policy = pm.new_license_policy(&ctx, &abc_corp).unwrap();
        let _xyz_policy = pm.new_license_policy(&ctx, &xyz_corp).unwrap();

        // Creating Resources
        let project = pm.new_resource_with(&ctx, &realm, "Project").unwrap();
        let job = pm.new_resource_with(&ctx, &realm, "Job").unwrap();

        // Creating Users
        let abc_dave = pm.new_principal_with(&ctx, &abc_corp, "dave").unwrap();
        let xyz_dan = pm.new_principal_with(&ctx, &xyz_corp, "dan").unwrap();

        // Set Resource Quota
        assert!(pm.new_resource_quota_with(&ctx, &project, &abc_dave, "ABC Project", 1).is_ok());
        assert!(pm.new_resource_quota_with(&ctx, &job, &abc_dave, "ABC Jobs", 2).is_ok());

        assert!(pm.new_resource_quota_with(&ctx, &project, &xyz_dan, "XYZ Project", 2).is_ok());
        assert!(pm.new_resource_quota_with(&ctx, &job, &xyz_dan, "XYZ Jobs", 3).is_ok());

        // abc can have at most 1 project
        assert!(pm.new_resource_instance_with(&ctx, &project, &abc_dave, "ABC Project", "1", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &project, &abc_dave, "ABC Project", "2", Status::COMPLETED).is_err());

        // abc can have at most 2 jobs
        assert!(pm.new_resource_instance_with(&ctx, &job, &abc_dave, "ABC Jobs", "1", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &job, &abc_dave, "ABC Jobs", "2", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &job, &abc_dave, "ABC Jobs", "3", Status::COMPLETED).is_err());

        // xyz can have at most 2 project
        assert!(pm.new_resource_instance_with(&ctx, &project, &xyz_dan, "XYZ Project", "1", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &project, &xyz_dan, "XYZ Project", "2", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &project, &xyz_dan, "XYZ Project", "3", Status::COMPLETED).is_err());

        // xyz can have at most 3 jobs
        assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_dan, "XYZ Jobs", "1", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_dan, "XYZ Jobs", "2", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_dan, "XYZ Jobs", "3", Status::COMPLETED).is_ok());
        assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_dan, "XYZ Jobs", "4", Status::COMPLETED).is_err());
    }

    use chrono::NaiveDateTime;
    use chrono::format::strftime::StrftimeItems;
    #[test]
    fn test_time() {
        let fmt = StrftimeItems::new("%Y-%m-%d %H:%M:%S");
        let dt = NaiveDate::from_ymd(2019, 7, 5).and_hms(23, 56, 4);
        assert_eq!(dt.format_with_items(fmt.clone()).to_string(), "2019-07-05 23:56:04");
        assert_eq!(NaiveDateTime::parse_from_str("2019-07-05 23:56:04", "%Y-%m-%d %H:%M:%S"),
           Ok(NaiveDate::from_ymd(2019, 7, 5).and_hms(23, 56, 4)));
    }
}
