//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_resources;
use super::models::PResource;

//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceRepository defines methods for accessing and persisting PResource
///
pub struct ResourceRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceRepository<'a> {
    /// Returns all resources for given security realm
    pub fn get_by_realm(&self, realm_id: &str) -> Vec<PResource> {
        let connection = self.factory.new_connection();
        match rbac_resources::table
            .filter(rbac_resources::realm_id.eq(realm_id.to_string()))
            .load::<PResource>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Stores PResource instance in the database
    pub fn create(&self, resource: &PResource) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_resources::table).values(resource).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates PResource instance in the database
    pub fn update(&self, resource: &PResource) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_resources::table.find(resource.id.clone())).set(resource).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all instances in the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resources::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Removes instance of resource in the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resources::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Retrieves instance of resource in the database
    pub fn get(&self, id: &str) -> Option<PResource> {
        let connection = self.factory.new_connection();
        match rbac_resources::table.find(id.to_string()).get_result::<PResource>(&*connection) {
            Ok(resource) => {
                //let children = ResourceInstance::belonging_to(&resource).load::<PResourceInstance>(&*connection)?;
                Some(resource)
            },
            Err(_) => None,
        }
    }

    /// Retrieves associations by resource-id
    pub fn get_by_ids(&self, resource_ids: Vec<String>) -> Vec<PResource> {
        let connection = self.factory.new_connection();
        match rbac_resources::table
            .filter(rbac_resources::id.eq_any(resource_ids))
            .load::<PResource>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::PResource;
    use plexrbac::persistence::factory::RepositoryFactory;
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_repository();
        repo.clear();

        let resource = PResource::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "app", None, None);
        let resource_str = format!("{:?}", resource);
        repo.create(&resource);
        let loaded = repo.get(resource.id.as_str()).unwrap();
        assert_eq!(resource_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_repository();
        repo.clear();

        let resource = PResource::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "app", None, None);
        repo.create(&resource);
        let mut loaded = repo.get(resource.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_get_by_ids() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_repository();
        repo.clear();

        repo.create(&PResource::new("9", "1", "app", None, None));
        repo.create(&PResource::new("10", "2", "app", None, None));
        let all = repo.get_by_ids(vec!["9".to_string(),"10".to_string()]);
        assert_eq!(2, all.len());
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_repository();
        repo.clear();

        let resource = PResource::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "app", None, None);
        repo.create(&resource);
        repo.delete(resource.id.as_str());
        let loaded = repo.get(resource.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_realm() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_repository();
        repo.clear();

        repo.create(&PResource::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "app", None, None));
        repo.create(&PResource::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "1", "report", None, None));

        let results = repo.get_by_realm("1");
        assert_eq!(2, results.len());
    }
}
