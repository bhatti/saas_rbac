//#![crate_name = "doc"]
//#[macro_use]
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;

//////////////////////////////////////////////////////////////////////////////////////////////
/// RepositoryFactory is used to create instance of repositories, which allow access to 
/// RBAC related database objects.
///
pub struct RepositoryFactory {
    pub pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,
}

impl RepositoryFactory {
    /// Creates instance of RepositoryFactory 
    pub fn new() -> RepositoryFactory {
        dotenv().ok(); // Grabbing ENV vars

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        // r2d2::Config Docs: https://docs.rs/r2d2/0.7.4/r2d2/struct.Config.html

        let manager = ConnectionManager::<SqliteConnection>::new(database_url);

        let pool = Pool::new(manager).expect("Failed to create pool.");
        RepositoryFactory {pool: Arc::new(pool)}
    }

    /// Creates instance of realm repository for persisting security realms
    pub fn new_realm_repository(&self) -> super::realm_repository::SecurityRealmRepository {
        super::realm_repository::SecurityRealmRepository {factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of organization repository for persisting org data
    pub fn new_org_repository(&self) -> super::org_repository::OrganizationRepository {
        super::org_repository::OrganizationRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of audit-record repository for persisting logs
    pub fn new_audit_record_repository(&self) -> super::audit_record_repository::AuditRecordRepository {
        super::audit_record_repository::AuditRecordRepository{factory: self}
    }

    /// Creates instance of group repository for persisting organization groups data
    pub fn new_group_repository(&self) -> super::group_repository::GroupRepository {
        super::group_repository::GroupRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of principal repository for persisting organization users
    pub fn new_principal_repository(&self) -> super::principal_repository::PrincipalRepository {
        super::principal_repository::PrincipalRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of group-principal repository for defining association between groups and
    /// principals
    pub fn new_group_principal_repository(&self) -> super::group_principal_repository::GroupPrincipalRepository {
        super::group_principal_repository::GroupPrincipalRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of resource repository for persisting target objects that need protection
    pub fn new_resource_repository(&self) -> super::resource_repository::ResourceRepository {
        super::resource_repository::ResourceRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates resource-insstance repository for persisting instances of resources
    pub fn new_resource_instance_repository(&self) -> super::resource_instance_repository::ResourceInstanceRepository{
        super::resource_instance_repository::ResourceInstanceRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates resource-limit repository for persisting resource-limits
    pub fn new_resource_limit_repository(&self) -> super::resource_limit_repository::ResourceLimitRepository {
        super::resource_limit_repository::ResourceLimitRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of role repository for persisting roles that are available for
    /// the organizations
    pub fn new_role_repository(&self) -> super::role_repository::RoleRepository {
        super::role_repository::RoleRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of role-roleable repository for defining association objects that can be tied with the role object.
    pub fn new_role_roleable_repository(&self) -> super::role_roleable_repository::RoleRoleableRepository {
        super::role_roleable_repository::RoleRoleableRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of claim repository for persisting mapping of resource and actions
    pub fn new_claim_repository(&self) -> super::claim_repository::ClaimRepository {
        super::claim_repository::ClaimRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of claim-claimable repository for persisting mapping of claim and
    /// claimable
    pub fn new_claim_claimable_repository(&self) -> super::claim_claimable_repository::ClaimClaimableRepository {
        super::claim_claimable_repository::ClaimClaimableRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of license-policy repository for persisting organization overall access
    pub fn new_license_policy_repository(&self) -> super::license_policy_repository::LicensePolicyRepository {
        super::license_policy_repository::LicensePolicyRepository{factory: self, audit_record_repository: self.new_audit_record_repository()}
    }

    /// Creates instance of rbac repository
    pub fn new_persistence_manager(&self) -> super::manager::PersistenceManager {
        super::manager::PersistenceManager{
            realm_repository: self.new_realm_repository(),
            org_repository: self.new_org_repository(),
            group_repository: self.new_group_repository(),
            principal_repository: self.new_principal_repository(),
            group_principal_repository: self.new_group_principal_repository(),
            resource_repository: self.new_resource_repository(),
            resource_instance_repository: self.new_resource_instance_repository(),
            resource_limit_repository: self.new_resource_limit_repository(),
            role_repository: self.new_role_repository(),
            role_roleable_repository: self.new_role_roleable_repository(),
            claim_repository: self.new_claim_repository(),
            claim_claimable_repository: self.new_claim_claimable_repository(),
            license_policy_repository: self.new_license_policy_repository(),
            audit_record_repository: self.new_audit_record_repository(),
        }
    }


    /// Creates a new database connection using database pool
    pub fn new_connection(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool.deref().get().unwrap()
    }
}

fn _establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

