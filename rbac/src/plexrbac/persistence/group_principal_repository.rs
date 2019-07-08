//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_group_principals;
use super::models::{PGroupPrincipal};
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{NaiveDate, NaiveDateTime, Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// GroupPrincipalRepository is used to store Many-to-Many association between organization 
/// group and principal
///
pub struct GroupPrincipalRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> GroupPrincipalRepository<'a> {
    /// Adds principal to group
    pub fn add_principal_to_group(&self, ctx: &SecurityContext, group_id: &str, principal_id: &str) -> Result<usize, RbacError> {
        let gp = PGroupPrincipal::new(group_id, principal_id);
        match self.create(&gp) {
            Ok(n) => {
                self.audit(ctx, format!("Adding principal to group {:?}", gp), "CREATE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Removes principal from group
    pub fn delete_principal_from_group(&self, ctx: &SecurityContext, group_id: &str, principal_id: &str) -> Result<usize, RbacError> {
        let gp = PGroupPrincipal::new(group_id, principal_id);
        match self.delete(&gp) {
            Ok(n) => {
                self.audit(ctx, format!("Removing principal from group {:?}", gp), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }


    /// Checks existence of relationship between group and principal
    pub fn exists(&self, association: &PGroupPrincipal) -> bool {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_group_principals::table
                .filter(rbac_group_principals::group_id.eq(association.group_id.clone()))
                .filter(rbac_group_principals::principal_id.eq(association.principal_id.clone()))
                .load::<PGroupPrincipal>(&*connection) {
                Ok(v) => if v.first() != None {true} else {false},
                _ => false,
            }
        } else {
            false
        }
    }

    /// Finds all Many-to-Many association between organization group and principal for given group
    pub fn get_by_group(&self, group_id: &str) -> Vec<PGroupPrincipal> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_group_principals::table
                .filter(rbac_group_principals::group_id.eq(group_id.to_string()))
                .load::<PGroupPrincipal>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Finds all Many-to-Many association between organization group and principal for given
    /// principal
    pub fn get_by_principal(&self, principal_id: &str) -> Vec<PGroupPrincipal> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_group_principals::table
                .filter(rbac_group_principals::principal_id.eq(principal_id.to_string()))
                .load::<PGroupPrincipal>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    pub fn create(&self, association: &PGroupPrincipal) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_group_principals::table).values(association).execute(&*connection)
    }

    pub fn delete(&self, association: &PGroupPrincipal) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_group_principals::table)
            .filter(rbac_group_principals::group_id.eq(association.group_id.clone()))
            .filter(rbac_group_principals::principal_id.eq(association.principal_id.clone()))
            .execute(&*connection)
    }

    ///////////////////////////////////// PRIVATE METHODS ////////////////////////////////
    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }

    // removing all associations - For testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        let _ = diesel::delete(rbac_group_principals::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::persistence::models::{PGroupPrincipal};

    #[test]
    fn test_create() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_principal_repository();
        repo.clear();
        //
        let association = PGroupPrincipal::new("1", "2");
        repo.create(&association).unwrap();
        assert_eq!(true, repo.exists(&association));
    }

    #[test]
    fn test_delete() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_principal_repository();
        repo.clear();

        let association = PGroupPrincipal::new("1", "2");
        repo.create(&association).unwrap();
        repo.delete(&association).unwrap();
        assert_eq!(false, repo.exists(&association));
    }

    #[test]
    fn test_get_all() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_principal_repository();
        repo.clear();

        repo.create(&PGroupPrincipal::new("1", "12")).unwrap();
        repo.create(&PGroupPrincipal::new("1", "13")).unwrap();
        repo.create(&PGroupPrincipal::new("2", "14")).unwrap();
        repo.create(&PGroupPrincipal::new("2", "15")).unwrap();

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
