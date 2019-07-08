//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::{rbac_roles};
use super::models::{PRole};
use plexrbac::domain::models::Role;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////////////////////
/// RoleRepository defines methods for accessing and persisting roles
///
pub struct RoleRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> RoleRepository<'a> {
    /// Creates role
    pub fn create(&self, ctx: &SecurityContext, role: &Role) -> Result<Role, RbacError> {
        let mut db_obj = role.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Created new role {:?}", db_obj), "CREATE");
        Ok(Role::from(&db_obj))
    }

    /// Updates the role
    pub fn update (&self, ctx: &SecurityContext, role: &Role) -> Result<Role, RbacError> {
        match self._get(role.organization_id.as_str(), role.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.parent_id = role.parent_id.clone();
                db_obj.description = role.description.clone();
                //db_obj.role_constraints = role.role_constraints.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated role {:?}", db_obj), "UPDATE");
                Ok(Role::from(&db_obj))
            }
            None => Err(RbacError::NotFound(format!("Role not found {:?}", role)))
        }
    }

    /// Retrieves role by id from the database
    pub fn get(&self, _ctx: &SecurityContext, org_id: &str, role_id: &str) -> Option<Role> {
        match self._get(org_id, role_id) {
            Some(role) => Some(Role::from(&role)),
            _ => None,
        }
    }

    /// Returns all roles for given organization
    pub fn get_by_org(&self, _ctx: &SecurityContext, organization_id: &str) -> HashMap<String, Role> {
        let mut roles = HashMap::new();
        for prole in &self._get_by_org(organization_id) {
            let role = Role::from(prole); 
            roles.insert(role.id.clone(), role);
        }
        roles
    }

    /// Deletes role by id from the database
    pub fn delete(&self, ctx: &SecurityContext, org_id: &str, id: &str) -> Result<usize, RbacError> {
        match self._delete(org_id, id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted role {:?}", id), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    ///////////////////////////////////// PRIVATE METHODS ////////////////////////////////
    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }

    /// Returns all roles for given organization
    pub fn _get_by_org(&self, organization_id: &str) -> Vec<PRole> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_roles::table
                .filter(rbac_roles::organization_id.eq(organization_id.to_string()))
                .load::<PRole>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Returns all roles for role-ids -- used internally
    fn _get_roles_by_role_ids(&self, role_ids: Vec<String>) -> Vec<PRole> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_roles::table
                .filter(rbac_roles::id.eq_any(role_ids))
                .load::<PRole>(&*connection) {
                Ok(v) => return v,
                _ => return vec![],
            }
        } else {
            vec![]
        }
    }
    /// Creates an instance of role
    fn _create(&self, role: &PRole) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_roles::table).values(role).execute(&*connection)
    }

    /// Updates previous instance of the role
    fn _update(&self, role: &PRole) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_roles::table.find(role.id.clone())).set(role).
            execute(&*connection)
    }

    /// Removes resource in the database
    fn _delete(&self, org_id: &str, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_roles::table
                .filter(rbac_roles::organization_id.eq(org_id.to_string()))
                .filter(rbac_roles::id.eq(id.to_string())))
                .execute(&*connection)
    }

    /// Deletes instance of role by id from the database
    fn __delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_roles::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves resource in the database
    fn _get(&self, org_id: &str, id: &str) -> Option<PRole> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_roles::table
                .filter(rbac_roles::organization_id.eq(org_id.to_string()))
                .filter(rbac_roles::id.eq(id.to_string()))
                .load::<PRole>(&*connection) {
                    Ok(v) => {
                        if let Some(c) = v.first() {
                            Some(c.clone())
                        } else {
                            None
                        }
                    }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Retrieves instance of role by id from the database
    fn __get(&self, id: &str) -> Result<PRole, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_roles::table.find(id.to_string()).get_result::<PRole>(&*connection)
        //let children = Role::belonging_to(&role).load::<Role>(&*connection)?;
    }

    /// Removes all instances of roles from the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_roles::table).execute(&*connection).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::Role;
    use plexrbac::common::SecurityContext;


    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_repository();
        repo.clear();

        let role = repo.create(&ctx, &Role::new("", "default", "2", "myrole", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let role_str = format!("{:?}", role);

        let loaded = repo.get(&ctx, role.organization_id.as_str(), role.id.as_str()).unwrap();
        assert_eq!(role_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_repository();
        repo.clear();

        let role = repo.create(&ctx, &Role::new("", "default", "2", "myrole", Some("desc".to_string()), Some("parent".to_string()))).unwrap();

        let mut loaded = repo.get(&ctx, role.organization_id.as_str(), role.id.as_str()).unwrap();
        loaded.description = Some("newdesc".to_string());

        assert!(repo.update(&ctx, &loaded).is_ok());
        let loaded = repo.get(&ctx, role.organization_id.as_str(), loaded.id.as_str()).unwrap();
        assert_eq!(Some("newdesc".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_repository();
        repo.clear();

        let role = repo.create(&ctx, &Role::new("", "default", "2", "myrole", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        repo.delete(&ctx, "2", role.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, role.organization_id.as_str(), role.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_role_ids() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_repository();
        repo.clear();

        let role1 = repo.create(&ctx, &Role::new("", "default", "2", "myrole1", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let role2 = repo.create(&ctx, &Role::new("", "default", "2", "myrole2", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        assert_eq!(2, repo._get_roles_by_role_ids(vec![role1.id.clone(), role2.id.clone()]).len());
    }

    #[test]
    fn test_get_by_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Role::new("", "default", "2", "myrole1", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let _ = repo.create(&ctx, &Role::new("", "default", "2", "myrole2", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let results = repo.get_by_org(&ctx, "2");
        assert_eq!(2, results.len());
    }
}
