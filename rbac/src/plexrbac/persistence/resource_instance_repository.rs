//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_resource_instances;
use super::models::PResourceInstance;

//////////////////////////////////////////////////////////////////////////////////////////////
/// ResourceInstanceRepository defines methods for accessing and persisting Resource instances
///
pub struct ResourceInstanceRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ResourceInstanceRepository<'a> {
    /// Returns all resource instances for given resource
    pub fn get_by_resource(&self, resource_id: &str, scope: &str) -> Vec<PResourceInstance> {
        let connection = self.factory.new_connection();
        match rbac_resource_instances::table
            .filter(rbac_resource_instances::resource_id.eq(resource_id.to_string()))
            .filter(rbac_resource_instances::scope.eq(scope.to_string()))
            .load::<PResourceInstance>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Count all resource instances for given resource
    pub fn count_by_resource(&self, resource_id: &str, scope: &str) -> i64 {
        let connection = self.factory.new_connection();
        match rbac_resource_instances::table
            .filter(rbac_resource_instances::resource_id.eq(resource_id.to_string()))
            .filter(rbac_resource_instances::scope.eq(scope.to_string()))
            .count()
            .get_result(&*connection) {
            Ok(len) => len,
            Err(_) => 0,
        }
    }

    /// Stores PResourceInstance instance in the database
    pub fn create(&self, instance: &PResourceInstance) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_resource_instances::table).values(instance).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates PResourceInstance instance in the database
    pub fn update(&self, instance: &PResourceInstance) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_resource_instances::table.find(instance.id.clone())).set(instance).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all resource-instances in the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resource_instances::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes instance of resource-instance in the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_resource_instances::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Retrieves resource-instance in the database
    pub fn get(&self, id: &str) -> Option<PResourceInstance> {
        let connection = self.factory.new_connection();
        match rbac_resource_instances::table.find(id.to_string()).get_result::<PResourceInstance>(&*connection) {
            Ok(instance) => {
                Some(instance)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::PResourceInstance;
    use plexrbac::persistence::factory::RepositoryFactory;
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_instance_repository();
        repo.clear();

        let instance = PResourceInstance::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", "ref", "INFLIGHT", None);
        let instance_str = format!("{:?}", instance);
        repo.create(&instance);

        let loaded = repo.get(instance.id.as_str()).unwrap();
        assert_eq!(instance_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_instance_repository();
        repo.clear();

        let instance = PResourceInstance::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", "ref", "INFLIGHT", None);
        repo.create(&instance);

        let mut loaded = repo.get(instance.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_instance_repository();
        repo.clear();

        let instance = PResourceInstance::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "11", "22", "", "ref", "INFLIGHT", None);
        repo.create(&instance);
        repo.delete(instance.id.as_str());
        let loaded = repo.get(instance.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_resource() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_resource_instance_repository();
        repo.clear();

        repo.create(&PResourceInstance::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "911", "22", "", "ref", "INFLIGHT", None));
        repo.create(&PResourceInstance::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "911", "33", "", "ref", "INFLIGHT", None));

        let results = repo.get_by_resource("911", "");
        assert_eq!(2, results.len());
    }
}
