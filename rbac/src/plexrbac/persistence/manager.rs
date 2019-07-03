//#![crate_name = "doc"]

extern crate uuid as uuu;
use super::models::*;
use plexrbac::domain::models::*;
use plexrbac::common::Constants;
use plexrbac::common::Status;
use plexrbac::security::context::SecurityContext;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use log::{info, warn};
use self::uuu::Uuid;
use std::collections::HashMap;
use plexrbac::common::RbacError;

//////////////////////////////////////////////////////////////////////////////////////////////
/// PersistenceManager defines high-level methods for accessing rbac entities
///
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
    ////////////////////////////////// SECURITY REALM CRUD OPERATIONS //////////////////////////////
    /// Creates or updates security realm
    pub fn new_realm_with(&self, ctx: &SecurityContext, name: &str) -> Result<SecurityRealm, RbacError> {
        self.save_realm(ctx, &SecurityRealm::new(name, ""))
    }
    /// Creates or updates security realm
    pub fn save_realm(&self, ctx: &SecurityContext, realm: &SecurityRealm) -> Result<SecurityRealm, RbacError> {
        match self.realm_repository.get(realm.id.as_str()) {
            Ok(mut db_obj) => {
                db_obj.description = realm.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.realm_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating realm {:?}", db_obj), "UPDATE");
                info!("Updating realm {:?}", db_obj);
                Ok(SecurityRealm::from(&db_obj))
            }
            _ => {
                let mut db_obj = realm.to();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.realm_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Creating new realm realm {:?}", db_obj), "CREATE");
                info!("Creating realm {:?}", db_obj);
                Ok(SecurityRealm::from(&db_obj))
            }
        }
    }
    /// Retrieves realm by realm-id from the database
    pub fn get_realm(&self, _ctx: &SecurityContext, realm_id: &str) -> Option<SecurityRealm> {
        match self.realm_repository.get(realm_id) {
            Ok(realm) => Some(SecurityRealm::from(&realm)),
            _ => None,
        }
    }

    ////////////////////////////////// ORGANIZATION CRUD OPERATIONS //////////////////////////////
    /// Creates or updates organization
    pub fn new_org_with(&self, ctx: &SecurityContext, name: &str) -> Result<Organization, RbacError> {
        self.save_org(ctx, &Organization::new("", None, name, "", None))
    }
    /// Creates or updates organization
    pub fn save_org(&self, ctx: &SecurityContext, org: &Organization) -> Result<Organization, RbacError> {
        match self.org_repository.get(org.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.url = org.url.clone();
                db_obj.description = org.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.org_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                info!("Updating organization {:?}", db_obj);
                self.audit(ctx, format!("Updating org {:?}", db_obj), "UPDATE");
                Ok(Organization::from(&db_obj))
            }
            None => {
                let mut db_obj = org.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.org_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Creating new org {:?}", db_obj), "CREATE");
                info!("Creating organization {:?}", db_obj);
                Ok(Organization::from(&db_obj))
            }
        }
    }

    /// Retrieves organization by org-id from the database
    pub fn get_org(&self, ctx: &SecurityContext, realm_id: &str, organization_id: &str) -> Option<Organization> {
        if let Some(porg) = self.org_repository.get(organization_id) {
            let mut org = Organization::from(&porg);
            self.populate_org(ctx, realm_id, &mut org);
            Some(org)
        } else {
            None
        }
    }

    fn get_claim_claimables_by_org(&self, ctx: &SecurityContext, realm_id: &str, organization_id: &str) -> Vec<ClaimClaimable> {
        let claims = self.get_claims_by_realm(ctx, realm_id);
        let policies = self.license_policy_repository.get_by_org(organization_id);
        let mut result = vec![];
        if let Some(license_policy) = policies.first() {
            for cc in &self.claim_claimable_repository.get_by_policy(license_policy.id.as_str()) {
                if let Some(claim) = claims.get(&cc.claim_id) {
                    result.push(ClaimClaimable::LicensePolicy(claim.clone(), realm_id.to_string(), cc.scope.clone(), cc.claim_constraints.clone()));
                } else {
                    warn!("Failed to find claim for id {}", cc.claim_id);
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
        org.resources = self.resource_repository.get_by_ids(resource_ids).iter().map(|r| Resource::from(r)).collect::<Vec<Resource>>();
        for role in &self.role_repository.get_by_org(org.id.as_str()) {
            org.roles.insert(role.id.clone(), Role::from(&role));
        }
        for group in &self.group_repository.get_by_org(org.id.as_str()) {
            org.groups.insert(group.id.clone(), Group::from(&group));
        }
    }

    ////////////////////////////////// Principal CRUD OPERATIONS //////////////////////////////
    /// Creates principal
    pub fn new_principal_with(&self, ctx: &SecurityContext, org: &Organization, username: &str) -> Result<Principal, RbacError> {
        self.save_principal(ctx, &Principal::new("", org.id.as_str(), username, None))
    }

    /// Creates principal
    pub fn save_principal(&self, ctx: &SecurityContext, principal: &Principal) -> Result<Principal, RbacError> {
        match self.principal_repository.get(principal.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.description = principal.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.principal_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                info!("Updating principal {:?}", db_obj);
                self.audit(ctx, format!("Updating principal {:?}", db_obj), "UPDATE");
                Ok(Principal::from(&db_obj))
            }
            None => {
                let mut db_obj = principal.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.principal_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Creating new principal {:?}", db_obj), "CREATE");
                info!("Creating principal {:?}", db_obj);
                Ok(Principal::from(&db_obj))
            }
        }
    }

    /// Retrieves principal by user-id from the database
    pub fn get_principal(&self, ctx: &SecurityContext, realm_id: &str, principal_id: &str) -> Option<Principal> {
        if let Some(pprincipal) = self.principal_repository.get(principal_id) {
            let mut principal = Principal::from(&pprincipal);
            self.populate_principal(ctx, realm_id, &mut principal);
            Some(principal)
        } else {
            None
        }
    }

    fn add_roles(&self, org_roles: &HashMap<String, Role>, role_ids: &Vec<String>, principal: &mut Principal) {
        for role_id in role_ids {
            if let Some(role) = org_roles.get(role_id) {
                principal.roles.insert(role_id.clone(), role.clone());
                if let Some(ref parent_id) = role.parent_id {
                    self.add_roles(org_roles, &vec![parent_id.clone()], principal);
                }
            } else {
                warn!("Failed to add role with id {} for {}-{} because it's not mapped to org while populating principal", role_id, principal.username, principal.id);
            }
        }
    }

    fn populate_principal(&self, ctx: &SecurityContext, realm_id: &str, principal: &mut Principal) {
        // populate roles directly map to principal
        let org_roles = self.get_roles_by_org(ctx, principal.organization_id.as_str());
        self.add_roles(&org_roles, &self.role_roleable_repository.get_role_ids_by_principal(principal.id.as_str()), principal);

        // Checking groups
        let org_groups = self.get_groups_by_org(ctx, principal.organization_id.as_str());
        for group_id in &self.group_repository.get_group_ids_by_principal(principal.id.as_str()) {
            // populate roles indirectly map to group
            self.add_roles(&org_roles, &self.role_roleable_repository.get_role_ids_by_group(group_id.clone()), principal);
            //
            // Adding groups
            if let Some(group) = org_groups.get(group_id) {
                principal.groups.insert(group.id.clone(), group.clone());
            } else {
                warn!("Failed to find group for id {} for {}-{} while populating principal", group_id, principal.username, principal.id);
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
                if claim_id_scopes.len() > 0 && (cc.scope.len() > 0 || cc.claim_constraints.len() > 0) && claim_id_scopes.get(&format!("{}_{}", claim.id, cc.scope)) == None {
                    warn!("Found different or missing role scope/constraints than what was set in policy principal claim: {:?}, org claim: {:?}, all org claims: {:?}", cc, claim, org_claim_claimables);
                } else {
                    principal.claims.push(ClaimClaimable::Role(claim.clone(), realm_id.to_string(), cc.claimable_id.clone(), cc.scope.clone(), cc.claim_constraints.clone()));
                }
            } else {
                warn!("Failed to find claim for id {} - principal {}-{} while populating principal", cc.claim_id, principal.username, principal.id);
            }
        }

        // Find claims mapped directly to principal
        for cc in &self.claim_claimable_repository.get_by_principal(principal.id.clone()) {
            if let Some(claim) = claims_by_id.get(&cc.claim_id) {
                if claim_id_scopes.len() > 0 && (cc.scope.len() > 0 || cc.claim_constraints.len() > 0) && claim_id_scopes.get(&format!("{}_{}", claim.id, cc.scope)) == None {
                    warn!("Found different or missing principal scope/constraints than what was set in policy {:?} - {:?}", claim, cc);
                } else {
                    principal.claims.push(ClaimClaimable::Principal(claim.clone(), realm_id.to_string(), cc.claimable_id.clone(), cc.scope.clone(), cc.claim_constraints.clone()));
                }
            } else {
                warn!("Failed to find claim for id {} - principal {}-{} while populating principal", cc.claim_id, principal.username, principal.id);
            }
        }

        // Creating resources
        let mut resource_ids = vec![];
        for cc in &principal.claims {
            match cc {
                ClaimClaimable::Role(claim, _, _, _, _) => resource_ids.push(claim.resource_id.clone()),
                ClaimClaimable::Principal(claim, _, _, _, _) => resource_ids.push(claim.resource_id.clone()),
                _ => (),
            };
        }
        principal.resources = self.resource_repository.get_by_ids(resource_ids).iter().map(|r| Resource::from(r)).collect::<Vec<Resource>>();
    }

    ////////////////////////////////// Group CRUD OPERATIONS //////////////////////////////
    /// Creates group with parent
    pub fn new_group_with_parent(&self, ctx: &SecurityContext, org: &Organization, parent: &Group, name: &str) -> Result<Group, RbacError> {
        self.save_group(ctx, &Group::new("".into(), org.id.as_str(), name, None, Some(parent.id.clone())))
    }

    /// Creates group
    pub fn new_group_with(&self, ctx: &SecurityContext, org: &Organization, name: &str) -> Result<Group, RbacError> {
        self.save_group(ctx, &Group::new("".into(), org.id.as_str(), name, None, None))
    }

    /// Creates group
    pub fn save_group(&self, ctx: &SecurityContext, group: &Group) -> Result<Group, RbacError> {
        match self.group_repository.get(group.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.parent_id = group.parent_id.clone();
                db_obj.description = group.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.group_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating group {:?}", db_obj), "UPDATE");
                info!("Updating group {:?}", db_obj);
                Ok(Group::from(&db_obj))
            }
            None => {
                let mut db_obj = group.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.group_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Creating new group {:?}", db_obj), "CREATE");
                info!("Creating group {:?}", db_obj);
                Ok(Group::from(&db_obj))
            }
        }
    }

    /// Returns all groups for given organization
    pub fn get_groups_by_org(&self, _ctx: &SecurityContext, organization_id: &str) -> HashMap<String, Group> {
        let mut groups = HashMap::new();
        for pgroup in &self.group_repository.get_by_org(organization_id) {
            let group = Group::from(pgroup); 
            groups.insert(group.id.clone(), group);
        }
        groups
    }

    /// Retrieves group by id from the database
    pub fn get_group(&self, _ctx: &SecurityContext, group_id: &str) -> Option<Group> {
        match self.group_repository.get(group_id) {
            Some(group) => Some(Group::from(&group)),
            None => None,
        }
    }

    /// Adds principal group
    pub fn map_principal_to_group(&self, ctx: &SecurityContext, principal: &Principal, group: &Group) -> Option<diesel::result::Error> {
        self.add_principal_to_group(ctx, group.id.as_str(), principal.id.as_str())
    }

    /// Adds principal to group
    pub fn add_principal_to_group(&self, ctx: &SecurityContext, group_id: &str, principal_id: &str) -> Option<diesel::result::Error> {
        let gp = PGroupPrincipal::new(group_id, principal_id);
        if let Some(err) = self.group_principal_repository.create(&gp) {
            Some(err)
        } else {
            self.audit(ctx, format!("Adding principal to group {:?}", gp), "CREATE");
            None
        }
    }

    /// Removes principal from group
    pub fn unmap_principal_from_group(&self, ctx: &SecurityContext, principal: &Principal, group: &Group) -> bool {
        self.remove_principal_from_group(ctx, group.id.as_str(), principal.id.as_str())
    }

    /// Removes principal from group
    pub fn remove_principal_from_group(&self, ctx: &SecurityContext, group_id: &str, principal_id: &str) -> bool {
        let gp = PGroupPrincipal::new(group_id, principal_id);
        if self.group_principal_repository.delete(&gp) {
            self.audit(ctx, format!("Removing principal from group {:?}", gp), "DELETE");
            true
        } else {
            false
        }
    }

    ////////////////////////////////// Role CRUD OPERATIONS //////////////////////////////
    /// Creates role with parent
    pub fn new_role_with_parent(&self, ctx: &SecurityContext, realm: &SecurityRealm, org: &Organization, parent: &Role, name: &str) -> Result<Role, RbacError> {
        self.save_role(ctx, &Role::new("".into(), realm.id.as_str(), org.id.as_str(), name, None, Some(parent.id.clone())))
    }

    /// Creates role
    pub fn new_role_with(&self, ctx: &SecurityContext, realm: &SecurityRealm, org: &Organization, name: &str) -> Result<Role, RbacError> {
        self.save_role(ctx, &Role::new("".into(), realm.id.as_str(), org.id.as_str(), name, None, None))
    }

    /// Creates role
    pub fn save_role(&self, ctx: &SecurityContext, role: &Role) -> Result<Role, RbacError> {
        match self.role_repository.get(role.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.parent_id = role.parent_id.clone();
                db_obj.description = role.description.clone();
                //db_obj.role_constraints = role.role_constraints.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.role_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating role {:?}", db_obj), "UPDATE");
                info!("Updating role {:?}", db_obj);
                Ok(Role::from(&db_obj))
            }
            None => {
                let mut db_obj = role.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.role_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Creating new role {:?}", db_obj), "CREATE");
                info!("Creating role {:?}", db_obj);
                Ok(Role::from(&db_obj))
            }
        }
    }

    /// Returns all roles for given organization
    pub fn get_roles_by_org(&self, _ctx: &SecurityContext, organization_id: &str) -> HashMap<String, Role> {
        let mut roles = HashMap::new();
        for prole in &self.role_repository.get_by_org(organization_id) {
            let role = Role::from(prole); 
            roles.insert(role.id.clone(), role);
        }
        roles
    }

    /// Retrieves role by id from the database
    pub fn get_role(&self, _ctx: &SecurityContext, role_id: &str) -> Option<Role> {
        match self.role_repository.get(role_id) {
            Some(role) => Some(Role::from(&role)),
            None => None,
        }
    }

    /// Adds role to principal
    pub fn map_principal_to_role(&self, ctx: &SecurityContext, principal: &Principal, role: &Role) -> Option<diesel::result::Error> {
        self.add_principal_to_role(ctx, role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Adds principal to role
    pub fn add_principal_to_role(&self, ctx: &SecurityContext, role_id: &str, principal_id: &str, constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Option<diesel::result::Error> {
        let rr = PRoleRoleable::new(role_id, principal_id, Constants::Principal.to_string().as_str(), constraints, effective_at, expired_at);
        if let Some(err) = self.role_roleable_repository.create(&rr) {
            Some(err)
        } else {
            self.audit(ctx, format!("Adding principal to role {:?}", rr), "CREATE");
            None
        }
    }

    /// Removes principal from role
    pub fn unmap_principal_from_role(&self, ctx: &SecurityContext, principal: &Principal, role: &Role) -> bool {
        self.remove_principal_from_role(ctx, role.id.as_str(), principal.id.as_str())
    }

    /// Removes principal from role
    pub fn remove_principal_from_role(&self, ctx: &SecurityContext, role_id: &str, principal_id: &str) -> bool {
        let rr = PRoleRoleable::new(role_id, principal_id, Constants::Principal.to_string().as_str(), "", Utc::now().naive_utc(), Utc::now().naive_utc());
        if self.role_roleable_repository.delete(&rr) {
            self.audit(ctx, format!("Removing principal from role {:?}", rr), "DELETE");
            true
        } else {
            false
        }
    }

    /// Adds group to role
    pub fn map_group_to_role(&self, ctx: &SecurityContext, group: &Group, role: &Role, constraints: &str) -> Option<diesel::result::Error> {
        self.add_group_to_role(ctx, role.id.as_str(), group.id.as_str(), constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Adds group to role
    pub fn add_group_to_role(&self, ctx: &SecurityContext, role_id: &str, group_id: &str, constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Option<diesel::result::Error> {
        let rr = PRoleRoleable::new(role_id, group_id, Constants::Group.to_string().as_str(), constraints, effective_at, expired_at);
        if let Some(err) = self.role_roleable_repository.create(&rr) {
            Some(err)
        } else {
            self.audit(ctx, format!("Adding group to role {:?}", rr), "DELETE");
            None
        }
    }

    /// Removes group from role
    pub fn unmap_group_from_role(&self, ctx: &SecurityContext, group: &Group, role: &Role) -> bool {
        self.remove_group_from_role(ctx, role.id.as_str(), group.id.as_str())
    }

    /// Removes group from role
    pub fn remove_group_from_role(&self, ctx: &SecurityContext, role_id: &str, group_id: &str) -> bool {
        let rr = PRoleRoleable::new(role_id, group_id, Constants::Group.to_string().as_str(), "", Utc::now().naive_utc(), Utc::now().naive_utc());
        if self.role_roleable_repository.delete(&rr) {
            self.audit(ctx, format!("Removing group from role {:?}", rr), "DELETE");
            true
        } else {
            false
        }
    }

    ////////////////////////////////// RESOURCE CRUD OPERATIONS //////////////////////////////
    /// Creates resource
    pub fn new_resource_with(&self, ctx: &SecurityContext, realm: &SecurityRealm, resource_name: &str) -> Result<Resource, RbacError> {
        self.save_resource(ctx, &Resource::new("", realm.id.as_str(), resource_name, None, Some("(CREATE|READ|UPDATE|DELETE)".to_string())))
    }

    /// Creates resource
    pub fn save_resource(&self, ctx: &SecurityContext, resource: &Resource) -> Result<Resource, RbacError> {
        match self.resource_repository.get(resource.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.allowable_actions = resource.allowable_actions .clone();
                db_obj.description = resource.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.resource_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating resource {:?}", db_obj), "UPDATE");
                info!("Updating resource {:?}", db_obj);
                Ok(Resource::from(&db_obj))
            }
            None => {
                let mut db_obj = resource.to();
                //db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.id = resource.resource_name.clone();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.resource_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Adding resource {:?}", db_obj), "CREATE");
                info!("Creating resource {:?}", db_obj);
                Ok(Resource::from(&db_obj))
            }
        }
    }

    /// Returns all resources for given security realm
    pub fn get_resources_by_realm(&self, _ctx: &SecurityContext, realm_id: &str) -> HashMap<String, Resource> {
        let mut resources = HashMap::new();
        for presource in &self.resource_repository.get_by_realm(realm_id) {
            let resource = Resource::from(presource); 
            resources.insert(resource.id.clone(), resource);
        }
        resources
    }

    /// Returns all claims for given security realm
    pub fn get_claims_by_realm(&self, _ctx: &SecurityContext, realm_id: &str) -> HashMap<String, Claim> {
        let mut claims = HashMap::new();
        for pclaim in &self.claim_repository.get_by_realm(realm_id) {
            let claim = Claim::from(pclaim); 
            claims.insert(claim.id.clone(), claim);
        }
        claims
    }

    pub fn get_claims_by_policy(&self, ctx: &SecurityContext, realm_id: &str, license_policy_id: &str) -> HashMap<String, Claim> {
        let all_claims = self.get_claims_by_realm(ctx, realm_id);
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


    /// Returns all resources for given claims
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
        for (_, resource) in self.get_resources_by_realm(ctx, realm_id) {
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

    /// Retrieves resource by id from the database
    pub fn get_resource(&self, _ctx: &SecurityContext, resource_id: &str) -> Option<Resource> {
        match self.resource_repository.get(resource_id) {
            Some(resource) => Some(Resource::from(&resource)),
            None => None,
        }
    }

    ////////////////////////////////// RESOURCE INSTANCE CRUD OPERATIONS //////////////////////////////
    /// Creates resource_instance
    pub fn new_resource_instance_with(&self, ctx: &SecurityContext, resource: &Resource, policy: &LicensePolicy, scope: &str, ref_id: &str, status: Status) -> Result<ResourceInstance, RbacError> {
        self.save_resource_instance(ctx, &ResourceInstance::new("", resource.id.as_str(), policy.id.as_str(), scope, ref_id, status.to_string().as_str(), None))
    }

    /// Creates resource_instance
    pub fn save_resource_instance(&self, ctx: &SecurityContext, instance: &ResourceInstance) -> Result<ResourceInstance, RbacError> {
        if let Some(quota) = self.resource_quota_repository.get_by_resource(instance.resource_id.as_str(), instance.scope.as_str()).first() {
            let count = self.resource_instance_repository.count_by_resource(instance.resource_id.as_str(), instance.scope.as_str());
            if count >= quota.max_value as i64 {
                warn!("Reached limit for {:?}  -- {:?}", instance, quota);
                return Err(RbacError::QuotaExceeded(format!("Reached limit for {:?} -- {:?}", instance, quota)));
            }
        } else {
            warn!("Reached limit for {:?}", instance);
            return Err(RbacError::QuotaExceeded(format!("Reached limit for {:?} -- quota not found", instance)));
        }
        //
        match self.resource_instance_repository.get(instance.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.status = instance.status.clone();
                db_obj.description = instance.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.resource_instance_repository.update(&db_obj) {
                    warn!("Failed to update resource instance {}", err.to_string());
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating resource instance {:?}", db_obj), "UPDATE");
                info!("Updating resource_instance {:?}", db_obj);
                Ok(ResourceInstance::from(&db_obj))
            }
            None => {
                let mut db_obj = instance.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.resource_instance_repository.create(&db_obj) {
                    warn!("Failed to create resource instance {}", err.to_string());
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Adding new resource instance {:?}", db_obj), "CREATE");
                info!("Creating resource_instance {:?}", db_obj);
                Ok(ResourceInstance::from(&db_obj))
            }
        }
    }

    /// Retrieves resource_instance by id from the database
    pub fn get_resource_instance(&self, _ctx: &SecurityContext, id: &str) -> Option<ResourceInstance> {
        match self.resource_instance_repository.get(id) {
            Some(instance) => Some(ResourceInstance::from(&instance)),
            None => None,
        }
    }

    ////////////////////////////////// RESOURCE quota CRUD OPERATIONS //////////////////////////////
    /// Creates resource_quota
    pub fn new_resource_quota_with(&self, ctx: &SecurityContext, resource: &Resource, policy: &LicensePolicy, scope: &str, max_value: i32) -> Result<ResourceQuota, RbacError> {
        self.save_resource_quota(ctx, &ResourceQuota::new("", resource.id.as_str(), policy.id.as_str(), scope, max_value, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)))
    }

    /// Creates resource_quota
    pub fn save_resource_quota(&self, ctx: &SecurityContext, quota: &ResourceQuota) -> Result<ResourceQuota, RbacError> {
        match self.resource_quota_repository.get(quota.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.max_value = quota.max_value.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.resource_quota_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating new resource quota {:?}", db_obj), "UPDATE");
                info!("Updating resource_quota {:?}", db_obj);
                Ok(ResourceQuota::from(&db_obj))
            }
            None => {
                let mut db_obj = quota.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.resource_quota_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Adding new resource quota {:?}", db_obj), "CREATE");
                info!("Creating resource_quota {:?}", db_obj);
                Ok(ResourceQuota::from(&db_obj))
            }
        }
    }

    /// Retrieves resource_quota by id from the database
    pub fn get_resource_quota(&self, _ctx: &SecurityContext, id: &str) -> Option<ResourceQuota> {
        match self.resource_quota_repository.get(id) {
            Some(quota) => Some(ResourceQuota::from(&quota)),
            None => None,
        }
    }

    ////////////////////////////////// CLAIM quota CRUD OPERATIONS //////////////////////////////
    /// Creates claim
    pub fn new_claim_with(&self, ctx: &SecurityContext, realm: &SecurityRealm, resource: &Resource, action: &str) -> Result<Claim, RbacError> {
        self.save_claim(ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), action, Constants::Allow.to_string().as_str(), None))
    }

    /// Creates claim
    pub fn save_claim(&self, ctx: &SecurityContext, claim: &Claim) -> Result<Claim, RbacError> {
        match self.claim_repository.get(claim.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.action = claim.action.clone();
                db_obj.effect = claim.effect.clone();
                db_obj.description = claim.description.clone();
                //db_obj.scope = claim.scope.clone();
                //db_obj.claim_constraints = claim.claim_constraints.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.claim_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating claim {:?}", db_obj), "CREATE");
                info!("Updating claim {:?}", db_obj);
                Ok(Claim::from(&db_obj))
            }
            None => {
                let mut db_obj = claim.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.claim_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Adding new claim {:?}", db_obj), "CREATE");
                info!("Creating claim {:?}", db_obj);
                Ok(Claim::from(&db_obj))
            }
        }
    }

    /// Retrieves claim by id from the database
    pub fn get_claim(&self, _ctx: &SecurityContext, claim_id: &str) -> Option<Claim> {
        match self.claim_repository.get(claim_id) {
            Some(claim) => Some(Claim::from(&claim)),
            None => None,
        }
    }


    /// Adds role to claim
    pub fn map_role_to_claim(&self, ctx: &SecurityContext, role: &Role, claim: &Claim, scope: &str, constraints: &str) -> Option<diesel::result::Error> {
        self.add_role_to_claim(ctx, role.id.as_str(), claim.id.as_str(), scope, constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Adds role to claim
    pub fn add_role_to_claim(&self, ctx: &SecurityContext, role_id: &str, claim_id: &str, scope: &str, claim_constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Option<diesel::result::Error> {
        let cc = PClaimClaimable::new(claim_id, role_id, Constants::Role.to_string().as_str(), scope, claim_constraints, effective_at, expired_at);
        if let Some(err) = self.claim_claimable_repository.create(&cc) {
            Some(err)
        } else {
            self.audit(ctx, format!("Adding role to claim {:?}", cc), "CREATE");
            None
        }
    }

    /// Removes role from claim
    pub fn unmap_role_from_claim(&self, ctx: &SecurityContext, role: &Role, claim: &Claim) -> bool {
        self.remove_role_from_claim(ctx, role.id.as_str(), claim.id.as_str())
    }

    /// Removes role from claim
    pub fn remove_role_from_claim(&self, ctx: &SecurityContext, role_id: &str, claim_id: &str) -> bool {
        let cc = PClaimClaimable::new(claim_id, role_id, Constants::Role.to_string().as_str(), "", "", Utc::now().naive_utc(), Utc::now().naive_utc());
        if self.claim_claimable_repository.delete(&cc) {
            self.audit(ctx, format!("Removing role from claim {:?}", cc), "DELETE");
            true
        } else {
            false
        }
    }

    /// Adds principal to claim
    pub fn map_principal_to_claim(&self, ctx: &SecurityContext, principal: &Principal, claim: &Claim, scope: &str, constraints: &str) -> Option<diesel::result::Error> {
        self.add_principal_to_claim(ctx, principal.id.as_str(), claim.id.as_str(), scope, constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Adds principal to claim
    pub fn add_principal_to_claim(&self, ctx: &SecurityContext, principal_id: &str, claim_id: &str, scope: &str, claim_constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Option<diesel::result::Error> {
        let cc = PClaimClaimable::new(claim_id, principal_id, Constants::Principal.to_string().as_str(), scope, claim_constraints, effective_at, expired_at);
        if let Some(err) = self.claim_claimable_repository.create(&cc) {
            Some(err)
        } else {
            self.audit(ctx, format!("Adding principal to claim {:?}", cc), "CREATE");
            None
        }
    }

    /// Removes principal from claim
    pub fn unmap_principal_from_claim(&self, ctx: &SecurityContext, principal: &Principal, claim: &Claim) -> bool {
        self.remove_principal_from_claim(ctx, principal.id.as_str(), claim.id.as_str())
    }

    /// Removes principal from claim
    pub fn remove_principal_from_claim(&self, ctx: &SecurityContext, principal_id: &str, claim_id: &str) -> bool {
        let cc = PClaimClaimable::new(claim_id, principal_id, Constants::Principal.to_string().as_str(), "", "", Utc::now().naive_utc(), Utc::now().naive_utc());
        if self.claim_claimable_repository.delete(&cc) {
            self.audit(ctx, format!("Removing principal from claim {:?}", cc), "DELETE");
            true
        } else {
            false
        }
    }


    /// Adds license-policy to claim
    pub fn map_license_policy_to_claim(&self, ctx: &SecurityContext, policy: &LicensePolicy, claim: &Claim, scope: &str, constraints: &str) -> Option<diesel::result::Error> {
        self.add_license_policy_to_claim(ctx, policy.id.as_str(), claim.id.as_str(), scope, constraints, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
    }

    /// Adds license-policy to claim
    pub fn add_license_policy_to_claim(&self, ctx: &SecurityContext, license_policy_id: &str, claim_id: &str, scope: &str, constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Option<diesel::result::Error> {
        let cc = PClaimClaimable::new(claim_id, license_policy_id, Constants::LicensePolicy.to_string().as_str(), scope, constraints, effective_at, expired_at);
        if let Some(err) = self.claim_claimable_repository.create(&cc) {
            Some(err)
        } else {
            self.audit(ctx, format!("Adding claim to license-policy {:?}", cc), "CREATE");
            None
        }
    }

    /// Removes license-policy from claim
    pub fn unmap_license_policy_from_claim(&self, ctx: &SecurityContext, policy: &LicensePolicy, claim: &Claim) -> bool {
        self.remove_license_policy_from_claim(ctx, policy.id.as_str(), claim.id.as_str())
    }

    /// Removes license-policy from claim
    pub fn remove_license_policy_from_claim(&self, ctx: &SecurityContext, license_policy_id: &str, claim_id: &str) -> bool {
        let cc = PClaimClaimable::new(claim_id, license_policy_id, Constants::LicensePolicy.to_string().as_str(), "", "", Utc::now().naive_utc(), Utc::now().naive_utc());
        if self.claim_claimable_repository.delete(&cc) {
            self.audit(ctx, format!("Removing claim from license-policy {:?}", cc), "DELETE");
            true
        } else {
            false
        }
    }


    ////////////////////////////////// LICENSE POLICY CRUD OPERATIONS //////////////////////////////
    /// Adds license-policy
    pub fn new_license_policy(&self, ctx: &SecurityContext, org: &Organization) -> Result<LicensePolicy, RbacError> {
        self.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)))
    }

    /// Adds license-policy
    pub fn save_license_policy(&self, ctx: &SecurityContext, policy: &LicensePolicy) -> Result<LicensePolicy, RbacError> {
        match self.license_policy_repository.get(policy.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.description = policy.description.clone();
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.license_policy_repository.update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updating license-policy {:?}", policy), "UPDATE");
                info!("Updating license policy {:?}", db_obj);
                if self.license_policy_repository.get_by_org(policy.organization_id.as_str()).len() > 1 {
                    warn!("Multiple license policies exist for {:?}", policy);
                }
                Ok(LicensePolicy::from(&db_obj))
            }
            None => {
                let mut db_obj = policy.to();
                db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
                db_obj.created_at = Utc::now().naive_utc();
                db_obj.created_by = Some(ctx.principal_id.clone());
                db_obj.updated_at = Utc::now().naive_utc();
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Some(err) = self.license_policy_repository.create(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Adding new license-policy {:?}", policy), "CREATE");
                info!("Creating license policy {:?}", db_obj);
                if self.license_policy_repository.get_by_org(policy.organization_id.as_str()).len() > 1 {
                    warn!("Multiple license policies exist for {:?}", policy);
                }
                Ok(LicensePolicy::from(&db_obj))
            }
        }
    }

    /// Retrieves license-policy by id from the database
    pub fn get_license_policy(&self, _ctx: &SecurityContext, policy_id: &str) -> Option<LicensePolicy> {
        match self.license_policy_repository.get(policy_id) {
            Some(policy) => Some(LicensePolicy::from(&policy)),
            None => None,
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
        self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
    }
}

#[cfg(test)]
mod tests {
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::security::context::SecurityContext;
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
    fn test_get_save_realm() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "desc")).unwrap();
        let realm_str = format!("{:?}", realm);
        let loaded = mgr.get_realm(&ctx, realm.id.as_str()).unwrap();
        assert_eq!(realm_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_get_save_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let org_str = format!("{:?}", org);
        let loaded = mgr.get_org(&ctx, realm.id.as_str(), org.id.as_str()).unwrap();
        assert_eq!(org_str, format!("{:?}", loaded));
        mgr.get_license_policy(&ctx, license_policy.id.as_str()).unwrap();
    }

    #[test]
    fn test_get_save_principal() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        //
        let principal = mgr.save_principal(&ctx, &Principal::new("", org.id.as_str(), "myusername-principal", Some("desc".to_string()))).unwrap();
        let principal_str = format!("{:?}", principal);
        let loaded = mgr.get_principal(&ctx, realm.id.as_str(), principal.id.as_str()).unwrap();
        assert_eq!(principal_str, format!("{:?}", loaded));
        mgr.get_license_policy(&ctx, license_policy.id.as_str()).unwrap();
    }

    #[test]
    fn test_get_save_group() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        //
        let group = mgr.save_group(&ctx, &Group::new("", org.id.as_str(), "mygroup", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let group_str = format!("{:?}", group);
        //
        let loaded = mgr.get_group(&ctx, group.id.as_str()).unwrap();
        assert_eq!(group_str, format!("{:?}", loaded));

        let principal = mgr.save_principal(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();

        assert_eq!(None, mgr.add_principal_to_group(&ctx, group.id.as_str(), principal.id.as_str()));
        assert_eq!(true, mgr.remove_principal_from_group(&ctx, group.id.as_str(), principal.id.as_str()));
        mgr.get_license_policy(&ctx, license_policy.id.as_str()).unwrap();
    }

    #[test]
    fn test_get_save_role() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let _license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        //
        let role = mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "myrole", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let loaded= mgr.get_role(&ctx, role.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", role), format!("{:?}", loaded));

        let principal = mgr.save_principal(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();

        assert_eq!(None, mgr.add_principal_to_role(&ctx, role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(true, mgr.remove_principal_from_role(&ctx, role.id.as_str(), principal.id.as_str()));

    }

    #[test]
    fn test_get_save_claim() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let _license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let role = mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "myrole", None, None)).unwrap();
        let principal = mgr.save_principal(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();
        assert_eq!(None, mgr.add_principal_to_role(&ctx, role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));

        let resource = mgr.save_resource(&ctx, &Resource::new("", realm.id.as_str(), "report", None, None)).unwrap();
        let claim1 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", Constants::Deny.to_string().as_str(), None)).unwrap();
        let claim2 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", Constants::Allow.to_string().as_str(), None)).unwrap();
        assert_eq!(None, mgr.add_role_to_claim(&ctx, role.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(None, mgr.add_role_to_claim(&ctx, role.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(true, mgr.remove_role_from_claim(&ctx, role.id.as_str(), claim1.id.as_str()));
        assert_eq!(true, mgr.remove_role_from_claim(&ctx, role.id.as_str(), claim2.id.as_str()));

        assert_eq!(None, mgr.add_principal_to_claim(&ctx, principal.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(None, mgr.add_principal_to_claim(&ctx, principal.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(true, mgr.remove_principal_from_claim(&ctx, principal.id.as_str(), claim1.id.as_str()));
        assert_eq!(true, mgr.remove_principal_from_claim(&ctx, principal.id.as_str(), claim2.id.as_str()));

        let loaded1 = mgr.get_claim(&ctx, claim1.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", claim1), format!("{:?}", loaded1));

        let loaded2 = mgr.get_claim(&ctx, claim2.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", claim2), format!("{:?}", loaded2));
    }

    #[test]
    fn test_get_save_resources() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();

        let resource = mgr.save_resource(&ctx, &Resource::new("", realm.id.as_str(), "report", None, None)).unwrap();
        let loaded = mgr.get_resource(&ctx, resource.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", resource), format!("{:?}", loaded));

        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();

        let policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let _ = mgr.save_resource_quota(&ctx, &ResourceQuota::new("", resource.id.as_str(), policy.id.as_str(), "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let instance = mgr.save_resource_instance(&ctx, &ResourceInstance::new("", resource.id.as_str(), "22", "", "refid", "INFLIGHT", Some("blah".to_string()))).unwrap();

        let loaded = mgr.get_resource_instance(&ctx, instance.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", instance), format!("{:?}", loaded));

        let quota = mgr.save_resource_quota(&ctx, &ResourceQuota::new("", resource.id.as_str(), "22", "", 22, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let loaded = mgr.get_resource_quota(&ctx, quota.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", quota), format!("{:?}", loaded));
    }

    #[test]
    fn test_get_save_license_policy() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let resource = mgr.save_resource(&ctx, &Resource::new("", realm.id.as_str(), "report", None, None)).unwrap();
        let claim1 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", "allow", None)).unwrap();
        let claim2 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", "allow", None)).unwrap();

        assert_eq!(None, mgr.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(None, mgr.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        assert_eq!(true, mgr.remove_license_policy_from_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str()));
        assert_eq!(true, mgr.remove_license_policy_from_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str()));

        let loaded = mgr.get_license_policy(&ctx, license_policy.id.as_str()).unwrap();
        assert_eq!(format!("{:?}", license_policy), format!("{:?}", loaded));
    }

    #[test]
    fn test_populate_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "plexobject", "url", None)).unwrap();
        let license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "admin", None, None)).unwrap();
        mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "employee", None, None)).unwrap();
        mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "manager", None, None)).unwrap();
        mgr.save_group(&ctx, &Group::new("", org.id.as_str(), "default", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        mgr.save_group(&ctx, &Group::new("", org.id.as_str(), "devops", Some("desc".to_string()), Some("parent".to_string()))).unwrap();

        let resource = mgr.save_resource(&ctx, &Resource::new("", realm.id.as_str(), "report", None, None)).unwrap();
        let claim1 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", "allow", None)).unwrap();
        let claim2 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", "allow", None)).unwrap();
        mgr.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        mgr.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        let loaded = mgr.get_org(&ctx, realm.id.as_str(), org.id.as_str()).unwrap();
        assert_eq!(3, loaded.roles.len());
        assert_eq!(2, loaded.groups.len());
        assert_eq!(2, loaded.claims.len());
    }

    #[test]
    fn test_populate_principal() {
        let ctx = SecurityContext::new("myorg", "myid");
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        let realm = mgr.save_realm(&ctx, &SecurityRealm::new("myrealm", "")).unwrap();
        let org = mgr.save_org(&ctx, &Organization::new("", None, "plexobject", "url", None)).unwrap();
        let license_policy = mgr.save_license_policy(&ctx, &LicensePolicy::new("", org.id.as_str(), "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let org_employee_role = mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "org_employee", None, None)).unwrap();
        let org_manager_role = mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "org_manager", None, None)).unwrap();

        let _default_group_employee = mgr.save_role(&ctx, &Role::new("", realm.id.as_str(), org.id.as_str(), "default_group_employee", None, None)).unwrap();

        let principal = mgr.save_principal(&ctx, &Principal::new("", org.id.as_str(), "myusername", None)).unwrap();
        let default_group = mgr.save_group(&ctx, &Group::new("", org.id.as_str(), "default", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        mgr.add_principal_to_group(&ctx, default_group.id.as_str(), principal.id.as_str());

        mgr.add_principal_to_role(&ctx, org_manager_role.id.as_str(), principal.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        mgr.add_group_to_role(&ctx, org_employee_role.id.as_str(), default_group.id.as_str(), "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));

        let resource = mgr.save_resource(&ctx, &Resource::new("", realm.id.as_str(), "report", None, None)).unwrap();
        let claim1 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "READ", "Allow", None)).unwrap();
        let claim2 = mgr.save_claim(&ctx, &Claim::new("", realm.id.as_str(), resource.id.as_str(), "UPDATE", "Allow", None)).unwrap();
        mgr.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim1.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        mgr.add_license_policy_to_claim(&ctx, license_policy.id.as_str(), claim2.id.as_str(), "", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        let loaded = mgr.get_principal(&ctx, realm.id.as_str(), principal.id.as_str()).unwrap();
        assert_eq!(2, loaded.roles.len());
        assert_eq!(1, loaded.groups.len());
        assert_eq!(true, mgr.remove_group_from_role(&ctx, org_employee_role.id.as_str(), default_group.id.as_str()));
    }

    #[test]
    fn test_banking() {
        init();

        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "banking").unwrap();

        // Creating organization
        let org = mgr.new_org_with(&ctx, "bank-of-flakes").unwrap();

        // Creating Users
        let tom = mgr.new_principal_with(&ctx, &org, "tom").unwrap();
        let cassy = mgr.new_principal_with(&ctx, &org, "cassy").unwrap();
        let ali = mgr.new_principal_with(&ctx, &org, "ali").unwrap();
        let mike = mgr.new_principal_with(&ctx, &org, "mike").unwrap();
        let larry = mgr.new_principal_with(&ctx, &org, "larry").unwrap();

        // Creating Roles
        let employee = mgr.new_role_with(&ctx, &realm, &org, "Employee").unwrap();
        let teller = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Teller").unwrap();
        let csr = mgr.new_role_with_parent(&ctx, &realm, &org, &teller, "CSR").unwrap();
        let accountant = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Accountant").unwrap();
        let accountant_manager = mgr.new_role_with_parent(&ctx, &realm, &org, &accountant, "AccountingManager").unwrap();
        let loan_officer = mgr.new_role_with_parent(&ctx, &realm, &org, &accountant_manager, "LoanOfficer").unwrap();

        // Creating Resources
        let deposit_account = mgr.new_resource_with(&ctx, &realm, "DepositAccount").unwrap();
        let loan_account = mgr.new_resource_with(&ctx, &realm, "LoanAccount").unwrap();
        let general_ledger = mgr.new_resource_with(&ctx, &realm, "GeneralLedger").unwrap();
        let posting_rules = mgr.new_resource_with(&ctx, &realm, "GeneralLedgerPostingRules").unwrap();

        // Creating claims for resources
        let cd_deposit = mgr.new_claim_with(&ctx, &realm, &deposit_account, "(CREATE|DELETE)").unwrap();
        let ru_deposit = mgr.new_claim_with(&ctx, &realm, &deposit_account, "(READ|UPDATE)").unwrap();

        let cd_loan = mgr.new_claim_with(&ctx, &realm, &loan_account, "(CREATE|DELETE)").unwrap();
        let ru_loan = mgr.new_claim_with(&ctx, &realm, &loan_account, "(READ|UPDATE)").unwrap();

        let rd_ledger = mgr.new_claim_with(&ctx, &realm, &general_ledger, "(READ|CREATE)").unwrap();
        let r_glpr = mgr.new_claim_with(&ctx, &realm, &general_ledger, "(READ)").unwrap();

        let cud_glpr = mgr.new_claim_with(&ctx, &realm, &posting_rules, "(CREATE|UPDATE|DELETE)").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_principal_to_role(&ctx, &tom, &teller);
        mgr.map_principal_to_role(&ctx, &cassy, &csr);
        mgr.map_principal_to_role(&ctx, &ali, &accountant);
        mgr.map_principal_to_role(&ctx, &mike, &accountant_manager);
        mgr.map_principal_to_role(&ctx, &larry, &loan_officer);

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &teller, &ru_deposit, "U.S.", r#"employeeRegion == "Midwest""#);
        mgr.map_role_to_claim(&ctx, &csr, &cd_deposit, "U.S.", r#"employeeRegion == "Midwest""#);
        mgr.map_role_to_claim(&ctx, &accountant, &rd_ledger, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
        mgr.map_role_to_claim(&ctx, &accountant, &ru_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#);
        mgr.map_role_to_claim(&ctx, &accountant_manager, &cd_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#);
        mgr.map_role_to_claim(&ctx, &accountant_manager, &r_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
        mgr.map_role_to_claim(&ctx, &loan_officer, &cud_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);

        let security_mgr = SecurityManager::new(mgr);
        // Tom, the teller should be able to READ DepositAccount with scope U.S when employeeRegion
        // == Midwest
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Tom, the teller should not be able to READ DepositAccount with scope U.S when employeeRegion
        // == Northeast
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Northeast".to_string()));
        assert!(security_mgr.check(&req).is_err());

        // Tom, the teller should not be able to DELETE DepositAccount with scope U.S when employeeRegion
        // == Midwest
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert!(security_mgr.check(&req).is_err());

        // Cassy, the CSR should be able to DELETE DepositAccount with scope U.S when employeeRegion
        // == Midwest
        let mgr = factory.new_persistence_manager();
        let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Cassy, the CSR should be able to DELETE DepositAccount with scope U.K when employeeRegion
        // == Midwest
        let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.K.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert!(security_mgr.check(&req).is_err());

        // Ali, the Accountant should be able to READ GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::READ, "GeneralLedger", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Ali, the Accountant should not be able to READ GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear is in past
        req.context.add("ledgerYear", ValueWrapper::Int(2000));
        assert!(security_mgr.check(&req).is_err());

        // Ali, the Accountant should not be able to DELETE GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::DELETE, "GeneralLedger", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        assert!(security_mgr.check(&req).is_err());

        // Mike, the Accountant Manager should be able to DELETE GeneralLedger with scope U.S when employeeRegion
        // == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedger", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());


        // Mike, the Accountant Manager should not be able to post posting-rules of general-ledger with scope U.S 
        // when employeeRegion == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        req.context.add("accountBlance", ValueWrapper::Int(500));
        assert!(security_mgr.check(&req).is_err());

        // Larry, the Loan Officer should be able to post posting-rules of general-ledger with scope U.S 
        // when employeeRegion == Midwest AND ledgerYear == current_year()
        let mut req = PermissionRequest::new(realm.id.as_str(), larry.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
        req.context.add("accountBlance", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        mgr.unmap_role_from_claim(&ctx, &teller, &ru_deposit);
        mgr.unmap_role_from_claim(&ctx, &csr, &cd_deposit);
        mgr.unmap_role_from_claim(&ctx, &accountant, &rd_ledger);
        mgr.unmap_role_from_claim(&ctx, &accountant, &ru_loan);
        mgr.unmap_role_from_claim(&ctx, &accountant_manager, &cd_loan);
        mgr.unmap_role_from_claim(&ctx, &accountant_manager, &r_glpr);
        mgr.unmap_role_from_claim(&ctx, &loan_officer, &cud_glpr);

        mgr.unmap_role_from_claim(&ctx, &loan_officer, &cud_glpr);
        assert_eq!(7, mgr.get_claims_by_realm(&ctx, realm.id.as_str()).len());
        assert_eq!(7, mgr.get_claims_by_policy(&ctx, realm.id.as_str(), "99").len());

        mgr.unmap_principal_from_role(&ctx, &tom, &teller);
        mgr.unmap_principal_from_role(&ctx, &cassy, &csr);
        mgr.unmap_principal_from_role(&ctx, &ali, &accountant);
        mgr.unmap_principal_from_role(&ctx, &mike, &accountant_manager);
        mgr.unmap_principal_from_role(&ctx, &larry, &loan_officer);
    }

    #[test]
    fn test_expense_report_with_groups() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "hypatia").unwrap();

        // Creating organization
        let org = mgr.new_org_with(&ctx, "hypatia").unwrap();

        // Creating Groups
        let group_employee = mgr.new_group_with(&ctx, &org, "Employee").unwrap();
        let group_manager = mgr.new_group_with_parent(&ctx, &org, &group_employee, "Manager").unwrap();

        // Creating Users
        let tom = mgr.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = mgr.new_principal_with(&ctx, &org, "mike").unwrap();

        // Mapping users to groups
        mgr.map_principal_to_group(&ctx, &tom, &group_employee);
        mgr.map_principal_to_group(&ctx, &mike, &group_employee);
        mgr.map_principal_to_group(&ctx, &mike, &group_manager);

        // Creating Roles
        let employee = mgr.new_role_with(&ctx, &realm, &org, "Employee").unwrap();
        let manager = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Manager").unwrap();

        // Creating Resources
        let expense_report = mgr.new_resource_with(&ctx, &realm, "ExpenseReport").unwrap();

        // Creating claims for resources
        let submit_report = mgr.new_claim_with(&ctx, &realm, &expense_report, "SUBMIT").unwrap();
        let approve_report = mgr.new_claim_with(&ctx, &realm, &expense_report, "APPROVE").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_group_to_role(&ctx, &group_employee, &employee, "");
        mgr.map_group_to_role(&ctx, &group_manager, &manager, "");

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &employee, &submit_report, "U.S.", r#"amount < 10000"#);
        mgr.map_role_to_claim(&ctx, &manager, &approve_report, "U.S.", r#"amount < 10000"#);

        let security_mgr = SecurityManager::new(mgr);
        // Tom should be able to submit report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::SUBMIT, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Tom should not be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert!(security_mgr.check(&req).is_err());

        // Mike should be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        let mgr = factory.new_persistence_manager();
        mgr.unmap_principal_from_group(&ctx, &tom, &group_employee);
        mgr.unmap_principal_from_group(&ctx, &mike, &group_employee);
        mgr.unmap_principal_from_group(&ctx, &mike, &group_manager);

        mgr.unmap_group_from_role(&ctx, &group_employee, &employee);
        mgr.unmap_group_from_role(&ctx, &group_manager, &manager);
    }

    #[test]
    fn test_expense_report_with_direct_claim_to_principal() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "rainier").unwrap();

        // Creating organization
        let org = mgr.new_org_with(&ctx, "rainier").unwrap();

        // Creating Users
        let tom = mgr.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = mgr.new_principal_with(&ctx, &org, "mike").unwrap();

        // Creating Resources
        let expense_report = mgr.new_resource_with(&ctx, &realm, "ExpenseReport").unwrap();

        // Creating claims for resources
        let submit_report = mgr.new_claim_with(&ctx, &realm, &expense_report, "SUBMIT").unwrap();
        let approve_report = mgr.new_claim_with(&ctx, &realm, &expense_report, "APPROVE").unwrap();

        // Map claims to roles as follows:
        mgr.map_principal_to_claim(&ctx, &tom, &submit_report, "U.S.", r#"amount < 10000"#);
        mgr.map_principal_to_claim(&ctx, &mike, &approve_report, "U.S.", r#"amount < 10000"#);

        let security_mgr = SecurityManager::new(mgr);
        // Tom should be able to submit report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::SUBMIT, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Tom should not be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert!(security_mgr.check(&req).is_err());

        // Mike should be able to approve report
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
        req.context.add("amount", ValueWrapper::Int(1000));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        let mgr = factory.new_persistence_manager();
        mgr.unmap_principal_from_claim(&ctx, &tom, &submit_report);
        mgr.unmap_principal_from_claim(&ctx, &mike, &approve_report);
    }

    #[test]
    fn test_feature_flag_with_geo_fencing() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "ada").unwrap();

        // Creating organization
        let org = mgr.new_org_with(&ctx, "ada").unwrap();

        // Creating Users
        let tom = mgr.new_principal_with(&ctx, &org, "tom").unwrap();
        let mike = mgr.new_principal_with(&ctx, &org, "mike").unwrap();

        // Creating Roles
        let customer = mgr.new_role_with(&ctx, &realm, &org, "Customer").unwrap();
        let beta_customer = mgr.new_role_with_parent(&ctx, &realm, &org, &customer, "BetaCustomer").unwrap();

        // Creating Resources
        let feature = mgr.new_resource_with(&ctx, &realm, "Feature").unwrap();

        // Creating claims for resources
        let view = mgr.new_claim_with(&ctx, &realm, &feature, "VIEW").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_principal_to_role(&ctx, &tom, &customer);
        mgr.map_principal_to_role(&ctx, &mike, &beta_customer);

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 100"#);
        mgr.map_role_to_claim(&ctx, &beta_customer, &view, "UI::Flag::AdvancedReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 200"#);

        let security_mgr = SecurityManager::new(mgr);

        // Tom should be able to view basic report if he lives close to Seattle
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
        req.context.add("customer_lat", ValueWrapper::Float(46.879967));
        req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Tom should not be able to view basic report if he lives far from Seattle
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
        req.context.add("customer_lat", ValueWrapper::Float(37.3230));
        req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
        assert!(security_mgr.check(&req).is_err());

        // Tom should not be able to view advanced report
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        req.context.add("customer_lat", ValueWrapper::Float(46.879967));
        req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
        assert!(security_mgr.check(&req).is_err());

        // Mike should be able to view advanced report
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        req.context.add("customer_lat", ValueWrapper::Float(46.879967));
        req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Mike should not be able to view advanced report if he lives far from Seattle
        let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        req.context.add("customer_lat", ValueWrapper::Float(37.3230));
        req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
        assert!(security_mgr.check(&req).is_err());
    }

    #[test]
    fn test_license_policy() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "curie").unwrap();

        // Creating organization
        let freemium_org = mgr.new_org_with(&ctx, "Freeloader").unwrap();
        let paid_org = mgr.new_org_with(&ctx, "Moneymaker").unwrap();

        // Create license policies
        let freemium_policy = mgr.new_license_policy(&ctx, &freemium_org).unwrap();
        let paid_policy = mgr.new_license_policy(&ctx, &paid_org).unwrap();

        // Creating Users
        let freemium_frank = mgr.new_principal_with(&ctx, &freemium_org, "frank").unwrap();
        let money_matt = mgr.new_principal_with(&ctx, &paid_org, "matt").unwrap();

        // Creating Roles
        let customer = mgr.new_role_with(&ctx, &realm, &freemium_org, "Customer").unwrap();
        let paid_customer = mgr.new_role_with(&ctx, &realm, &paid_org, "PaidCustomer").unwrap();

        // Creating Resources
        let feature = mgr.new_resource_with(&ctx, &realm, "Feature").unwrap();

        // Creating claims for resources
        let view = mgr.new_claim_with(&ctx, &realm, &feature, "VIEW").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_principal_to_role(&ctx, &freemium_frank, &customer);
        mgr.map_principal_to_role(&ctx, &money_matt, &customer);
        mgr.map_principal_to_role(&ctx, &money_matt, &paid_customer);

        // Map claims to policies as follows:
        mgr.map_license_policy_to_claim(&ctx, &freemium_policy, &view, "UI::Flag::BasicReport", "");
        mgr.map_license_policy_to_claim(&ctx, &paid_policy, &view, "UI::Flag::AdvancedReport", "");

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", "");
        mgr.map_role_to_claim(&ctx, &paid_customer, &view, "UI::Flag::AdvancedReport", "");

        let security_mgr = SecurityManager::new(mgr);

        // Frank should be able to view basic report
        let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        let mgr = factory.new_persistence_manager();
        // Frank should not be able to view advanced report
        let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        assert!(security_mgr.check(&req).is_err());

        // Matt should be able to view advanced report
        let req = PermissionRequest::new(realm.id.as_str(), money_matt.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        mgr.unmap_license_policy_from_claim(&ctx, &freemium_policy, &view);
        mgr.unmap_license_policy_from_claim(&ctx, &paid_policy, &view);
    }

    #[test]
    fn test_app_report() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "SeeEye").unwrap();

        // Creating organization
        let org = mgr.new_org_with(&ctx, "SeeEye").unwrap();

        // Create license policies
        let policy = mgr.new_license_policy(&ctx, &org).unwrap();

        // Creating Users
        let dave = mgr.new_principal_with(&ctx, &org, "dave").unwrap();
        let qari = mgr.new_principal_with(&ctx, &org, "qari").unwrap();
        let ali = mgr.new_principal_with(&ctx, &org, "ali").unwrap();

        // Creating Roles
        let developer = mgr.new_role_with(&ctx, &realm, &org, "Developer").unwrap();
        let qa = mgr.new_role_with(&ctx, &realm, &org, "QA").unwrap();
        let admin = mgr.new_role_with_parent(&ctx, &realm, &org, &developer, "Admin").unwrap();

        // Creating Resources
        let app = mgr.new_resource_with(&ctx, &realm, "App").unwrap();

        // Creating claims for resources
        let submit_view = mgr.new_claim_with(&ctx, &realm, &app, "(SUBMIT|VIEW)").unwrap();
        let view = mgr.new_claim_with(&ctx, &realm, &app, "VIEW").unwrap();
        let create_delete = mgr.new_claim_with(&ctx, &realm, &app, "(CREATE|DELETE)").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_principal_to_role(&ctx, &dave, &developer);
        mgr.map_principal_to_role(&ctx, &qari, &qa);
        mgr.map_principal_to_role(&ctx, &ali, &admin);

        // Map claims to policies as follows:
        mgr.map_license_policy_to_claim(&ctx, &policy, &submit_view, "com.xyz.app", "appSize < 1000");
        mgr.map_license_policy_to_claim(&ctx, &policy, &view, "com.xyz.app", "appSize < 1000");
        mgr.map_license_policy_to_claim(&ctx, &policy, &create_delete, "com.xyz.app", "");

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &developer, &submit_view, "com.xyz.app", "appSize < 1000");
        mgr.map_role_to_claim(&ctx, &qa, &view, "com.xyz.app", "appSize < 1000");
        mgr.map_role_to_claim(&ctx, &admin, &create_delete, "com.xyz.app", "");

        let security_mgr = SecurityManager::new(mgr);

        // Dave should be able to submit app
        let mut req = PermissionRequest::new(realm.id.as_str(), dave.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Qari should be able to view app
        let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::VIEW, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Qari should not be able to create app
        let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert!(security_mgr.check(&req).is_err());

        // Ali should be able to create app
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Ali should be able to submit app
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert_eq!(PermissionResponse::Allow, security_mgr.check(&req).unwrap());

        // Ali should not be able to submit app with large app
        let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
        req.context.add("appSize", ValueWrapper::Int(5000));
        assert!(security_mgr.check(&req).is_err());
    }

    #[test]
    fn test_project() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "JobGrid").unwrap();

        // Creating organization
        let abc_corp = mgr.new_org_with(&ctx, "ABC").unwrap();
        let xyz_corp = mgr.new_org_with(&ctx, "XYZ").unwrap();

        // Create license policies
        let abc_policy = mgr.new_license_policy(&ctx, &abc_corp).unwrap();
        let xyz_policy = mgr.new_license_policy(&ctx, &xyz_corp).unwrap();

        // Creating Users
        let abc_dave = mgr.new_principal_with(&ctx, &abc_corp, "dave").unwrap();
        let abc_ali = mgr.new_principal_with(&ctx, &abc_corp, "ali").unwrap();

        let xyz_dan = mgr.new_principal_with(&ctx, &xyz_corp, "dan").unwrap();
        let xyz_ann = mgr.new_principal_with(&ctx, &xyz_corp, "ann").unwrap();

        // Creating Roles
        let abc_developer = mgr.new_role_with(&ctx, &realm, &abc_corp, "Developer").unwrap();
        let abc_admin = mgr.new_role_with_parent(&ctx, &realm, &abc_corp, &abc_developer, "Admin").unwrap();

        let xyz_developer = mgr.new_role_with(&ctx, &realm, &xyz_corp, "Developer").unwrap();
        let xyz_admin = mgr.new_role_with_parent(&ctx, &realm, &xyz_corp, &xyz_developer, "Admin").unwrap();

        // Creating Resources
        let project = mgr.new_resource_with(&ctx, &realm, "Project").unwrap();
        let job = mgr.new_resource_with(&ctx, &realm, "Job").unwrap();

        // Creating claims for resources
        let project_create_delete = mgr.new_claim_with(&ctx, &realm, &project, "(CREATE|DELETE)").unwrap();
        let project_view = mgr.new_claim_with(&ctx, &realm, &project, "VIEW").unwrap();
        let job_view_submit = mgr.new_claim_with(&ctx, &realm, &job, "(VIEW|SUBMIT)").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_principal_to_role(&ctx, &abc_dave, &abc_developer);
        mgr.map_principal_to_role(&ctx, &abc_ali, &abc_admin);

        mgr.map_principal_to_role(&ctx, &xyz_dan, &xyz_developer);
        mgr.map_principal_to_role(&ctx, &xyz_ann, &xyz_admin);

        // Map claims to policies as follows:
        mgr.map_license_policy_to_claim(&ctx, &abc_policy, &project_create_delete, "com.abc.app", "");
        mgr.map_license_policy_to_claim(&ctx, &abc_policy, &project_view, "com.abc.app", "");
        mgr.map_license_policy_to_claim(&ctx, &abc_policy, &job_view_submit, "com.abc.app", "appSize < 1000");

        mgr.map_license_policy_to_claim(&ctx, &xyz_policy, &project_create_delete, "com.xyz.app", "");
        mgr.map_license_policy_to_claim(&ctx, &xyz_policy, &project_view, "com.xyz.app", "");
        mgr.map_license_policy_to_claim(&ctx, &xyz_policy, &job_view_submit, "com.xyz.app", "appSize < 1000");

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &abc_admin, &project_create_delete, "com.abc.app", "");
        mgr.map_role_to_claim(&ctx, &abc_developer, &project_view, "com.abc.app", "");
        mgr.map_role_to_claim(&ctx, &abc_developer, &job_view_submit, "com.abc.app", "appSize < 1000");

        mgr.map_role_to_claim(&ctx, &xyz_admin, &project_create_delete, "com.xyz.app", "");
        mgr.map_role_to_claim(&ctx, &xyz_developer, &project_view, "com.xyz.app", "");
        mgr.map_role_to_claim(&ctx, &xyz_developer, &job_view_submit, "com.xyz.app", "appSize < 1000");

        let security_mgr = SecurityManager::new(mgr);

        // Ali for ABC should create project
        let mut req = PermissionRequest::new(realm.id.as_str(), abc_ali.id.as_str(), ActionType::CREATE, "Project", "com.abc.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert!(security_mgr.check(&req).is_ok());

        // Dave for ABC should be able to submit job
        let mut req = PermissionRequest::new(realm.id.as_str(), abc_dave.id.as_str(), ActionType::SUBMIT, "Job", "com.abc.app");
        req.context.add("appSize", ValueWrapper::Int(500));
        assert!(security_mgr.check(&req).is_ok());

    }

    #[test]
    fn test_quota_limits() {
        init();
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "JobGrid").unwrap();

        // Creating organization
        let abc_corp = mgr.new_org_with(&ctx, "ABC").unwrap();
        let xyz_corp = mgr.new_org_with(&ctx, "XYZ").unwrap();

        // Create license policies
        let abc_policy = mgr.new_license_policy(&ctx, &abc_corp).unwrap();
        let xyz_policy = mgr.new_license_policy(&ctx, &xyz_corp).unwrap();

        // Creating Resources
        let project = mgr.new_resource_with(&ctx, &realm, "Project").unwrap();
        let job = mgr.new_resource_with(&ctx, &realm, "Job").unwrap();

        // Set Resource Quota
        assert!(mgr.new_resource_quota_with(&ctx, &project, &abc_policy, "ABC Project", 1).is_ok());
        assert!(mgr.new_resource_quota_with(&ctx, &job, &abc_policy, "ABC Jobs", 2).is_ok());

        assert!(mgr.new_resource_quota_with(&ctx, &project, &xyz_policy, "XYZ Project", 2).is_ok());
        assert!(mgr.new_resource_quota_with(&ctx, &job, &xyz_policy, "XYZ Jobs", 3).is_ok());

        // abc can have at most 1 project
        assert!(mgr.new_resource_instance_with(&ctx, &project, &abc_policy, "ABC Project", "1", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &project, &abc_policy, "ABC Project", "2", Status::COMPLETED).is_err());

        // abc can have at most 3 jobs
        assert!(mgr.new_resource_instance_with(&ctx, &job, &abc_policy, "ABC Jobs", "1", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &job, &abc_policy, "ABC Jobs", "2", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &job, &abc_policy, "ABC Jobs", "3", Status::COMPLETED).is_err());

        // xyz can have at most 2 project
        assert!(mgr.new_resource_instance_with(&ctx, &project, &xyz_policy, "XYZ Project", "1", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &project, &xyz_policy, "XYZ Project", "2", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &project, &xyz_policy, "XYZ Project", "3", Status::COMPLETED).is_err());

        // xyz can have at most 4 jobs
        assert!(mgr.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "1", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "2", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "3", Status::COMPLETED).is_ok());
        assert!(mgr.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "4", Status::COMPLETED).is_err());
    }
}
