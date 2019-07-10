//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::rbac_organizations;
use super::models::{POrganization};
use plexrbac::domain::models::Organization;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;

//////////////////////////////////////////////////////////////////////////////////////////////
/// OrganizationRepository defines methods for accessing and persisting organizations
///
pub struct OrganizationRepository<'a> {
    pub data_source: &'a dyn super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> OrganizationRepository<'a> {
    /// Creates an organization
    pub fn create(&self, ctx: &SecurityContext, org: &Organization) -> Result<Organization, RbacError> {
        let mut db_obj = org.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Created new org {:?}", db_obj), "CREATE");
        Ok(Organization::from(&db_obj))
    }

    /// Updates an organization
    pub fn update(&self, ctx: &SecurityContext, org: &Organization) -> Result<Organization, RbacError> {
        match self._get(org.id.as_str()) {
            Ok(mut db_obj) => {
                db_obj.url = org.url.clone();
                db_obj.description = org.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated org {:?}", db_obj), "UPDATE");
                Ok(Organization::from(&db_obj))
            }
            Err(err) => Err(RbacError::Persistence(err.to_string()))
        }
    }

    /// Retrieves all organizations from the database
    pub fn all(&self, _ctx: &SecurityContext) -> Vec<Organization> {
        self._all().iter().map(|org| Organization::from(&org)).collect::<Vec<Organization>>()
    }

    /// Retrieves organization by org-id from the database
    pub fn get(&self, _ctx: &SecurityContext, organization_id: &str) -> Option<Organization> {
        if let Ok(porg) = self._get(organization_id) {
            Some(Organization::from(&porg))
        } else {
            None
        }
    }

    /// Deletes realm by id from the database
    pub fn delete(&self, ctx: &SecurityContext, id: &str) -> Result<usize, RbacError> {
        match self._delete(id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted organization {:?}", id), "DELETE");
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

    /// Deletes an organization by id
    fn _delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_organizations::table.find(id.to_string())).execute(&*connection)
    }

    /// Returns all organizations available
    fn _all(&self) -> Vec<POrganization> {
        if let Ok(connection) = self.data_source.new_connection() {
            //users::table.select(rbac_organizations::organization_id).inner_join(rbac_groups::table).group_by(rbac_organizations::id).load(&conn);
            match rbac_organizations::table
                //.inner_join(rbac_groups::table)
                .load::<POrganization>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Creates new instance of the organization
    fn _create(&self, org: &POrganization) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_organizations::table).values(org).execute(&*connection)
    }

    /// Updates previous instance of the organization
    fn _update(&self, org: &POrganization) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_organizations::table.find(org.id.clone())).set(org).
            execute(&*connection)
    }

    /// Finds an organization by id
    fn _get(&self, id: &str) -> Result<POrganization, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_organizations::table.find(id.to_string()).get_result::<POrganization>(&*connection)
        //let groups = Group::belonging_to(&org).load::<Group>(&*connection)?;
        //let groups = Group::belonging_to(&vec![org]).select(name).load::<String>(&*connection)?;
    }

    /// Removes all organizations from the database - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        let _ = diesel::delete(rbac_organizations::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::Organization;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_create() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_org_repository();
        repo.clear();

        let org = repo.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let org_str = format!("{:?}", org);
        let loaded = repo.get(&ctx, org.id.as_str()).unwrap();
        assert_eq!(org_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_org_repository();
        repo.clear();

        let org = repo.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        let mut loaded = repo.get(&ctx, org.id.as_str()).unwrap();
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
        let repo = locator.new_org_repository();
        repo.clear();

        let org = repo.create(&ctx, &Organization::new("", None, "myorg", "url", None)).unwrap();
        repo.delete(&ctx, org.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, org.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_all() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_org_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Organization::new("", None, "myorg1", "url", None)).unwrap();
        let _ = repo.create(&ctx, &Organization::new("", None, "myorg2", "url", None)).unwrap();

        let results = repo.all(&ctx);
        assert_eq!(2, results.len());
    }
}
