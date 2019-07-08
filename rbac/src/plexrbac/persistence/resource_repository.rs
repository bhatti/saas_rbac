//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::rbac_resources;
use super::models::PResource;
use plexrbac::domain::models::Resource;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceRepository defines methods for accessing and persisting PResource
///
pub struct ResourceRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceRepository<'a> {
    /// Creates resource
    pub fn create(&self, ctx: &SecurityContext, resource: &Resource) -> Result<Resource, RbacError> {
        let mut db_obj = resource.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Adding resource {:?}", db_obj), "CREATE");
        Ok(Resource::from(&db_obj))
    }

    /// Creates resource
    pub fn update(&self, ctx: &SecurityContext, resource: &Resource) -> Result<Resource, RbacError> {
        match self._get(resource.realm_id.as_str(), resource.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.allowable_actions = resource.allowable_actions .clone();
                db_obj.description = resource.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated resource {:?}", db_obj), "UPDATE");
                Ok(Resource::from(&db_obj))
            }
            None => Err(RbacError::NotFound(format!("Resource not found {:?}", resource)))
        }
    }

    /// Retrieves resource by id from the database
    pub fn get(&self, _ctx: &SecurityContext, realm_id: &str, resource_id: &str) -> Option<Resource> {
        match self._get(realm_id, resource_id) {
            Some(resource) => Some(Resource::from(&resource)),
            _ => None,
        }
    }

    /// Returns all resources for given security realm
    pub fn get_by_realm(&self, _ctx: &SecurityContext, realm_id: &str) -> HashMap<String, Resource> {
        let mut resources = HashMap::new();
        for presource in &self._get_by_realm(realm_id) {
            let resource = Resource::from(presource); 
            resources.insert(resource.id.clone(), resource);
        }
        resources
    }

    /// Deletes resource by id from the database
    pub fn delete(&self, ctx: &SecurityContext, realm_id: &str, id: &str) -> Result<usize, RbacError> {
        match self._delete(realm_id, id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted resource {:?}", id), "DELETE");
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

    /// Stores PResource instance in the database
    fn _create(&self, resource: &PResource) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_resources::table).values(resource).execute(&*connection)
    }

    /// Updates PResource instance in the database
    fn _update(&self, resource: &PResource) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_resources::table.find(resource.id.clone())).set(resource).
            execute(&*connection)
    }

    /// Removes resource in the database
    fn _delete(&self, realm_id: &str, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_resources::table
                .filter(rbac_resources::realm_id.eq(realm_id.to_string()))
                .filter(rbac_resources::id.eq(id.to_string())))
                .execute(&*connection)
    }

    /// Removes resource in the database
    fn __delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_resources::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves resource in the database
    fn __get(&self, id: &str) -> Result<PResource, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_resources::table.find(id.to_string()).get_result::<PResource>(&*connection)
        //let children = ResourceInstance::belonging_to(&resource).load::<PResourceInstance>(&*connection)?;
    }

    /// Retrieves resource in the database
    fn _get(&self, realm_id: &str, id: &str) -> Option<PResource> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resources::table
                .filter(rbac_resources::realm_id.eq(realm_id.to_string()))
                .filter(rbac_resources::id.eq(id.to_string()))
                .load::<PResource>(&*connection) {
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

    /// Returns all resources for given security realm
    fn _get_by_realm(&self, realm_id: &str) -> Vec<PResource> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resources::table
                .filter(rbac_resources::realm_id.eq(realm_id.to_string()))
                .load::<PResource>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieves associations by resource-id
    pub fn _get_by_ids(&self, resource_ids: Vec<String>) -> Vec<PResource> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_resources::table
                .filter(rbac_resources::id.eq_any(resource_ids))
                .load::<PResource>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Removes all instances in the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_resources::table).execute(&*connection).unwrap();
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::Resource;
    use plexrbac::common::SecurityContext;


    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_repository();
        repo.clear();

        let resource = repo.create(&ctx, &Resource::new("", "1", "app", None, None)).unwrap();
        let resource_str = format!("{:?}", resource);
        let loaded = repo.get(&ctx, "1", resource.id.as_str()).unwrap();
        assert_eq!(resource_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_repository();
        repo.clear();

        let resource = repo.create(&ctx, &Resource::new("", "1", "app", None, None)).unwrap();
        let mut loaded = repo.get(&ctx, "1", resource.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert!(repo.update(&ctx, &loaded).is_ok());
        let loaded = repo.get(&ctx, "1", loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_get_by_ids() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_repository();
        repo.clear();

        let resource1 = repo.create(&ctx, &Resource::new("", "1", "app1", None, None)).unwrap();
        let resource2 = repo.create(&ctx, &Resource::new("", "1", "app2", None, None)).unwrap();
        let all = repo._get_by_ids(vec![resource1.id.clone(), resource2.id.clone()]);
        assert_eq!(2, all.len());
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_repository();
        repo.clear();

        let resource = repo.create(&ctx, &Resource::new("", "1", "app", None, None)).unwrap();
        repo.delete(&ctx, "1", resource.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, "1", resource.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_realm() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_resource_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Resource::new("", "1", "app1", None, None)).unwrap();
        let _ = repo.create(&ctx, &Resource::new("", "1", "app2", None, None)).unwrap();

        let results = repo.get_by_realm(&ctx, "1");
        assert_eq!(2, results.len());
    }
}
