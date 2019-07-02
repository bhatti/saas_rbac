//#![crate_name = "doc"]

use diesel::prelude::*;
use super::schema::{rbac_claims};
use super::models::{PClaim};

//////////////////////////////////////////////////////////////////////////////////////////////
/// ClaimRepository defines methods for accessing and persisting Claim
///
pub struct ClaimRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ClaimRepository<'a> {
    /// Returns all claims for given security realm
    pub fn get_by_realm(&self, realm_id: &str) -> Vec<PClaim> {
        let connection = self.factory.new_connection();
        match rbac_claims::table
            .filter(rbac_claims::realm_id.eq(realm_id.to_string()))
            .load::<PClaim>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Retrieves claims by claim-ids
    pub fn get_claim_by_claim_ids(&self, claim_ids: Vec<String>) -> Vec<PClaim> {
        let connection = self.factory.new_connection();
        match rbac_claims::table
            .filter(rbac_claims::id.eq_any(claim_ids))
            .load::<PClaim>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }


    /// Stores Claim instance in the database
    pub fn create(&self, claim: &PClaim) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_claims::table).values(claim).execute(&*connection) {
            Ok(_) => {
                None
            },
            Err(err) => Some(err),
        }
    }

    /// Updates Claim instance in the database
    pub fn update(&self, claim: &PClaim) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::update(rbac_claims::table.find(claim.id.clone())).set(claim).
            execute(&*connection) {
            Ok(_) => {
                None
            },
            Err(err) => Some(err),
        }
    }

    /// Removes all instances in the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_claims::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Removes instance of claim in the database
    pub fn delete(&self, id: &str) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_claims::table.find(id.to_string())).execute(&*connection) {
            Ok(n) => {
               n > 0 
            },
            _ => false,
        }

    }

    /// Retrieves instance of claim in the database
    pub fn get(&self, id: &str) -> Option<PClaim> {
        let connection = self.factory.new_connection();
        match rbac_claims::table.find(id.to_string()).get_result::<PClaim>(&*connection) {
            Ok(claim) => {
                //let children = PClaimInstance::belonging_to(&claim).load::<PClaimInstance>(&*connection)?;
                Some(claim)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate uuid as uuu;
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::persistence::models::{PClaim};
    use self::uuu::Uuid;

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_repository();
        repo.clear();

        let claim = PClaim::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "11", "view", "allow", None);
        let claim_str = format!("{:?}", claim);
        repo.create(&claim);

        let loaded = repo.get(claim.id.as_str()).unwrap();
        assert_eq!(claim_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_update() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_repository();
        repo.clear();

        let claim = PClaim::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "11", "view", "allow", None);
        repo.create(&claim);

        let mut loaded = repo.get(claim.id.as_str()).unwrap();
        loaded.description = Some("blah".to_string());
        repo.update(&loaded);
        let loaded = repo.get(loaded.id.as_str()).unwrap();
        assert_eq!(Some("blah".to_string()), loaded.description);
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_repository();
        repo.clear();

        let claim = PClaim::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "11", "view", "allow", None);
        repo.create(&claim);
        assert_eq!(1, repo.get_claim_by_claim_ids(vec![claim.id.clone()]).len());
        repo.delete(claim.id.as_str());
        let loaded = repo.get(claim.id.as_str());
        assert_eq!(None, loaded);
    }

    #[test]
    fn test_get_by_realm() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_repository();
        repo.clear();

        repo.create(&PClaim::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "1", "view", "allow", None));
        repo.create(&PClaim::new(Uuid::new_v4().to_hyphenated().to_string().as_str(), "99", "1", "edit", "allow", None));

        let results = repo.get_by_realm("99");
        assert_eq!(2, results.len());
    }
}
