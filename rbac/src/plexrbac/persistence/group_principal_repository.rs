//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_group_principals;
use super::models::{PGroupPrincipal};

//////////////////////////////////////////////////////////////////////////////////////////////
/// GroupPrincipalRepository is used to store Many-to-Many association between organization 
/// group and principal
///
pub struct GroupPrincipalRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> GroupPrincipalRepository<'a> {
    /// Checks existence of relationship between group and principal
    pub fn exists(&self, association: &PGroupPrincipal) -> bool {
        let connection = self.factory.new_connection();
        match rbac_group_principals::table
            .filter(rbac_group_principals::group_id.eq(association.group_id.clone()))
            .filter(rbac_group_principals::principal_id.eq(association.principal_id.clone()))
            .load::<PGroupPrincipal>(&*connection) {
            Ok(v) => if v.first() != None {true} else {false},
            _ => false,
        }
    }

    /// Finds all Many-to-Many association between organization group and principal for given group
    pub fn get_by_group(&self, group_id: &str) -> Vec<PGroupPrincipal> {
        let connection = self.factory.new_connection();
        match rbac_group_principals::table
            .filter(rbac_group_principals::group_id.eq(group_id.to_string()))
            .load::<PGroupPrincipal>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Finds all Many-to-Many association between organization group and principal for given
    /// principal
    pub fn get_by_principal(&self, principal_id: &str) -> Vec<PGroupPrincipal> {
        let connection = self.factory.new_connection();
        match rbac_group_principals::table
            .filter(rbac_group_principals::principal_id.eq(principal_id.to_string()))
            .load::<PGroupPrincipal>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    pub fn create(&self, association: &PGroupPrincipal) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_group_principals::table).values(association).execute(&*connection) {
            Ok(_) => {
                None
            },
            Err(err) => Some(err),
        }
    }

    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_group_principals::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    pub fn delete(&self, association: &PGroupPrincipal) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_group_principals::table)
            .filter(rbac_group_principals::group_id.eq(association.group_id.clone()))
            .filter(rbac_group_principals::principal_id.eq(association.principal_id.clone()))
            .execute(&*connection) {
            Ok(n) => {
                n > 0
            },
            _ => false,
        }
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::persistence::models::{PGroupPrincipal};

    #[test]
    fn test_create() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_principal_repository();
        repo.clear();
        //
        let association = PGroupPrincipal::new("1", "2");
        repo.create(&association);
        assert_eq!(true, repo.exists(&association));
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_principal_repository();
        repo.clear();

        let association = PGroupPrincipal::new("1", "2");
        repo.create(&association);
        repo.delete(&association);
        assert_eq!(false, repo.exists(&association));
    }

    #[test]
    fn test_get_all() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_principal_repository();
        repo.clear();

        repo.create(&PGroupPrincipal::new("1", "12"));
        repo.create(&PGroupPrincipal::new("1", "13"));
        repo.create(&PGroupPrincipal::new("2", "14"));
        repo.create(&PGroupPrincipal::new("2", "15"));

        let results = repo.get_by_group("1");
        assert_eq!(2, results.len());
        let results = repo.get_by_group("2");
        assert_eq!(2, results.len());
        let results = repo.get_by_principal("12");
        assert_eq!(1, results.len());
        let results = repo.get_by_principal("13");
        assert_eq!(1, results.len());
    }
}
