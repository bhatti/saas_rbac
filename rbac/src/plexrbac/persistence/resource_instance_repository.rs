//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::rbac_resource_instances;
use super::models::PResourceInstance;
use plexrbac::domain::models::ResourceInstance;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;


//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceInstanceRepository defines methods for accessing and persisting Resource instances
///
pub struct ResourceInstanceRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceInstanceRepository<'a> {
    /// Creates resource_instance - this should not be called directly but instead called from
    /// Persistence manager that checks quota before creating instance.
    pub fn _create(&self, ctx: &SecurityContext, instance: &ResourceInstance) -> Result<ResourceInstance, RbacError> {
        let mut db_obj = instance.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self.__create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Adding new resource instance {:?}", db_obj), "CREATE");
        Ok(ResourceInstance::from(&db_obj))
    }

    /// Update the resource_instance
    pub fn update(&self, ctx: &SecurityContext, instance: &ResourceInstance) -> Result<ResourceInstance, RbacError> {
        match self._get(instance.id.as_str()) {
            Ok(mut db_obj) => {
                db_obj.status = instance.status.clone();
                db_obj.description = instance.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self.__update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated resource instance {:?}", db_obj), "UPDATE");
                Ok(ResourceInstance::from(&db_obj))
            }
            Err(err) => Err(RbacError::Persistence(err.to_string()))
        }
    }

    /// Retrieves resource_instance by id from the database
    pub fn get(&self, _ctx: &SecurityContext, id: &str) -> Option<ResourceInstance> {
        match self._get(id) {
            Ok(instance) => Some(ResourceInstance::from(&instance)),
            _ => None,
        }
    }

    /// Returns all resource instances for given resource
    pub fn get_by_resource(&self, _ctx: &SecurityContext, resource_id: &str) -> Vec<ResourceInstance> {
        self._get_by_resource(resource_id).iter().map(|r| ResourceInstance::from(&r)).collect::<Vec<ResourceInstance>>()
    }

    /// Returns all resource instances for given resource and scope
    pub fn _get_by_resource_scope(&self, resource_id: &str, scope: &str) -> Vec<PResourceInstance> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resource_instances::table
                .filter(rbac_resource_instances::resource_id.eq(resource_id.to_string()))
                .filter(rbac_resource_instances::scope.eq(scope.to_string()))
                .load::<PResourceInstance>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Count all resource instances for given resource
    pub fn count_by_resource(&self, resource_id: &str, scope: &str, status: &str) -> i64 {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resource_instances::table
                .filter(rbac_resource_instances::resource_id.eq(resource_id.to_string()))
                .filter(rbac_resource_instances::scope.eq(scope.to_string()))
                .filter(rbac_resource_instances::status.eq(status.to_string()))
                .count()
                .get_result(&*connection) {
                Ok(len) => len,
                Err(_) => 0,
            }
        } else {
            0
        }
    }

    pub fn count_recent_by_resource(&self, resource_id: &str, scope: &str, status: &str) -> i64 {
        let recent = Utc::now().naive_utc() - time::Duration::seconds(3600);
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resource_instances::table
                .filter(rbac_resource_instances::resource_id.eq(resource_id.to_string()))
                .filter(rbac_resource_instances::scope.eq(scope.to_string()))
                .filter(rbac_resource_instances::status.eq(status.to_string()))
                .filter(rbac_resource_instances::created_at.ge(recent))
                .count()
                .get_result(&*connection) {
                Ok(len) => len,
                Err(_) => 0,
            }
        } else {
            0
        }
    }


    /// Deletes resource instance by id from the database
    pub fn delete(&self, ctx: &SecurityContext, id: &str) -> Result<usize, RbacError> {
        match self._delete(id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted resource instance {:?}", id), "DELETE");
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

    /// Returns all resource instances for given resource
    fn _get_by_resource(&self, resource_id: &str) -> Vec<PResourceInstance> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resource_instances::table
                .filter(rbac_resource_instances::resource_id.eq(resource_id.to_string()))
                .load::<PResourceInstance>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Stores PResourceInstance instance in the database
    fn __create(&self, instance: &PResourceInstance) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_resource_instances::table).values(instance).execute(&*connection)
    }

    /// Updates PResourceInstance instance in the database
    fn __update(&self, instance: &PResourceInstance) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_resource_instances::table.find(instance.id.clone())).set(instance).
            execute(&*connection)
    }

    /// Deletes instance of resource-instance in the database
    fn _delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_resource_instances::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves resource-instance in the database
    fn _get(&self, id: &str) -> Result<PResourceInstance, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_resource_instances::table.find(id.to_string()).get_result::<PResourceInstance>(&*connection)
    }

    /// Removes all resource-instances in the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        let _ = diesel::delete(rbac_resource_instances::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::ResourceInstance;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_instance_repository();
        repo.clear();

        let instance = repo._create(&ctx, &ResourceInstance::new("", "11", "22", "", "refid", "INFLIGHT", Some("blah".to_string()))).unwrap();
        let instance_str = format!("{:?}", instance);

        let loaded = repo.get(&ctx, instance.id.as_str()).unwrap();
        assert_eq!(instance_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_instance_repository();
        repo.clear();

        let instance = repo._create(&ctx, &ResourceInstance::new("", "11", "22", "", "refid", "INFLIGHT", Some("blah".to_string()))).unwrap();

        let mut loaded = repo.get(&ctx, instance.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert!(repo.update(&ctx, &loaded).is_ok());
        let loaded = repo.get(&ctx, loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_instance_repository();
        repo.clear();

        let instance = repo._create(&ctx, &ResourceInstance::new("", "11", "22", "", "refid", "INFLIGHT", Some("blah".to_string()))).unwrap();
        repo.delete(&ctx, instance.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, instance.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_resource() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_instance_repository();
        repo.clear();

        let _ = repo._create(&ctx, &ResourceInstance::new("", "11", "22", "", "refid1", "INFLIGHT", Some("blah".to_string()))).unwrap();
        let _ = repo._create(&ctx, &ResourceInstance::new("", "11", "22", "", "refid2", "INFLIGHT", Some("blah".to_string()))).unwrap();

        let results = repo._get_by_resource_scope("11", "");
        assert_eq!(2, results.len());
        let results = repo.get_by_resource(&ctx, "11");
        assert_eq!(2, results.len());
    }
}
