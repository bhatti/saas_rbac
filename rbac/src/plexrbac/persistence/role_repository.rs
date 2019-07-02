//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::{rbac_roles};
use super::models::{PRole};

//////////////////////////////////////////////////////////////////////////////////////////////
/// RoleRepository defines methods for accessing and persisting roles
///
pub struct RoleRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> RoleRepository<'a> {
    /// Returns all roles for given organization
    pub fn get_by_org(&self, organization_id: &str) -> Vec<PRole> {
        let connection = self.factory.new_connection();
        match rbac_roles::table
            .filter(rbac_roles::organization_id.eq(organization_id.to_string()))
            .load::<PRole>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Returns all roles for role-ids
    pub fn get_roles_by_role_ids(&self, role_ids: Vec<String>) -> Vec<PRole> {
        let connection = self.factory.new_connection();
        match rbac_roles::table
            .filter(rbac_roles::id.eq_any(role_ids))
            .load::<PRole>(&*connection) {
            Ok(v) => return v,
            _ => return vec![],
        }
    }

    /// Creates an instance of role
    pub fn create(&self, role: &PRole) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_roles::table).values(role).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates previous instance of the role
    pub fn update(&self, role: &PRole) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_roles::table.find(role.id.clone())).set(role).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all instances of roles from the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_roles::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes instance of role by id from the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_roles::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Retrieves instance of role by id from the database
    pub fn get(&self, id: &str) -> Option<PRole> {
        let connection = self.factory.new_connection();
        match rbac_roles::table.find(id.to_string()).get_result::<PRole>(&*connection) {
            Ok(role) => {
                //let children = Role::belonging_to(&role).load::<Role>(&*connection)?;
                Some(role)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::PRole;
    use plexrbac::persistence::factory::RepositoryFactory;
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_repository();
        repo.clear();

        let role = PRole::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "2", "myrole", Some("desc".to_string()), Some("parent".to_string()));
        let role_str = format!("{:?}", role);
        repo.create(&role);

        let loaded = repo.get(role.id.as_str()).unwrap();
        assert_eq!(role_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_repository();
        repo.clear();

        let role = PRole::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "2", "myrole", Some("desc".to_string()), Some("parent".to_string()));
        repo.create(&role);

        let mut loaded = repo.get(role.id.as_str()).unwrap();
        loaded.description = Some("newdesc".to_string());
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("newdesc".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_repository();
        repo.clear();

        let role = PRole::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "2", "myrole", None, None);
        repo.create(&role);
        assert_eq!(1, repo.get_roles_by_role_ids(vec![role.id.clone()]).len());
        repo.delete(role.id.as_str());
        let loaded = repo.get(role.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_all() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_repository();
        repo.clear();

        repo.create(&PRole::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "2", "myrole1", None, None));
        repo.create(&PRole::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "2", "myrole2", None, None));

        let results = repo.get_by_org("2");
        assert_eq!(2, results.len());
    }
}
