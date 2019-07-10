//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::rbac_license_policies;
use super::models::{PLicensePolicy};
use chrono::{Utc};
use plexrbac::domain::models::LicensePolicy;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use self::uuu::Uuid;

//////////////////////////////////////////////////////////////////////////////////////////////
/// LicensePolicyRepository defines methods for accessing and persisting license-policies that
/// defines overall access policies for customers
///
pub struct LicensePolicyRepository<'a> {
    pub data_source: &'a dyn super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> LicensePolicyRepository<'a> {
    /// Creates license-policy
    pub fn create(&self, ctx: &SecurityContext, policy: &LicensePolicy) -> Result<LicensePolicy, RbacError> {
        let mut db_obj = policy.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if self._get_by_org(policy.organization_id.as_str()).len() > 0 {
            self.audit(ctx, format!("License policy already exist for {:?}", policy), "CREATE");
            return Err(RbacError::Duplicate(format!("License policy already exist for {:?}", policy)));
        }
        //
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Adding new license-policy {:?}", policy), "CREATE");
        Ok(LicensePolicy::from(&db_obj))
    }

    /// Updates the license-policy
    pub fn update(&self, ctx: &SecurityContext, policy: &LicensePolicy) -> Result<LicensePolicy, RbacError> {
        match self._get(policy.organization_id.as_str(), policy.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.effective_at = policy.effective_at.clone();
                db_obj.expired_at= policy.expired_at.clone();
                db_obj.description = policy.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated license-policy {:?}", policy), "UPDATE");
                if self._get_by_org(policy.organization_id.as_str()).len() > 1 {
                    self.audit(ctx, format!("Multiple license policies exist for {:?}", policy), "CREATE");
                    return Err(RbacError::Duplicate(format!("Multiple license policies exist for {:?}", policy)));
                }
                Ok(LicensePolicy::from(&db_obj))
            }
            None => Err(RbacError::NotFound(format!("License policy not found {:?}", policy)))
        }
    }


    /// Retrieves license-policy by id from the database
    pub fn get(&self, _ctx: &SecurityContext, org_id: &str, policy_id: &str) -> Option<LicensePolicy> {
        match self._get(org_id, policy_id) {
            Some(policy) => Some(LicensePolicy::from(&policy)),
            _ => None,
        }
    }

    /// Returns groups by group-ids
    pub fn get_by_org(&self, _ctx: &SecurityContext, org_id: &str) -> Vec<LicensePolicy> {
        self._get_by_org(org_id).iter().map(|l| LicensePolicy::from(&l)).collect::<Vec<LicensePolicy>>()
    }

    /// Deletes license policy by id from the database
    pub fn delete(&self, ctx: &SecurityContext, org_id: &str, id: &str) -> Result<usize, RbacError> {
        match self._delete(org_id, id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted license policy {:?}", id), "DELETE");
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

    /// Creates instance of license-policy
    fn _create(&self, license_policy: &PLicensePolicy) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_license_policies::table).values(license_policy).execute(&*connection)
    }

    /// Updates previous instance of the license-policy
    fn _update(&self, license_policy: &PLicensePolicy) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_license_policies::table.find(license_policy.id.clone())).set(license_policy).
            execute(&*connection)
    }

    /// Deletes instance of the license-policy by id from the database
    fn _delete(&self, org_id: &str, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_license_policies::table
                .filter(rbac_license_policies::organization_id.eq(org_id.to_string()))
                .filter(rbac_license_policies::id.eq(id.to_string())))
                .execute(&*connection)
    }

    /// Deletes instance of the license-policy by id from the database
    fn __delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_license_policies::table.find(id.to_string())).execute(&*connection)
    }


    /// Retrieves instance of the license-policy by id from the database
    fn _get(&self, org_id: &str, id: &str) -> Option<PLicensePolicy> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_license_policies::table
                .filter(rbac_license_policies::organization_id.eq(org_id.to_string()))
                .filter(rbac_license_policies::id.eq(id.to_string()))
                .load::<PLicensePolicy>(&*connection) {
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

    /// Retrieves associations by organization-id
    fn _get_by_org(&self, organization_id: &str) -> Vec<PLicensePolicy> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_license_policies::table
                .filter(rbac_license_policies::organization_id.eq(organization_id.to_string()))
                .filter(rbac_license_policies::effective_at.le(now))
                .filter(rbac_license_policies::expired_at.ge(now))
                .load::<PLicensePolicy>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieves instance of the license-policy by id from the database
    fn __get(&self, id: &str) -> Result<PLicensePolicy, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_license_policies::table.find(id.to_string()).get_result::<PLicensePolicy>(&*connection) 
    }

    /// Removes all instances of the license-policy from the database for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        let _ = diesel::delete(rbac_license_policies::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::LicensePolicy;
    use plexrbac::common::SecurityContext;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_license_policy_repository();
        repo.clear();

        let license_policy = repo.create(&ctx, &LicensePolicy::new("", "99", "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let license_policy_str = format!("{:?}", license_policy);

        let loaded = repo.get(&ctx, "99", license_policy.id.as_str()).unwrap();
        assert_eq!(license_policy_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_license_policy_repository();
        repo.clear();

        let license_policy = repo.create(&ctx, &LicensePolicy::new("", "99", "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let mut loaded = repo.get(&ctx, "99", license_policy.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        repo.update(&ctx, &loaded).unwrap();
        let loaded = repo.get(&ctx, "99", loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_get_by_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_license_policy_repository();
        repo.clear();

        let _ = repo.create(&ctx, &LicensePolicy::new("", "99", "default-policy1", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        assert!(repo.create(&ctx, &LicensePolicy::new("", "99", "default-policy2", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).is_err());
        let policies = repo.get_by_org(&ctx, "99");
        assert_eq!(1, policies.len());
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_license_policy_repository();
        repo.clear();

        let license_policy = repo.create(&ctx, &LicensePolicy::new("", "99", "default-policy", Some("desc".to_string()), Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.delete(&ctx, "99", license_policy.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, "99", license_policy.id.as_str());
        assert!(loaded.is_none());
    }
}
