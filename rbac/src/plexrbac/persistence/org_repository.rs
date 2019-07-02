//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::rbac_organizations;
use super::models::{POrganization};

//////////////////////////////////////////////////////////////////////////////////////////////
/// OrganizationRepository defines methods for accessing and persisting organizations
///
pub struct OrganizationRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> OrganizationRepository<'a> {
    /// Returns all organizations available
    pub fn all(&self) -> Vec<POrganization> {
        let connection = self.factory.new_connection();
        //users::table.select(rbac_organizations::organization_id).inner_join(rbac_groups::table).group_by(rbac_organizations::id).load(&*conn);
        match rbac_organizations::table
            //.inner_join(rbac_groups::table)
            .load::<POrganization>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Creates new instance of the organization
    pub fn create(&self, org: &POrganization) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_organizations::table).values(org).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Updates previous instance of the organization
    pub fn update(&self, org: &POrganization) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_organizations::table.find(org.id.clone())).set(org).
            execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all organizations from the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_organizations::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes an organization by id
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_organizations::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }

    /// Finds an organization by id
    pub fn get(&self, id: &str) -> Option<POrganization> {
        let connection = self.factory.new_connection();
        match rbac_organizations::table.find(id.to_string()).get_result::<POrganization>(&*connection) {
            Ok(org) => {
                //let groups = Group::belonging_to(&org).load::<Group>(&*connection)?;
                //let groups = Group::belonging_to(&vec![org]).select(name).load::<String>(&*connection)?;
                Some(org)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::models::POrganization;
    use plexrbac::persistence::factory::RepositoryFactory;
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_org_repository();
        repo.clear();

        let org = POrganization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "myorganization", "url", None);
        let org_str = format!("{:?}", org);
        repo.create(&org);

        let loaded = repo.get(org.id.as_str()).unwrap();
        assert_eq!(org_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_org_repository();
        repo.clear();

        let org = POrganization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "myorganization", "url", None);
        repo.create(&org);

        let mut loaded = repo.get(org.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_org_repository();
        repo.clear();

        let organization = POrganization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "myorganization", "url", None);
        repo.create(&organization);
        repo.delete(organization.id.as_str());
        let loaded = repo.get(organization.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_all() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_org_repository();
        repo.clear();

        repo.create(&POrganization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "myorganization1", "url", None));
        repo.create(&POrganization::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), None, "myorganization2", "url", None));

        let results = repo.all();
        assert_eq!(2, results.len());
    }
}
