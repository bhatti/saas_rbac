//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_realms;
use super::models::PSecurityRealm;

//////////////////////////////////////////////////////////////////////////////////////////////
/// SecurityRealmRepository defines methods for accessing and persisting security realms that
/// encompasses security of RBAC system.
///
pub struct SecurityRealmRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> SecurityRealmRepository<'a> {
    /// Returns all security realms available (up-to 1000)
    pub fn all(&self) -> Vec<PSecurityRealm> {
        let connection = self.factory.new_connection();
        match rbac_realms::table
            .limit(1000)
            .load::<PSecurityRealm>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Creates an instance of security realm
    pub fn create(&self, realm: &PSecurityRealm) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_realms::table).values(realm).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates instance of realm in the database
    pub fn update(&self, realm: &PSecurityRealm) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_realms::table.find(realm.id.clone())).set(realm).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all security realms from the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_realms::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes realm by id from the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_realms::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Retrieves realm by id from the database
    pub fn get(&self, id: &str) -> Result<PSecurityRealm, diesel::result::Error> {
        let connection = self.factory.new_connection();
        rbac_realms::table.find(id.to_string()).get_result::<PSecurityRealm>(&*connection)
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::models::PSecurityRealm;
    use plexrbac::persistence::factory::RepositoryFactory;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_realm_repository();
        repo.clear();

        let realm = PSecurityRealm::new("myrealm", "");
        let realm_str = format!("{:?}", realm);
        repo.create(&realm);

        let loaded = repo.get(realm.id.as_str()).unwrap();
        assert_eq!(realm_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_realm_repository();
        repo.clear();

        let realm = PSecurityRealm::new("myrealm", "");
        repo.create(&realm);

        let mut loaded = repo.get(realm.id.as_str()).unwrap();
        loaded.description = "blah".to_string();
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!("blah".to_string(), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_realm_repository();
        repo.clear();

        let mut realm = PSecurityRealm::new("myrealm", "");
        repo.create(&mut realm);
        realm.description = "blah".to_string();
        assert_eq!(None, repo.update(&realm));
        repo.delete(realm.id.as_str());

        assert_eq!(true, repo.get(realm.id.as_str()).is_err());
    }

    #[test]
    fn test_get_all() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_realm_repository();
        repo.clear();

        repo.create(&mut PSecurityRealm::new("myrealm1", ""));
        repo.create(&mut PSecurityRealm::new("myrealm2", ""));

        let results = repo.all();
        assert_eq!(2, results.len());
    }
}
