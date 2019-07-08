//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_role_roleables;
use super::models::PRoleRoleable;
use plexrbac::common::Constants;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{NaiveDate, NaiveDateTime, Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// RoleRoleableRepository defines association between role-roleable where roleable can be
/// principal or group that can be associated with roles for defining access control.
///
pub struct RoleRoleableRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> RoleRoleableRepository<'a> {
    /// Adds principal to role
    pub fn add_principal_to_role(&self, ctx: &SecurityContext, role_id: &str, principal_id: &str, constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Result<usize, RbacError> {
        let rr = PRoleRoleable::new(role_id, principal_id, Constants::Principal.to_string().as_str(), constraints, effective_at, expired_at);
        match self.create(&rr) {
            Ok(n) => {
                self.audit(ctx, format!("Adding principal to role {:?}", rr), "CREATE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Removes principal from role
    pub fn delete_principal_from_role(&self, ctx: &SecurityContext, role_id: &str, principal_id: &str) -> Result<usize, RbacError> {
        let rr = PRoleRoleable::new(role_id, principal_id, Constants::Principal.to_string().as_str(), "", Utc::now().naive_utc(), Utc::now().naive_utc());
        match self.delete(&rr) {
            Ok(n) => {
                self.audit(ctx, format!("Removing principal from role {:?}", rr), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Adds group to role
    pub fn add_group_to_role(&self, ctx: &SecurityContext, role_id: &str, group_id: &str, constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Result<usize, RbacError> {
        let rr = PRoleRoleable::new(role_id, group_id, Constants::Group.to_string().as_str(), constraints, effective_at, expired_at);
        match self.create(&rr) {
            Ok(n) => {
                self.audit(ctx, format!("Adding group to role {:?}", rr), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Removes group from role
    pub fn delete_group_from_role(&self, ctx: &SecurityContext, role_id: &str, group_id: &str) -> Result<usize, RbacError> {
        let rr = PRoleRoleable::new(role_id, group_id, Constants::Group.to_string().as_str(), "", Utc::now().naive_utc(), Utc::now().naive_utc());
        match self.delete(&rr) {
            Ok(n) => {
                self.audit(ctx, format!("Removing group from role {:?}", rr), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }


    /// Returns role-ids for given principal
    pub fn get_role_ids_by_principal(&self, principal_id: &str) -> Vec<String> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
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
        } else {
            vec![]
        }
    }

    /// Returns role-ids for given group
    pub fn get_role_ids_by_group(&self, group_id: String) -> Vec<String> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
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
        } else {
            vec![]
        }
    }

    /// Checks if association exists
    pub fn exists(&self, rr: &PRoleRoleable) -> bool {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
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
        } else {
            false
        }
    }

    /// Retrieves associations by role-id
    pub fn get_by_role(&self, role_id: &str) -> Vec<PRoleRoleable> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_role_roleables::table
                .filter(rbac_role_roleables::role_id.eq(role_id.to_string()))
                .filter(rbac_role_roleables::effective_at.le(now))
                .filter(rbac_role_roleables::expired_at.ge(now))
                .load::<PRoleRoleable>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieves associations by roleable-id and type
    pub fn get_by_roleable(&self, roleable_id: &str, roleable_type: &str) -> Vec<PRoleRoleable> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_role_roleables::table
                .filter(rbac_role_roleables::roleable_id.eq(roleable_id.to_string()))
                .filter(rbac_role_roleables::roleable_type.eq(roleable_type.to_string()))
                .filter(rbac_role_roleables::effective_at.le(now))
                .filter(rbac_role_roleables::expired_at.ge(now))
                .load::<PRoleRoleable>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Creates new assocoation between role and role-able
    pub fn create(&self, rr: &PRoleRoleable) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_role_roleables::table).values(rr).execute(&*connection)
    }


    /// Deletes assocoation between role and role-able
    pub fn delete(&self, rr: &PRoleRoleable) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_role_roleables::table)
            .filter(rbac_role_roleables::role_id.eq(rr.role_id.clone()))
            .filter(rbac_role_roleables::roleable_id.eq(rr.roleable_id.clone()))
            .filter(rbac_role_roleables::roleable_type.eq(rr.roleable_type.clone()))
            .execute(&*connection)
    }

    ///////////////////////////////////// PRIVATE METHODS ////////////////////////////////
    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }

    /// Removes all assocoations between role and role-able
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_role_roleables::table).execute(&*connection).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::models::PRoleRoleable;
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_exists() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_roleable_repository();
        repo.clear();

        let rr = PRoleRoleable::new("1", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&rr).unwrap();
        assert_eq!(true, repo.exists(&rr));
    }

    #[test]
    fn test_delete() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_roleable_repository();
        repo.clear();
        let rr = PRoleRoleable::new("1", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&rr).unwrap();
        repo.delete(&rr).unwrap();
        assert_eq!(false, repo.exists(&rr));
    }

    #[test]
    fn test_get_all() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_role_roleable_repository();
        repo.clear();

        repo.create(&PRoleRoleable::new("1", "12", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.create(&PRoleRoleable::new("1", "13", "principal", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.create(&PRoleRoleable::new("2", "14", "group", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.create(&PRoleRoleable::new("2", "15", "group", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let results = repo.get_by_role("1");
        assert_eq!(2, results.len());
        let results = repo.get_by_roleable("12", "principal");
        assert_eq!(1, results.len());
    }
}
