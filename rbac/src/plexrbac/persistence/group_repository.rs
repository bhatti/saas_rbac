//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::{rbac_groups, rbac_group_principals};
use super::models::{PGroup};

//////////////////////////////////////////////////////////////////////////////////////////////
/// GroupRepository defines methods for accessing and persisting groups
///
pub struct GroupRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> GroupRepository<'a> {
    /// Returns groups for given organization
    pub fn get_by_org(&self, organization_id: &str) -> Vec<PGroup> {
        //let orgs = rbac_organizations::table.load::<POrganization>(&*connection)?;
        //let groups = PGroup::belonging_to(&orgs).load::<PGroup>(&*connection)? .grouped_by(&orgs);
        //let data = orgs.into_iter().zip(groups).collect::<Vec<_>>();
        //
        let connection = self.factory.new_connection();
        match rbac_groups::table
            .filter(rbac_groups::organization_id.eq(organization_id.to_string()))
            .load::<PGroup>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Returns group-ids for given principal
    pub fn get_group_ids_by_principal(&self, principal_id: &str) -> Vec<String> {
        let connection = self.factory.new_connection();
        if let Ok(group_ids) = rbac_group_principals::table.select(rbac_group_principals::group_id)
            .filter(rbac_group_principals::principal_id.eq(principal_id.to_string()))
            .group_by(rbac_group_principals::group_id)
            .load::<String>(&*connection) {
                group_ids
        } else {
            vec![]
        }
    }

    /// Returns groups by group-ids
    pub fn get_groups_by_group_ids(&self, group_ids: Vec<String>) -> Vec<PGroup> {
        let connection = self.factory.new_connection();
        match rbac_groups::table
            .filter(rbac_groups::id.eq_any(group_ids))
            .load::<PGroup>(&*connection) {
            Ok(v) => return v,
            _ => return vec![],
        }
    }


    /// Stores new group in the database
    pub fn create(&self, group: &PGroup) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_groups::table).values(group).execute(&*connection) {
            Ok(_) => {
                None
            },
            Err(err) => Some(err),
        }
    }

    /// Updates group in the database
    pub fn update(&self, group: &PGroup) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_groups::table.find(group.id.clone())).set(group).
            execute(&*connection) {
            Ok(_) => {
                None
            },
            Err(err) => Some(err),
        }
    }

    /// Removes all groups in the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_groups::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Removes group in the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_groups::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }
    }

    /// Retrieves group from the database
    pub fn get(&self, id: &str,) -> Option<PGroup> {
        let connection = self.factory.new_connection();
        match rbac_groups::table.find(id.to_string()).get_result::<PGroup>(&*connection) {
            Ok(group) => {
                //let children = PGroup::belonging_to(&group).load::<PGroup>(&*connection)?;
                Some(group)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::persistence::models::{PGroup};
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_repository();
        repo.clear();

        let group = PGroup::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "mygroup", None, None);
        let group_str = format!("{:?}", group);
        repo.create(&group);

        let loaded = repo.get(group.id.as_str()).unwrap();
        assert_eq!(group_str, format!("{:?}", loaded));
        assert_eq!(1, repo.get_groups_by_group_ids(vec![group.id.clone()]).len());
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_repository();
        repo.clear();

        let group = PGroup::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "mygroup", None, None);
        repo.create(&group);

        let mut loaded = repo.get(group.id.as_str()).unwrap();
        loaded.description = Some(String::from("my desc"));
        assert_eq!(None, repo.update(&loaded));
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("my desc".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_repository();
        repo.clear();

        let group = PGroup::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "mygroup", None, None);
        repo.create(&group);
        repo.delete(group.id.as_str());
        let loaded = repo.get(group.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_org() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_group_repository();
        repo.clear();

        repo.create(&PGroup::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "mygroup1", None, None));
        repo.create(&PGroup::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "2", "mygroup2", None, None));

        let results = repo.get_by_org("2");
        assert_eq!(2, results.len());
    }
}
