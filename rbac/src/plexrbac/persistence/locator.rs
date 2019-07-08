//#![crate_name = "doc"]
//#[macro_use]

use super::data_source::{DataSource};

//////////////////////////////////////////////////////////////////////////////////////////////
/// RepositoryLocator is used to create instance of repositories, which allow access to 
/// RBAC related database objects.
///
pub struct RepositoryLocator<'a> {
    pub data_source: &'a DataSource
}

impl<'a> RepositoryLocator<'a> {
    /// Creates instance of RepositoryLocator 
    pub fn new(data_source: &DataSource) -> RepositoryLocator {
        RepositoryLocator {data_source: data_source}
    }

    /// Creates instance of realm repository for persisting security realms
    pub fn new_realm_repository(&self) -> super::realm_repository::SecurityRealmRepository {
        RepositoryLocator::build_realm_repository(self.data_source)
    }

    /// Creates instance of organization repository for persisting org data
    pub fn new_org_repository(&self) -> super::org_repository::OrganizationRepository {
        RepositoryLocator::build_org_repository(self.data_source)
    }

    /// Creates instance of audit-record repository for persisting logs
    pub fn new_audit_record_repository(&self) -> super::audit_record_repository::AuditRecordRepository {
        RepositoryLocator::build_audit_record_repository(self.data_source)
    }

    /// Creates instance of group repository for persisting organization groups data
    pub fn new_group_repository(&self) -> super::group_repository::GroupRepository {
        RepositoryLocator::build_group_repository(self.data_source)
    }

    /// Creates instance of principal repository for persisting organization users
    pub fn new_principal_repository(&self) -> super::principal_repository::PrincipalRepository {
        RepositoryLocator::build_principal_repository(self.data_source)
    }

    /// Creates instance of group-principal repository for defining association between groups and
    /// principals
    pub fn new_group_principal_repository(&self) -> super::group_principal_repository::GroupPrincipalRepository {
        RepositoryLocator::build_group_principal_repository(self.data_source)
    }

    /// Creates instance of resource repository for persisting target objects that need protection
    pub fn new_resource_repository(&self) -> super::resource_repository::ResourceRepository {
        RepositoryLocator::build_resource_repository(self.data_source)
    }

    /// Creates resource-insstance repository for persisting instances of resources
    pub fn new_resource_instance_repository(&self) -> super::resource_instance_repository::ResourceInstanceRepository{
        RepositoryLocator::build_resource_instance_repository(self.data_source)
    }

    /// Creates resource-quota repository for persisting resource-quotas
    pub fn new_resource_quota_repository(&self) -> super::resource_quota_repository::ResourceQuotaRepository {
        RepositoryLocator::build_resource_quota_repository(self.data_source)
    }

    /// Creates instance of role repository for persisting roles that are available for
    /// the organizations
    pub fn new_role_repository(&self) -> super::role_repository::RoleRepository {
        RepositoryLocator::build_role_repository(self.data_source)
    }

    /// Creates instance of role-roleable repository for defining association objects that can be tied with the role object.
    pub fn new_role_roleable_repository(&self) -> super::role_roleable_repository::RoleRoleableRepository {
        RepositoryLocator::build_role_roleable_repository(self.data_source)
    }

    /// Creates instance of claim repository for persisting mapping of resource and actions
    pub fn new_claim_repository(&self) -> super::claim_repository::ClaimRepository {
        RepositoryLocator::build_claim_repository(self.data_source)
    }

    /// Creates instance of claim-claimable repository for persisting mapping of claim and
    /// claimable
    pub fn new_claim_claimable_repository(&self) -> super::claim_claimable_repository::ClaimClaimableRepository {
        RepositoryLocator::build_claim_claimable_repository(self.data_source)
    }

    /// Creates instance of license-policy repository for persisting organization overall access
    pub fn new_license_policy_repository(&self) -> super::license_policy_repository::LicensePolicyRepository {
        RepositoryLocator::build_license_policy_repository(self.data_source)
    }

    /// Creates instance of rbac repository
    pub fn new_persistence_manager(&self) -> super::manager::PersistenceManager {
        RepositoryLocator::build_persistence_manager(self.data_source)
    }


