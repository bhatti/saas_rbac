//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_realms;
use super::models::PSecurityRealm;
use plexrbac::domain::models::SecurityRealm;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// SecurityRealmRepository defines methods for accessing and persisting security realms that
/// encompasses security of RBAC system.
///
pub struct SecurityRealmRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> SecurityRealmRepository<'a> {
    ////////////////////////////////// SECURITY REALM CRUD OPERATIONS //////////////////////////////
    /// Updates security realm
    pub fn update(&self, ctx: &SecurityContext, realm: &SecurityRealm) -> Result<SecurityRealm, RbacError> {
        match self._get(realm.id.as_str()) {
            Ok(mut db_obj) => {
                db_obj.description = realm.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated realm {:?}", db_obj), "UPDATE");
                Ok(SecurityRealm::from(&db_obj))
            }
            Err(err) => Err(RbacError::Persistence(err.to_string()))
        }
    }

    /// Creates security realm
    pub fn create(&self, ctx: &SecurityContext, realm: &SecurityRealm) -> Result<SecurityRealm, RbacError> {
        let mut db_obj = realm.to();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Created new realm realm {:?}", db_obj), "CREATE");
        Ok(SecurityRealm::from(&db_obj))
    }

    /// Retrieves realm by realm-id from the database
    pub fn get(&self, _ctx: &SecurityContext, realm_id: &str) -> Option<SecurityRealm> {
        match self._get(realm_id) {
            Ok(realm) => Some(SecurityRealm::from(&realm)),
            _ => None,
        }
    }

    /// Retrieves all realm from the database
    pub fn all(&self, _ctx: &SecurityContext) -> Vec<SecurityRealm> {
        self._all().iter().map(|r| SecurityRealm::from(&r)).collect::<Vec<SecurityRealm>>()
    }

    /// Deletes realm by id from the database
    pub fn delete(&self, ctx: &SecurityContext, id: &str) -> Result<usize, RbacError> {
        match self._delete(id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted security realm {:?}", id), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    ///////////////////////////////////// PRIVATE METHODS ////////////////////////////////
    /// Returns all security realms available (up-to 1000)
    fn _all(&self) -> Vec<PSecurityRealm> {
        if let Ok(conn) = self.data_source.new_connection() {
            match rbac_realms::table
                .limit(1000)
                .load::<PSecurityRealm>(&*conn) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Creates an instance of security realm
    fn _create(&self, realm: &PSecurityRealm) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_realms::table).values(realm).execute(&*connection)
    }

    /// Updates instance of realm in the database
    fn _update(&self, realm: &PSecurityRealm) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_realms::table.find(realm.id.clone())).set(realm).execute(&*connection)
    }

    /// Retrieves realm by id from the database
    fn _get(&self, id: &str) -> Result<PSecurityRealm, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_realms::table.find(id.to_string()).get_result::<PSecurityRealm>(&*connection)
    }

    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }

    fn _delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_realms::table.find(id.to_string())).execute(&*connection)
    }

    /// Removes all security realms from the database for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        let _ = diesel::delete(rbac_realms::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::domain::models::SecurityRealm;
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_realm_repository();
        repo.clear();

        let realm = SecurityRealm::new("myrealm", Some("blah".to_string()));
        let realm_str = format!("{:?}", realm);
        repo.create(&ctx, &realm).unwrap();

        let loaded = repo.get(&ctx, realm.id.as_str()).unwrap();
        assert_eq!(realm_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_realm_repository();
        repo.clear();

        let realm = SecurityRealm::new("myrealm", Some("desc".to_string()));
        repo.create(&ctx, &realm).unwrap();

        let mut loaded = repo.get(&ctx, realm.id.as_str()).unwrap();
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
        let repo = locator.new_realm_repository();
        repo.clear();

        let mut realm = SecurityRealm::new("myrealm", None);
        repo.create(&ctx, &mut realm).unwrap();
        repo.delete(&ctx, realm.id.as_str()).unwrap();

        assert_eq!(None, repo.get(&ctx, realm.id.as_str()));
    }

    #[test]
    fn test_get_all() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_realm_repository();
        repo.clear();

        repo.create(&ctx, &mut SecurityRealm::new("myrealm1", None)).unwrap();
        repo.create(&ctx, &mut SecurityRealm::new("myrealm2", None)).unwrap();

        let results = repo.all(&ctx);
        assert_eq!(2, results.len());
    }
}
