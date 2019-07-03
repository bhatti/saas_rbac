//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_role_roleables;
use super::models::PRoleRoleable;
use plexrbac::common::Constants;
use chrono::{Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// RoleRoleableRepository defines association between role-roleable where roleable can be
/// principal or group that can be associated with roles for defining access control.
///
pub struct RoleRoleableRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> RoleRoleableRepository<'a> {
    /// Returns role-ids for given principal
    pub fn get_role_ids_by_principal(&self, principal_id: &str) -> Vec<String> {
        let now = Utc::now().naive_utc();
        let connection = self.factory.new_connection();
        if let Ok(role_ids) = rbac_role_roleables::table.select(rbac_role_roleables::role_id)
            .filter(rbac_role_roleables::roleable_type.eq(Constants::Principal.to_string()))
            .filter(rbac_role_roleables::roleable_id.eq(principal_id.to_string()))
            .filter(rbac_role_roleables::effective_at.le(now))
            .filter(rbac_role_roleables::expired_at.ge(now))
            .group_by(rbac_role_roleables::role_id)
            .load::<String>(&*connection) {
                role_ids
        } else {
            vec![]
        }
    }

    /// Returns role-ids for given group
    pub fn get_role_ids_by_group(&self, group_id: String) -> Vec<String> {
        let now = Utc::now().naive_utc();
        let connection = self.factory.new_connection();
        if let Ok(role_ids) = rbac_role_roleables::table.select(rbac_role_roleables::role_id)
            .filter(rbac_role_roleables::roleable_type.eq(Constants::Group.to_string()))
            .filter(rbac_role_roleables::roleable_id.eq(group_id.clone()))
            .filter(rbac_role_roleables::effective_at.le(now))
            .filter(rbac_role_roleables::expired_at.ge(now))
            .group_by(rbac_role_roleables::role_id)
            .load::<String>(&*connection) {
            role_ids
        } else {
            vec![]
        }
    }

    /// Checks if association exists
    pub fn exists(&self, rr: &PRoleRoleable) -> bool {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_role_roleables::table
            .filter(rbac_role_roleables::role_id.eq(rr.role_id.clone()))
            .filter(rbac_role_roleables::roleable_id.eq(rr.roleable_id.clone()))
            .filter(rbac_role_roleables::roleable_type.eq(rr.roleable_type.clone()))
            .filter(rbac_role_roleables::effective_at.le(now))
            .filter(rbac_role_roleables::expired_at.ge(now))
            .load::<PRoleRoleable>(&*connection) {
            Ok(v) => if v.first() != None {true} else {false},
            _ => false,
        }
    }

    /// Retrieves associations by role-id
    pub fn get_by_role(&self, role_id: &str) -> Vec<PRoleRoleable> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_role_roleables::table
            .filter(rbac_role_roleables::role_id.eq(role_id.to_string()))
            .filter(rbac_role_roleables::effective_at.le(now))
            .filter(rbac_role_roleables::expired_at.ge(now))
            .load::<PRoleRoleable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Retrieves associations by roleable-id and type
    pub fn get_by_roleable(&self, roleable_id: &str, roleable_type: &str) -> Vec<PRoleRoleable> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_role_roleables::table
            .filter(rbac_role_roleables::roleable_id.eq(roleable_id.to_string()))
            .filter(rbac_role_roleables::roleable_type.eq(roleable_type.to_string()))
            .filter(rbac_role_roleables::effective_at.le(now))
            .filter(rbac_role_roleables::expired_at.ge(now))
            .load::<PRoleRoleable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Creates new assocoation between role and role-able
    pub fn create(&self, rr: &PRoleRoleable) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_role_roleables::table).values(rr).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all assocoations between role and role-able
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_role_roleables::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes assocoation between role and role-able
    pub fn delete(&self, rr: &PRoleRoleable) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_role_roleables::table)
            .filter(rbac_role_roleables::role_id.eq(rr.role_id.clone()))
            .filter(rbac_role_roleables::roleable_id.eq(rr.roleable_id.clone()))
            .filter(rbac_role_roleables::roleable_type.eq(rr.roleable_type.clone()))
            .execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::models::PRoleRoleable;
    use plexrbac::persistence::factory::RepositoryFactory;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_exists() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_roleable_repository();
        repo.clear();

        let rr = PRoleRoleable::new("1", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&rr);
        assert_eq!(true, repo.exists(&rr));
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_roleable_repository();
        repo.clear();
        let rr = PRoleRoleable::new("1", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&rr);
        repo.delete(&rr);
        assert_eq!(false, repo.exists(&rr));
    }

    #[test]
    fn test_get_all() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_role_roleable_repository();
        repo.clear();

        repo.create(&PRoleRoleable::new("1", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PRoleRoleable::new("1", "13", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PRoleRoleable::new("2", "14", "group", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PRoleRoleable::new("2", "15", "group", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));

        let results = repo.get_by_role("1");
        assert_eq!(2, results.len());
        let results = repo.get_by_roleable("12", "principal");
        assert_eq!(1, results.len());
    }
}
