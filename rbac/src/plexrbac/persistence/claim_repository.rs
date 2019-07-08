//#![crate_name = "doc"]
extern crate uuid as uuu;

use diesel::prelude::*;
use super::schema::{rbac_claims};
use super::models::{PClaim};
use plexrbac::domain::models::Claim;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{Utc};
use self::uuu::Uuid;
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////////////////////
/// ClaimRepository defines methods for accessing and persisting Claim
///
pub struct ClaimRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ClaimRepository<'a> {
    /// Creates claim
    pub fn create(&self, ctx: &SecurityContext, claim: &Claim) -> Result<Claim, RbacError> {
        let mut db_obj = claim.to();
        db_obj.id = Uuid::new_v4().to_hyphenated().to_string();
        db_obj.created_at = Some(Utc::now().naive_utc());
        db_obj.created_by = Some(ctx.principal_id.clone());
        db_obj.updated_at = Some(Utc::now().naive_utc());
        db_obj.updated_by = Some(ctx.principal_id.clone());
        if let Err(err) = self._create(&db_obj) {
            return Err(RbacError::Persistence(err.to_string()));
        }
        self.audit(ctx, format!("Adding new claim {:?}", db_obj), "CREATE");
        Ok(Claim::from(&db_obj))
    }

    /// Updates the claim
    pub fn update(&self, ctx: &SecurityContext, claim: &Claim) -> Result<Claim, RbacError> {
        match self._get(claim.realm_id.as_str(), claim.resource_id.as_str(), claim.id.as_str()) {
            Some(mut db_obj) => {
                db_obj.action = claim.action.clone();
                db_obj.effect = claim.effect.clone();
                db_obj.description = claim.description.clone();
                db_obj.updated_at = Some(Utc::now().naive_utc());
                db_obj.updated_by = Some(ctx.principal_id.clone());
                if let Err(err) = self._update(&db_obj) {
                    return Err(RbacError::Persistence(err.to_string()));
                }
                self.audit(ctx, format!("Updated claim {:?}", db_obj), "CREATE");
                Ok(Claim::from(&db_obj))
            }
            None => Err(RbacError::NotFound(format!("Claim not found {:?}", claim)))
        }
    }

    /// Retrieves claim by id from the database
    pub fn get(&self, _ctx: &SecurityContext, realm_id: &str, resource_id: &str, claim_id: &str) -> Option<Claim> {
        match self._get(realm_id, resource_id, claim_id) {
            Some(claim) => Some(Claim::from(&claim)),
            _ => None,
        }
    }

    /// Returns all claims for given security realm and resource
    pub fn get_by_realm_resource(&self, _ctx: &SecurityContext, realm_id: &str, resource_id: &str) -> Vec<Claim> {
        self._get_by_realm_resource(realm_id, resource_id).iter().map(|c| Claim::from(&c)).collect::<Vec<Claim>>()
    }

    /// Returns all claims for given security realm
    pub fn get_claims_by_realm(&self, ctx: &SecurityContext, realm_id: &str) -> HashMap<String, Claim> {
        let mut claims = HashMap::new();
        for pclaim in &self._get_by_realm(realm_id) {
            let claim = Claim::from(pclaim); 
            claims.insert(claim.id.clone(), claim);
        }
        claims
    }

    /// Returns all claims for given security realm
    pub fn get_by_realm(&self, _ctx: &SecurityContext, realm_id: &str) -> Vec<Claim> {
        self._get_by_realm(realm_id).iter().map(|c| Claim::from(&c)).collect::<Vec<Claim>>()
    }

    /// Retrieves claims by claim-ids -- used internally
    pub fn _get_by_claim_ids(&self, claim_ids: Vec<String>) -> Vec<PClaim> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_claims::table
                .filter(rbac_claims::id.eq_any(claim_ids))
                .load::<PClaim>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }


    /// Deletes claims by id from the database
    pub fn delete(&self, ctx: &SecurityContext, realm_id: &str, resource_id: &str, id: &str) -> Result<usize, RbacError> {
        match self._delete(realm_id, resource_id, id) {
            Ok(n) => {
                self.audit(ctx, format!("Deleted claim {:?}", id), "DELETE");
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

    /// Stores Claim instance in the database
    fn _create(&self, claim: &PClaim) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_claims::table).values(claim).execute(&*connection)
    }

    /// Updates Claim instance in the database
    fn _update(&self, claim: &PClaim) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::update(rbac_claims::table.find(claim.id.clone())).set(claim).
            execute(&*connection)
    }

    /// Removes instance of claim in the database
    fn _delete(&self, realm_id: &str, resource_id: &str, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_claims::table
                .filter(rbac_claims::realm_id.eq(realm_id.to_string()))
                .filter(rbac_claims::resource_id.eq(resource_id.to_string()))
                .filter(rbac_claims::id.eq(id.to_string())))
                .execute(&*connection)
    }

    /// Removes instance of claim in the database
    fn __delete(&self, id: &str) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_claims::table.find(id.to_string())).execute(&*connection)
    }

    /// Retrieves instance of claim in the database
    fn __get(&self, id: &str) -> Result<PClaim, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_claims::table.find(id.to_string()).get_result::<PClaim>(&*connection)
        //let children = PClaimInstance::belonging_to(&claim).load::<PClaimInstance>(&*connection)?;
    }

    /// Retrieves instance of claim in the database
    fn _get(&self, realm_id: &str, resource_id: &str, claim_id: &str) -> Option<PClaim> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_claims::table
                .filter(rbac_claims::realm_id.eq(realm_id.to_string()))
                .filter(rbac_claims::resource_id.eq(resource_id.to_string()))
                .filter(rbac_claims::id.eq(claim_id.to_string()))
                .load::<PClaim>(&*connection) {
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

    /// Returns all claims for given security realm and resource
    fn _get_by_realm_resource(&self, realm_id: &str, resource_id: &str) -> Vec<PClaim> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_claims::table
                .filter(rbac_claims::realm_id.eq(realm_id.to_string()))
                .filter(rbac_claims::resource_id.eq(resource_id.to_string()))
                .load::<PClaim>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Returns all claims for given security realm
    fn _get_by_realm(&self, realm_id: &str) -> Vec<PClaim> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_claims::table
                .filter(rbac_claims::realm_id.eq(realm_id.to_string()))
                .load::<PClaim>(&*connection) {
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
        let _ = diesel::delete(rbac_claims::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::domain::models::Claim;
    use plexrbac::common::SecurityContext;

    #[test]
    fn test_save() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_repository();
        repo.clear();

        let claim = repo.create(&ctx, &Claim::new("", "99", "11", "READ", "Allow", None)).unwrap();
        let claim_str = format!("{:?}", claim);

        let loaded = repo.get(&ctx, "99", "11", claim.id.as_str()).unwrap();
        assert_eq!(claim_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_repository();
        repo.clear();

        let claim = repo.create(&ctx, &Claim::new("", "99", "11", "READ", "Allow", None)).unwrap();

        let mut loaded = repo.get(&ctx, "99", "11", claim.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        repo.update(&ctx, &loaded).unwrap();
        let loaded = repo.get(&ctx, "99", "11", loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_repository();
        repo.clear();

        let claim = repo.create(&ctx, &Claim::new("", "99", "11", "READ", "Allow", None)).unwrap();
        repo.delete(&ctx, "99", "11", claim.id.as_str()).unwrap();
        let loaded = repo.get(&ctx, "99", "11", claim.id.as_str());
        assert!(loaded.is_none());
    }

    #[test]
    fn test_get_by_claim_ids() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_repository();
        repo.clear();

        let claim = repo.create(&ctx, &Claim::new("", "99", "11", "READ", "Allow", None)).unwrap();
        assert_eq!(1, repo._get_by_claim_ids(vec![claim.id.clone()]).len());
    }

    #[test]
    fn test_get_by_realm() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Claim::new("", "99", "11", "READ", "Allow", None)).unwrap();
        let _ = repo.create(&ctx, &Claim::new("", "99", "12", "READ", "Allow", None)).unwrap();
        let results = repo.get_by_realm(&ctx, "99");
        assert_eq!(2, results.len());
    }

    #[test]
    fn test_get_by_realm_resource() {
        let ctx = SecurityContext::new("myorg", "myid");
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_repository();
        repo.clear();

        let _ = repo.create(&ctx, &Claim::new("", "99", "11", "READ", "Allow", None)).unwrap();
        let _ = repo.create(&ctx, &Claim::new("", "99", "11", "WRITE", "Allow", None)).unwrap();
        let results = repo.get_by_realm_resource(&ctx, "99", "11");
        assert_eq!(2, results.len());
    }
}