    /// Creates instance of realm repository for persisting security realms
    pub fn build_realm_repository(data_source: &DataSource) -> super::realm_repository::SecurityRealmRepository {
        super::realm_repository::SecurityRealmRepository {data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of organization repository for persisting org data
    pub fn build_org_repository(data_source: &DataSource) -> super::org_repository::OrganizationRepository {
        super::org_repository::OrganizationRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of audit-record repository for persisting logs
    pub fn build_audit_record_repository(data_source: &DataSource) -> super::audit_record_repository::AuditRecordRepository {
        super::audit_record_repository::AuditRecordRepository{data_source: data_source}
    }

    /// Creates instance of group repository for persisting organization groups data
    pub fn build_group_repository(data_source: &DataSource) -> super::group_repository::GroupRepository {
        super::group_repository::GroupRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of principal repository for persisting organization users
    pub fn build_principal_repository(data_source: &DataSource) -> super::principal_repository::PrincipalRepository {
        super::principal_repository::PrincipalRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of group-principal repository for defining association between groups and
    /// principals
    pub fn build_group_principal_repository(data_source: &DataSource) -> super::group_principal_repository::GroupPrincipalRepository {
        super::group_principal_repository::GroupPrincipalRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of resource repository for persisting target objects that need protection
    pub fn build_resource_repository(data_source: &DataSource) -> super::resource_repository::ResourceRepository {
        super::resource_repository::ResourceRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates resource-insstance repository for persisting instances of resources
    pub fn build_resource_instance_repository(data_source: &DataSource) -> super::resource_instance_repository::ResourceInstanceRepository{
        super::resource_instance_repository::ResourceInstanceRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates resource-quota repository for persisting resource-quotas
    pub fn build_resource_quota_repository(data_source: &DataSource) -> super::resource_quota_repository::ResourceQuotaRepository {
        super::resource_quota_repository::ResourceQuotaRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of role repository for persisting roles that are available for
    /// the organizations
    pub fn build_role_repository(data_source: &DataSource) -> super::role_repository::RoleRepository {
        super::role_repository::RoleRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of role-roleable repository for defining association objects that can be tied with the role object.
    pub fn build_role_roleable_repository(data_source: &DataSource) -> super::role_roleable_repository::RoleRoleableRepository {
        super::role_roleable_repository::RoleRoleableRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of claim repository for persisting mapping of resource and actions
    pub fn build_claim_repository(data_source: &DataSource) -> super::claim_repository::ClaimRepository {
        super::claim_repository::ClaimRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of claim-claimable repository for persisting mapping of claim and
    /// claimable
    pub fn build_claim_claimable_repository(data_source: &DataSource) -> super::claim_claimable_repository::ClaimClaimableRepository {
        super::claim_claimable_repository::ClaimClaimableRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of license-policy repository for persisting organization overall access
    pub fn build_license_policy_repository(data_source: &DataSource) -> super::license_policy_repository::LicensePolicyRepository {
        super::license_policy_repository::LicensePolicyRepository{data_source: data_source, audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source)}
    }

    /// Creates instance of rbac repository
    pub fn build_persistence_manager(data_source: &DataSource) -> super::manager::PersistenceManager {
        super::manager::PersistenceManager{
            realm_repository: RepositoryLocator::build_realm_repository(data_source),
            org_repository: RepositoryLocator::build_org_repository(data_source),
            group_repository: RepositoryLocator::build_group_repository(data_source),
            principal_repository: RepositoryLocator::build_principal_repository(data_source),
            group_principal_repository: RepositoryLocator::build_group_principal_repository(data_source),
            resource_repository: RepositoryLocator::build_resource_repository(data_source),
            resource_instance_repository: RepositoryLocator::build_resource_instance_repository(data_source),
            resource_quota_repository: RepositoryLocator::build_resource_quota_repository(data_source),
            role_repository: RepositoryLocator::build_role_repository(data_source),
            role_roleable_repository: RepositoryLocator::build_role_roleable_repository(data_source),
            claim_repository: RepositoryLocator::build_claim_repository(data_source),
            claim_claimable_repository: RepositoryLocator::build_claim_claimable_repository(data_source),
            license_policy_repository: RepositoryLocator::build_license_policy_repository(data_source),
            audit_record_repository: RepositoryLocator::build_audit_record_repository(data_source),
        }
    }
}

