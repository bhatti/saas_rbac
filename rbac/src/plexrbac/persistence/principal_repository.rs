//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::rbac_principals;
use super::models::PPrincipal;
use plexrbac::domain::models::Principal;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;

//////////////////////////////////////////////////////////////////////////////////////////////
/// PrincipalRepository defines methods for accessing and persisting principals/users
///
pub struct PrincipalRepository<'a> {
    pub data_source: &'a dyn super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> PrincipalRepository<'a> {
    /// Retrieves principal by user-id from the database
    pub fn get(&self, _ctx: &SecurityContext, principal_id: &str) -> Option<Principal> {
        if let Ok(pprincipal) = self._get(principal_id) {
            Some(Principal::from(&pprincipal))
        } else {
            None
        }
    }

    /// Creates principal
    pub fn create(&self, ctx: &SecurityContext, principal: &Principal) -> Result<Principal, RbacError> {
        let mut db_obj = principal.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Created new principal {:?}", db_obj), "CREATE");
        Ok(Principal::from(&db_obj))
    }

    /// Updates principal
    pub fn update(&self, ctx: &SecurityContext, principal: &Principal) -> Result<Principal, RbacError> {
        match self._get(principal.id.as_str()) {
            Ok(mut db_obj) => {
                db_obj.description = principal.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated principal {:?}", db_obj), "UPDATE");
                Ok(Principal::from(&db_obj))
            }
            Err(err) => Err(RbacError::Persistence(err.to_string()))
        }
    }

    /// Returns all principals/users for given organization
    pub fn get_by_org(&self, _ctx: &SecurityContext, organization_id: &str) -> Vec<Principal> {
        self._get_by_org(organization_id).iter().map(|g| Principal::from(&g)).collect::<Vec<Principal>>()
    }

    /// Deletes principal by id from the database
    pub fn delete(&self, ctx: &SecurityContext, id: &str) -> Result<usize, RbacError> {
        match self._delete(id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted principal {:?}", id), "DELETE");
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

    /// Returns all principals/users for given organization
    pub fn _get_by_org(&self, organization_id: &str) -> Vec<PPrincipal> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_principals::table
                .filter(rbac_principals::organization_id.eq(organization_id.to_string()))
                .load::<PPrincipal>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Creates instance of principal/user
    fn _create(&self, principal: &PPrincipal) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_principals::table).values(principal).execute(&*connection)
    }

    /// Updates previous instance of the principal/user
    fn _update(&self, principal: &PPrincipal) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_principals::table.find(principal.id.clone())).set(principal).
            execute(&*connection)
    }

    /// Deletes instance of the principal/user by id from the database
    fn _delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_principals::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves instance of the principal/user by id from the database
    fn _get(&self, id: &str) -> Result<PPrincipal, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_principals::table.find(id.to_string()).get_result::<PPrincipal>(&*connection)
    }

    /// Removes all instances of the principal/user from the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_principals::table).execute(&*connection).unwrap();
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::Principal;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_principal_repository();
        repo.clear();

        let principal = repo.create(&ctx, &Principal::new("", "2", "myusername-principal", Some("desc".to_string()))).unwrap();
        let principal_str = format!("{:?}", principal);

        let loaded = repo.get(&ctx, principal.id.as_str()).unwrap();
        assert_eq!(principal_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_principal_repository();
        repo.clear();

        let principal = repo.create(&ctx, &Principal::new("", "2", "myusername-principal", Some("desc".to_string()))).unwrap();

        let mut loaded = repo.get(&ctx, principal.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert!(repo.update(&ctx, &loaded).is_ok());
        let loaded = repo.get(&ctx, loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_principal_repository();
        repo.clear();

        let principal = repo.create(&ctx, &Principal::new("", "2", "myusername-principal", Some("desc".to_string()))).unwrap();
        repo.delete(&ctx, principal.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, principal.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_principal_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Principal::new("", "2", "myusername-principal1", Some("desc".to_string()))).unwrap();
        let _ = repo.create(&ctx, &Principal::new("", "2", "myusername-principal2", Some("desc".to_string()))).unwrap();
        let results = repo.get_by_org(&ctx, "2");
        assert_eq!(2, results.len());
    }
}
