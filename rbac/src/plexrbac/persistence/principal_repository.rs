//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_principals;
use super::models::PPrincipal;

//////////////////////////////////////////////////////////////////////////////////////////////
/// PrincipalRepository defines methods for accessing and persisting principals/users
///
pub struct PrincipalRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> PrincipalRepository<'a> {
    /// Returns all principals/users for given organization
    pub fn get_by_org(&self, organization_id: &str) -> Vec<PPrincipal> {
        let connection = self.factory.new_connection();
        match rbac_principals::table
            .filter(rbac_principals::organization_id.eq(organization_id.to_string()))
            .load::<PPrincipal>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Creates instance of principal/user
    pub fn create(&self, principal: &PPrincipal) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_principals::table).values(principal).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates previous instance of the principal/user
    pub fn update(&self, principal: &PPrincipal) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_principals::table.find(principal.id.clone())).set(principal).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all instances of the principal/user from the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_principals::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes instance of the principal/user by id from the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_principals::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }
    }

    /// Retrieves instance of the principal/user by id from the database
    pub fn get(&self, id: &str) -> Option<PPrincipal> {
        let connection = self.factory.new_connection();
        match rbac_principals::table.find(id.to_string()).get_result::<PPrincipal>(&*connection) {
            Ok(principal) => {
                Some(principal)
            },
            Err(_) => None,
        }
    }

}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::PPrincipal;
    use plexrbac::persistence::factory::RepositoryFactory;
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_principal_repository();
        repo.clear();

        let principal = PPrincipal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "myprincipal", None);
        let principal_str = format!("{:?}", principal);
        repo.create(&principal);

        let loaded = repo.get(principal.id.as_str()).unwrap();
        assert_eq!(principal_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_principal_repository();
        repo.clear();

        let principal = PPrincipal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "myprincipal", None);
        repo.create(&principal);

        let mut loaded = repo.get(principal.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_principal_repository();
        repo.clear();

        let principal = PPrincipal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "myprincipal", None);
        repo.create(&principal);
        repo.delete(principal.id.as_str());
        let loaded = repo.get(principal.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_org() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_principal_repository();
        repo.clear();

        repo.create(&PPrincipal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "myprincipal1", None));
        repo.create(&PPrincipal::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "myprincipal2", None));

        let results = repo.get_by_org("2");
        assert_eq!(2, results.len());
    }
}
