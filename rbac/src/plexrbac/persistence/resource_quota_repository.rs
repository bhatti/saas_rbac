//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::rbac_resource_quotas;
use super::models::PResourceQuota;
use plexrbac::domain::models::ResourceQuota;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;

//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceQuotaRepository defines methods for accessing and persisting Resource quotas
///
pub struct ResourceQuotaRepository<'a> {
    pub data_source: &'a dyn super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceQuotaRepository<'a> {
    /// Creates resource_quota
    pub fn create(&self, ctx: &SecurityContext, quota: &ResourceQuota) -> Result<ResourceQuota, RbacError> {
        let mut db_obj = quota.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Adding new resource quota {:?}", db_obj), "CREATE");
        Ok(ResourceQuota::from(&db_obj))
    }

    /// Updates the resource_quota
    pub fn update(&self, ctx: &SecurityContext, quota: &ResourceQuota) -> Result<ResourceQuota, RbacError> {
        match self._get(quota.id.as_str()) {
            Ok(mut db_obj) => {
                db_obj.effective_at = quota.effective_at.clone();
                db_obj.expired_at= quota.expired_at.clone();
                db_obj.max_value = quota.max_value.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated new resource quota {:?}", db_obj), "UPDATE");
                Ok(ResourceQuota::from(&db_obj))
            }
            Err(err) => Err(RbacError::Persistence(err.to_string()))
        }
    }

    /// Retrieves resource_quota by id from the database
    pub fn get(&self, _ctx: &SecurityContext, id: &str) -> Option<ResourceQuota> {
        match self._get(id) {
            Ok(quota) => Some(ResourceQuota::from(&quota)),
            _ => None,
        }
    }

    /// Returns all resource quotas for given resource
    pub fn get_by_resource(&self, _ctx: &SecurityContext, resource_id: &str) -> Vec<ResourceQuota> {
        self._get_by_resource(resource_id).iter().map(|g| ResourceQuota::from(&g)).collect::<Vec<ResourceQuota>>()
    }

    /// Returns all resource quotas for given resource and scope
    pub fn get_by_resource_scope(&self, resource_id: &str, scope: &str) -> Vec<PResourceQuota> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resource_quotas::table
                .filter(rbac_resource_quotas::resource_id.eq(resource_id.to_string()))
                .filter(rbac_resource_quotas::scope.eq(scope.to_string()))
                .filter(rbac_resource_quotas::effective_at.le(now))
                .filter(rbac_resource_quotas::expired_at.ge(now))
                .load::<PResourceQuota>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Deletes resource quota by id from the database
    pub fn delete(&self, ctx: &SecurityContext, id: &str) -> Result<usize, RbacError> {
        match self._delete(id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted resource quota {:?}", id), "DELETE");
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

    /// Stores PResourceQuota quota in the database
    fn _create(&self, quota: &PResourceQuota) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_resource_quotas::table).values(quota).execute(&*connection)
    }

    /// Updates PResourceQuota quota in the database
    fn _update(&self, quota: &PResourceQuota) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_resource_quotas::table.find(quota.id.clone())).set(quota).
            execute(&*connection)
    }

    /// Deletes quota of resource-quota in the database
    fn _delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_resource_quotas::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves resource-quota in the database
    fn _get(&self, id: &str) -> Result<PResourceQuota, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_resource_quotas::table.find(id.to_string()).get_result::<PResourceQuota>(&*connection)
    }

    /// Returns all resource quotas for given resource
    fn _get_by_resource(&self, resource_id: &str) -> Vec<PResourceQuota> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resource_quotas::table
                .filter(rbac_resource_quotas::resource_id.eq(resource_id.to_string()))
                .filter(rbac_resource_quotas::effective_at.le(now))
                .filter(rbac_resource_quotas::expired_at.ge(now))
                .load::<PResourceQuota>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }


    /// Removes all resource-quotas in the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_resource_quotas::table).execute(&*connection).unwrap();
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use chrono::{NaiveDate, Utc};
    use plexrbac::domain::models::ResourceQuota;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_quota_repository();
        repo.clear();

        let quota  = repo.create(&ctx, &ResourceQuota::new("", "11", "22", "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let quota_str = format!("{:?}", quota);

        let loaded = repo.get(&ctx, quota.id.as_str()).unwrap();
        assert_eq!(quota_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_quota_repository();
        repo.clear();

        let quota  = repo.create(&ctx, &ResourceQuota::new("", "11", "22", "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let mut loaded = repo.get(&ctx, quota.id.as_str()).unwrap();
        loaded.max_value = 30;
        assert!(repo.update(&ctx, &loaded).is_ok());
        let loaded = repo.get(&ctx, loaded.id.as_str()).unwrap();
        assert_eq!(30, loaded.max_value);
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_quota_repository();
        repo.clear();

        let quota  = repo.create(&ctx, &ResourceQuota::new("", "11", "22", "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.delete(&ctx, quota.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, quota.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_resource() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_quota_repository();
        repo.clear();

        let _ = repo.create(&ctx, &ResourceQuota::new("", "11", "22", "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let _ = repo.create(&ctx, &ResourceQuota::new("", "11", "23", "", 2, Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        let results = repo.get_by_resource_scope("11", "");
        assert_eq!(2, results.len());
        let results = repo.get_by_resource(&ctx, "11");
        assert_eq!(2, results.len());
    }
}
