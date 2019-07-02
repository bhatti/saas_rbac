//#![crate_name = "doc"]

use diesel::prelude::*;
use plexrbac::domain::models::Constants;
use super::schema::rbac_claim_claimables;
use super::models::PClaimClaimable;
use chrono::{Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// ClaimClaimableRepository defines association between claim an claimable -- principal, group,
/// license-policy
///
pub struct ClaimClaimableRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ClaimClaimableRepository<'a> {
    /// Checks if association exists
    pub fn exists(&self, association: &PClaimClaimable) -> bool {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_claim_claimables::table
            .filter(rbac_claim_claimables::claim_id.eq(association.claim_id.clone()))
            .filter(rbac_claim_claimables::claimable_id.eq(association.claimable_id.clone()))
            .filter(rbac_claim_claimables::claimable_type.eq(association.claimable_type.clone()))
            .filter(rbac_claim_claimables::effective_at.le(now))
            .filter(rbac_claim_claimables::expired_at.ge(now))
            .load::<PClaimClaimable>(&*connection) {
            Ok(v) => if v.first() != None {true} else {false},
            _ => false,
        }
    }

    /// Retrieves associations by claim-id
    pub fn get_by_claims(&self, claim_ids: Vec<String>) -> Vec<PClaimClaimable> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_claim_claimables::table
            .filter(rbac_claim_claimables::claim_id.eq_any(claim_ids))
            .filter(rbac_claim_claimables::effective_at.le(now))
            .filter(rbac_claim_claimables::expired_at.ge(now))
            .load::<PClaimClaimable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Retrieves claims by principal-id
    pub fn get_by_principal(&self, principal_id: String) -> Vec<PClaimClaimable> {
        let now = Utc::now().naive_utc();
        let connection = self.factory.new_connection();
        match rbac_claim_claimables::table
            .filter(rbac_claim_claimables::claimable_id.eq(principal_id))
            .filter(rbac_claim_claimables::claimable_type.eq(Constants::Principal.to_string()))
            .filter(rbac_claim_claimables::effective_at.le(now))
            .filter(rbac_claim_claimables::expired_at.ge(now))
            .load::<PClaimClaimable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Retrieves claims by role-ids
    pub fn get_by_roles(&self, role_ids: Vec<String>) -> Vec<PClaimClaimable> {
        let now = Utc::now().naive_utc();
        let connection = self.factory.new_connection();
        match rbac_claim_claimables::table
            .filter(rbac_claim_claimables::claimable_id.eq_any(role_ids))
            .filter(rbac_claim_claimables::claimable_type.eq(Constants::Role.to_string()))
            .filter(rbac_claim_claimables::effective_at.le(now))
            .filter(rbac_claim_claimables::expired_at.ge(now))
            .load::<PClaimClaimable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Retrieves claim-ids by license-policy
    pub fn get_by_policy(&self, license_policy_id: &str) -> Vec<PClaimClaimable> {
        let now = Utc::now().naive_utc();
        let connection = self.factory.new_connection();
        //if let Ok(claims_policies) = rbac_claims::table.inner_join(rbac_claim_claimables::table)
        //    .filter(rbac_license_policies::id.eq(license_policy_id))
        //    .load::<(PClaim, PClaimLicensePolicy)>(&*connection) {
        //}
        match rbac_claim_claimables::table
            .filter(rbac_claim_claimables::effective_at.le(now))
            .filter(rbac_claim_claimables::expired_at.ge(now))
            .filter(rbac_claim_claimables::claimable_id.eq(license_policy_id.to_string()))
            .filter(rbac_claim_claimables::claimable_type.eq(Constants::LicensePolicy.to_string()))
            .group_by(rbac_claim_claimables::claim_id)
            .load::<PClaimClaimable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }


    /// Retrieves associations by claimable-id and type
    pub fn get_by_claimables(&self, claimable_ids: Vec<String>, claimable_type: String) -> Vec<PClaimClaimable> {
        let connection = self.factory.new_connection();
        let now = Utc::now().naive_utc();
        match rbac_claim_claimables::table
            .filter(rbac_claim_claimables::claimable_id.eq_any(claimable_ids))
            .filter(rbac_claim_claimables::claimable_type.eq(claimable_type))
            .filter(rbac_claim_claimables::effective_at.le(now))
            .filter(rbac_claim_claimables::expired_at.ge(now))
            .load::<PClaimClaimable>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Creates new assocoation between claim and claimable
    pub fn create(&self, association: &PClaimClaimable) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_claim_claimables::table).values(association).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all assocoations between claim and claimable
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_claim_claimables::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Deletes assocoation between claim and claimable
    pub fn delete(&self, association: &PClaimClaimable) -> bool {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_claim_claimables::table)
            .filter(rbac_claim_claimables::claim_id.eq(association.claim_id.clone()))
            .filter(rbac_claim_claimables::claimable_id.eq(association.claimable_id.clone()))
            .filter(rbac_claim_claimables::claimable_type.eq(association.claimable_type.clone()))
            .execute(&*connection) {
            Ok(n) => n > 0,
            _ => false,
        }

    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::models::PClaimClaimable;
    use plexrbac::persistence::factory::RepositoryFactory;
    use chrono::{NaiveDate,Utc};

    #[test]
    fn test_get_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_claimable_repository();
        repo.clear();

        let association = PClaimClaimable::new("1", "12", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&association);
        assert_eq!(true, repo.exists(&association));
    }

    #[test]
    fn test_delete() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_claimable_repository();
        repo.clear();

        let association = PClaimClaimable::new("1", "12", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&association);
        repo.delete(&association);
        assert_eq!(false, repo.exists(&association));
    }

    #[test]
    fn test_get_all() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_claim_claimable_repository();
        repo.clear();

        repo.create(&PClaimClaimable::new("1", "12", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PClaimClaimable::new("1", "13", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PClaimClaimable::new("2", "14", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));
        repo.create(&PClaimClaimable::new("2", "15", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)));

        let results = repo.get_by_claims(vec!["1".to_string()]);
        assert_eq!(2, results.len());
        let results = repo.get_by_claimables(vec!["12".to_string()], "Principal".to_string());
        assert_eq!(1, results.len());
    }
}
