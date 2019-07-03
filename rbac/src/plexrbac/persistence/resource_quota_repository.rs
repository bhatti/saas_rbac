//#![crate_name = "doc"]
use diesel::prelude::*;
use super::schema::rbac_resource_quotas;
use super::models::PResourceQuota;
use chrono::{Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceQuotaRepository defines methods for accessing and persisting Resource quotas
///
pub struct ResourceQuotaRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceQuotaRepository<'a> {
    /// Returns all resource quotas for given resource
    pub fn get_by_resource(&self, resource_id: &str, scope: &str) -> Vec<PResourceQuota> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_resource_quotas::table
            .filter(rbac_resource_quotas::resource_id.eq(resource_id.to_string()))
            .filter(rbac_resource_quotas::scope.eq(scope.to_string()))
            .filter(rbac_resource_quotas::effective_at.le(now))
            .filter(rbac_resource_quotas::expired_at.ge(now))
            .load::<PResourceQuota>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Stores PResourceQuota quota in the database
    pub fn create(&self, quota: &PResourceQuota) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_resource_quotas::table).values(quota).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates PResourceQuota quota in the database
    pub fn update(&self, quota: &PResourceQuota) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_resource_quotas::table.find(quota.id.clone())).set(quota).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all resource-quotas in the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resource_quotas::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes quota of resource-quota in the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resource_quotas::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Retrieves resource-quota in the database
    pub fn get(&self, id: &str) -> Option<PResourceQuota> {
        let connection = self.factory.new_connection();
        match rbac_resource_quotas::table.find(id.to_string()).get_result::<PResourceQuota>(&*connection) {
            Ok(quota) => {
                Some(quota)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::PResourceQuota;
    use plexrbac::persistence::factory::RepositoryFactory;
    use chrono::{NaiveDate, Utc};
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_quota_repository();
        repo.clear();

        let quota = PResourceQuota::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        let quota_str = format!("{:?}", quota);
        repo.create(&quota);

        let loaded = repo.get(quota.id.as_str()).unwrap();
        assert_eq!(quota_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_quota_repository();
        repo.clear();

        let quota = PResourceQuota::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&quota);

        let mut loaded = repo.get(quota.id.as_str()).unwrap();
        loaded.max_value = 30;
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(30, loaded.max_value);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_quota_repository();
        repo.clear();

        let quota = PResourceQuota::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&quota);
        repo.delete(quota.id.as_str());
        let loaded = repo.get(quota.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_resource() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_quota_repository();
        repo.clear();

        repo.create(&PResourceQuota::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PResourceQuota::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "33", "", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));

        let results = repo.get_by_resource("11", "");
        assert_eq!(2, results.len());
    }
}
