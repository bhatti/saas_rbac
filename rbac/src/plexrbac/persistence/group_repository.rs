//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::{rbac_groups, rbac_group_principals};
use super::models::{PGroup};
use plexrbac::domain::models::Group;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////////////////////
/// GroupRepository defines methods for accessing and persisting groups
///
pub struct GroupRepository<'a> {
    pub data_source: &'a dyn super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> GroupRepository<'a> {
    /// Creates a group
    pub fn create(&self, ctx: &SecurityContext, group: &Group) -> Result<Group, RbacError> {
        let mut db_obj = group.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Created new group {:?}", db_obj), "CREATE");
        Ok(Group::from(&db_obj))
    }

    /// Updated the group
    pub fn update (&self, ctx: &SecurityContext, group: &Group) -> Result<Group, RbacError> {
        match self._get(group.organization_id.as_str(), group.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.parent_id = group.parent_id.clone();
                db_obj.description = group.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated group {:?}", db_obj), "UPDATE");
                Ok(Group::from(&db_obj))
            }
            None => Err(RbacError::NotFound(format!("Group not found {:?}", group)))
        }
    }

    /// Returns all groups for given organization
    pub fn get_by_org(&self, _ctx: &SecurityContext, organization_id: &str) -> HashMap<String, Group> {
        let mut groups = HashMap::new();
        for pgroup in &self._get_by_org(organization_id) {
            let group = Group::from(pgroup); 
            groups.insert(group.id.clone(), group);
        }
        groups
    }

    /// Returns groups by group-ids
    fn get_by_group_ids(&self, _ctx: &SecurityContext, group_ids: Vec<String>) -> Vec<Group> {
        self._get_by_group_ids(group_ids).iter().map(|g| Group::from(&g)).collect::<Vec<Group>>()
    }


    /// Retrieves group by id from the database
    pub fn get(&self, _ctx: &SecurityContext, org_id: &str, id: &str) -> Option<Group> {
        match self._get(org_id, id) {
            Some(group) => Some(Group::from(&group)),
            _ => None,
        }
    }

    /// Deletes group by id from the database
    pub fn delete(&self, ctx: &SecurityContext, org_id: &str, id: &str) -> Result<usize, RbacError> {
        match self._delete(org_id, id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted group {:?}", id), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Returns group-ids for given principal
    pub fn get_group_ids_by_principal(&self, _ctx: &SecurityContext, principal_id: &str) -> Vec<String> {
        if let Ok(connection) = self.data_source.new_connection() {
            if let Ok(group_ids) = rbac_group_principals::table.select(rbac_group_principals::group_id)
                .filter(rbac_group_principals::principal_id.eq(principal_id.to_string()))
                .group_by(rbac_group_principals::group_id)
                .load::<String>(&*connection) {
                    group_ids
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    ///////////////////////////////////// PRIVATE METHODS ////////////////////////////////
    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }

    /// Returns groups for given organization
    pub fn _get_by_org(&self, organization_id: &str) -> Vec<PGroup> {
        //let orgs = rbac_organizations::table.load::<POrganization>(&*connection)?;
        //let groups = PGroup::belonging_to(&orgs).load::<PGroup>(&*connection)? .grouped_by(&orgs);
        //let data = orgs.into_iter().zip(groups).collect::<Vec<_>>();
        //
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_groups::table
                .filter(rbac_groups::organization_id.eq(organization_id.to_string()))
                .load::<PGroup>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Returns groups by group-ids
    fn _get_by_group_ids(&self, group_ids: Vec<String>) -> Vec<PGroup> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_groups::table
                .filter(rbac_groups::id.eq_any(group_ids))
                .load::<PGroup>(&*connection) {
                Ok(v) => return v,
                _ => return vec![],
            }
        } else {
            vec![]
        }
    }


    /// Stores new group in the database
    fn _create(&self, group: &PGroup) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_groups::table).values(group).execute(&*connection)
    }

    /// Updates group in the database
    fn _update(&self, group: &PGroup) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_groups::table.find(group.id.clone())).set(group).
            execute(&*connection)
    }

    /// Removes instance of claim in the database
    fn _delete(&self, org_id: &str, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_groups::table
                .filter(rbac_groups::organization_id.eq(org_id.to_string()))
                .filter(rbac_groups::id.eq(id.to_string())))
                .execute(&*connection)
    }

    /// Removes group in the database
    fn __delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_groups::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves group from the database
     fn _get(&self, org_id: &str, id: &str) -> Option<PGroup> {
         if let Ok(connection) = self.data_source.new_connection() {
             match rbac_groups::table
                 .filter(rbac_groups::organization_id.eq(org_id.to_string()))
                 .filter(rbac_groups::id.eq(id.to_string()))
                 .load::<PGroup>(&*connection) {
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

    /// Retrieves group from the database
    fn __get(&self, id: &str) -> Result<PGroup, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_groups::table.find(id.to_string()).get_result::<PGroup>(&*connection)
    }

    /// Removes all groups in the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_groups::table).execute(&*connection).unwrap();
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::Group;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_repository();
        repo.clear();

        let group = repo.create(&ctx, &Group::new("", "2", "mygroup", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let group_str = format!("{:?}", group);

        let loaded = repo.get(&ctx, "2", group.id.as_str()).unwrap();
        assert_eq!(group_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_repository();
        repo.clear();

        let group = repo.create(&ctx, &Group::new("", "2", "mygroup", Some("desc".to_string()), Some("parent".to_string()))).unwrap();

        let mut loaded = repo.get(&ctx, "2", group.id.as_str()).unwrap();
        loaded.description = Some(String::from("my desc"));
        assert!(repo.update(&ctx, &loaded).is_ok());
        let loaded = repo.get(&ctx, "2", loaded.id.as_str()).unwrap();
        assert_eq!(Some("my desc".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_repository();
        repo.clear();

        let group = repo.create(&ctx, &Group::new("", "2", "mygroup", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        repo.delete(&ctx, "2", group.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, "2", group.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_group_ids() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_repository();
        repo.clear();

        let group1 = repo.create(&ctx, &Group::new("", "2", "mygroup1", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let group2 = repo.create(&ctx, &Group::new("", "2", "mygroup2", Some("desc".to_string()), Some("parent".to_string()))).unwrap();

        assert_eq!(2, repo.get_by_group_ids(&ctx, vec![group1.id.clone(), group2.id.clone()]).len());
    }

    #[test]
    fn test_get_by_org() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_group_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Group::new("", "2", "mygroup1", Some("desc".to_string()), Some("parent".to_string()))).unwrap();
        let _ = repo.create(&ctx, &Group::new("", "2", "mygroup2", Some("desc".to_string()), Some("parent".to_string()))).unwrap();

        let results = repo.get_by_org(&ctx, "2");
        assert_eq!(2, results.len());
    }
}
