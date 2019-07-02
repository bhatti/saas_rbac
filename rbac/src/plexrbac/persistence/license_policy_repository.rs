//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_license_policies;
use super::models::{PLicensePolicy};
use chrono::{Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// LicensePolicyRepository defines methods for accessing and persisting license-policies that
/// defines overall access policies for customers
///
pub struct LicensePolicyRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> LicensePolicyRepository<'a> {
    /// Creates instance of license-policy
    pub fn create(&self, license_policy: &PLicensePolicy) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_license_policies::table).values(license_policy).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates previous instance of the license-policy
    pub fn update(&self, license_policy: &PLicensePolicy) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_license_policies::table.find(license_policy.id.clone())).set(license_policy).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all instances of the license-policy from the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_license_policies::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes instance of the license-policy by id from the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_license_policies::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }
    }

    /// Retrieves instance of the license-policy by id from the database
    pub fn get(&self, id: &str) -> Option<PLicensePolicy> {
        let connection = self.factory.new_connection();
        match rbac_license_policies::table.find(id.to_string()).get_result::<PLicensePolicy>(&*connection) {
            Ok(license_policy) => {
                Some(license_policy)
            },
            Err(_) => None,
        }
    }

    /// Retrieves associations by organization-id
    pub fn get_by_org(&self, organization_id: &str) -> Vec<PLicensePolicy> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_license_policies::table
            .filter(rbac_license_policies::organization_id.eq(organization_id.to_string()))
            .filter(rbac_license_policies::effective_at.le(now))
            .filter(rbac_license_policies::expired_at.ge(now))
            .load::<PLicensePolicy>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::persistence::models::{PLicensePolicy};
    use self::uuu::Uuid;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_license_policy_repository();
        repo.clear();

        let license_policy = PLicensePolicy::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "mylicense_policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        let license_policy_str = format!("{:?}", license_policy);
        repo.create(&license_policy);

        let loaded = repo.get(license_policy.id.as_str()).unwrap();
        assert_eq!(license_policy_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_license_policy_repository();
        repo.clear();

        let license_policy = PLicensePolicy::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "mylicense_policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&license_policy);
        let mut loaded = repo.get(license_policy.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        repo.update(&loaded);
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_get_by_org() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_license_policy_repository();
        repo.clear();

        repo.create(&PLicensePolicy::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "mylicense_policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PLicensePolicy::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "mylicense_policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));

        let policies = repo.get_by_org("99");
        assert_eq!(2, policies.len());
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_license_policy_repository();
        repo.clear();

        let license_policy = PLicensePolicy::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "mylicense_policy", None, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&license_policy);
        repo.delete(license_policy.id.as_str());
        let loaded = repo.get(license_policy.id.as_str());
        assert_eq!(None, loaded);
    }
}
