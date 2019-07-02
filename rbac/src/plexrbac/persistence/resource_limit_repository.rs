//#![crate_name = "doc"]
use diesel::prelude::*;
use super::schema::rbac_resource_limits;
use super::models::PResourceLimit;
use chrono::{Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceLimitRepository defines methods for accessing and persisting Resource limits
///
pub struct ResourceLimitRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceLimitRepository<'a> {
    /// Returns all resource limits for given resource
    pub fn get_by_resource(&self, resource_id: &str) -> Vec<PResourceLimit> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_resource_limits::table
            .filter(rbac_resource_limits::resource_id.eq(resource_id.to_string()))
            .filter(rbac_resource_limits::effective_at.le(now))
            .filter(rbac_resource_limits::expired_at.ge(now))
            .load::<PResourceLimit>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Stores PResourceLimit limit in the database
    pub fn create(&self, limit: &PResourceLimit) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_resource_limits::table).values(limit).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates PResourceLimit limit in the database
    pub fn update(&self, limit: &PResourceLimit) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_resource_limits::table.find(limit.id.clone())).set(limit).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all resource-limits in the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resource_limits::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes limit of resource-limit in the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resource_limits::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Retrieves resource-limit in the database
    pub fn get(&self, id: &str) -> Option<PResourceLimit> {
        let connection = self.factory.new_connection();
        match rbac_resource_limits::table.find(id.to_string()).get_result::<PResourceLimit>(&*connection) {
            Ok(limit) => {
                Some(limit)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::PResourceLimit;
    use plexrbac::persistence::factory::RepositoryFactory;
    use chrono::{NaiveDate, Utc};
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_limit_repository();
        repo.clear();

        let limit = PResourceLimit::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "report", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        let limit_str = format!("{:?}", limit);
        repo.create(&limit);

        let loaded = repo.get(limit.id.as_str()).unwrap();
        assert_eq!(limit_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_limit_repository();
        repo.clear();

        let limit = PResourceLimit::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "report", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&limit);

        let mut loaded = repo.get(limit.id.as_str()).unwrap();
        loaded.max_value = 30;
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(30, loaded.max_value);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_limit_repository();
        repo.clear();

        let limit = PResourceLimit::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "report", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&limit);
        repo.delete(limit.id.as_str());
        let loaded = repo.get(limit.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_resource() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_limit_repository();
        repo.clear();

        repo.create(&PResourceLimit::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "report", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PResourceLimit::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "33", "report", 0, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));

        let results = repo.get_by_resource("11");
        assert_eq!(2, results.len());
    }
}
