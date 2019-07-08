//#![crate_name = "doc"]

use diesel::prelude::*;
use plexrbac::common::Constants;
use super::schema::rbac_claim_claimables;
use super::models::PClaimClaimable;
use plexrbac::common::SecurityContext;
use plexrbac::common::RbacError;
use chrono::{NaiveDate, NaiveDateTime, Utc};

//////////////////////////////////////////////////////////////////////////////////////////////
/// ClaimClaimableRepository defines association between claim an claimable -- principal, group,
/// license-policy
///
pub struct ClaimClaimableRepository<'a> {
    pub data_source: &'a super::data_source::DataSource,
    pub audit_record_repository: super::audit_record_repository::AuditRecordRepository<'a>,
}

impl<'a> ClaimClaimableRepository<'a> {
    /// Adds principal to claim
    /// TODO verify against license policy -- for all below
    /// TODO verify time bounds against license policy -- for all below
    pub fn add_principal_to_claim(&self, ctx: &SecurityContext, principal_id: &str, claim_id: &str, scope: &str, claim_constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Result<usize, RbacError> {
        let cc = PClaimClaimable::new(claim_id, principal_id, Constants::Principal.to_string().as_str(), scope, claim_constraints, effective_at, expired_at);
        match self.create(&cc) {
            Ok(n) => {
                self.audit(ctx, format!("Adding principal to claim {:?}", cc), "CREATE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Removes principal from claim
    pub fn delete_principal_from_claim(&self, ctx: &SecurityContext, principal_id: &str, claim_id: &str) -> Result<usize, RbacError> {
        let cc = PClaimClaimable::new(claim_id, principal_id, Constants::Principal.to_string().as_str(), "", "", Utc::now().naive_utc(), Utc::now().naive_utc());
        match self.delete(&cc) {
            Ok(n) => {
                self.audit(ctx, format!("Removing principal from claim {:?}", cc), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Adds role to claim
    pub fn add_role_to_claim(&self, ctx: &SecurityContext, role_id: &str, claim_id: &str, scope: &str, claim_constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Result<usize, RbacError> {
        let cc = PClaimClaimable::new(claim_id, role_id, Constants::Role.to_string().as_str(), scope, claim_constraints, effective_at, expired_at);
        match self.create(&cc) {
            Ok(n) => {
                self.audit(ctx, format!("Adding role to claim {:?}", cc), "CREATE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Removes role from claim
    pub fn delete_role_from_claim(&self, ctx: &SecurityContext, role_id: &str, claim_id: &str) -> Result<usize, RbacError> {
        let cc = PClaimClaimable::new(claim_id, role_id, Constants::Role.to_string().as_str(), "", "", Utc::now().naive_utc(), Utc::now().naive_utc());
        match self.delete(&cc) {
            Ok(n) => {
                self.audit(ctx, format!("Removing role from claim {:?}", cc), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }


    /// Adds license-policy to claim
    pub fn add_license_policy_to_claim(&self, ctx: &SecurityContext, license_policy_id: &str, claim_id: &str, scope: &str, constraints: &str, effective_at: NaiveDateTime, expired_at: NaiveDateTime) -> Result<usize, RbacError> {
        let cc = PClaimClaimable::new(claim_id, license_policy_id, Constants::LicensePolicy.to_string().as_str(), scope, constraints, effective_at, expired_at);
        match self.create(&cc) {
            Ok(n) => {
                self.audit(ctx, format!("Adding claim to license-policy {:?}", cc), "CREATE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Removes license-policy from claim
    pub fn delete_license_policy_from_claim(&self, ctx: &SecurityContext, license_policy_id: &str, claim_id: &str) -> Result<usize, RbacError> {
        let cc = PClaimClaimable::new(claim_id, license_policy_id, Constants::LicensePolicy.to_string().as_str(), "", "", Utc::now().naive_utc(), Utc::now().naive_utc());
        match self.delete(&cc) {
            Ok(n) => {
                self.audit(ctx, format!("Removing claim from license-policy {:?}", cc), "DELETE");
                Ok(n)
            },
            Err(err) => Err(RbacError::Persistence(err.to_string())),
        }
    }

    /// Checks if association exists
    pub fn exists(&self, association: &PClaimClaimable) -> bool {
        if let Ok(connection) = self.data_source.new_connection() {
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
        } else {
            false
        }
    }

    /// Retrieves associations by claim-id
    pub fn get_by_claims(&self, claim_ids: Vec<String>) -> Vec<PClaimClaimable> {
        if let Ok(connection) = self.data_source.new_connection() {
            let now = Utc::now().naive_utc();
            match rbac_claim_claimables::table
                .filter(rbac_claim_claimables::claim_id.eq_any(claim_ids))
                .filter(rbac_claim_claimables::effective_at.le(now))
                .filter(rbac_claim_claimables::expired_at.ge(now))
                .load::<PClaimClaimable>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieves claims by principal-id
    pub fn get_by_principal(&self, principal_id: String) -> Vec<PClaimClaimable> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_claim_claimables::table
                .filter(rbac_claim_claimables::claimable_id.eq(principal_id))
                .filter(rbac_claim_claimables::claimable_type.eq(Constants::Principal.to_string()))
                .filter(rbac_claim_claimables::effective_at.le(now))
                .filter(rbac_claim_claimables::expired_at.ge(now))
                .load::<PClaimClaimable>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieves claims by role-ids
    pub fn get_by_roles(&self, role_ids: Vec<String>) -> Vec<PClaimClaimable> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_claim_claimables::table
                .filter(rbac_claim_claimables::claimable_id.eq_any(role_ids))
                .filter(rbac_claim_claimables::claimable_type.eq(Constants::Role.to_string()))
                .filter(rbac_claim_claimables::effective_at.le(now))
                .filter(rbac_claim_claimables::expired_at.ge(now))
                .load::<PClaimClaimable>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieves claim-ids by license-policy
    pub fn get_by_policy(&self, license_policy_id: &str) -> Vec<PClaimClaimable> {
        let now = Utc::now().naive_utc();
        if let Ok(connection) = self.data_source.new_connection() {
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
        } else {
            vec![]
        }
    }


    /// Retrieves associations by claimable-id and type
    pub fn get_by_claimables(&self, claimable_ids: Vec<String>, claimable_type: String) -> Vec<PClaimClaimable> {
        if let Ok(connection) = self.data_source.new_connection() {
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
        } else {
            vec![]
        }
    }

    /// Creates new assocoation between claim and claimable
    pub fn create(&self, association: &PClaimClaimable) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_claim_claimables::table).values(association).execute(&*connection)
    }

    /// Deletes assocoation between claim and claimable
    pub fn delete(&self, association: &PClaimClaimable) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::delete(rbac_claim_claimables::table)
            .filter(rbac_claim_claimables::claim_id.eq(association.claim_id.clone()))
            .filter(rbac_claim_claimables::claimable_id.eq(association.claimable_id.clone()))
            .filter(rbac_claim_claimables::claimable_type.eq(association.claimable_type.clone()))
            .execute(&*connection)
    }

    ///////////////////////////////////// PRIVATE METHODS ////////////////////////////////
    fn audit(&self, ctx: &SecurityContext, message: String, action: &str) {
        let _ = self.audit_record_repository.create_with(message.as_str(), action, format!("{:?}", ctx).as_str(), ctx.principal_id.clone());
        info!("{}", message);
    }

    /// Removes all assocoations between claim and claimable - for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        let _ = diesel::delete(rbac_claim_claimables::table).execute(&*connection);
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::models::PClaimClaimable;
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use chrono::{NaiveDate,Utc};

    #[test]
    fn test_get_save() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_claimable_repository();
        repo.clear();

        let association = PClaimClaimable::new("1", "12", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&association).unwrap();
        assert_eq!(true, repo.exists(&association));
    }

    #[test]
    fn test_delete() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_claimable_repository();
        repo.clear();

        let association = PClaimClaimable::new("1", "12", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0));
        repo.create(&association).unwrap();
        repo.delete(&association).unwrap();
        assert_eq!(false, repo.exists(&association));
    }

    #[test]
    fn test_get_all() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_claim_claimable_repository();
        repo.clear();

        repo.create(&PClaimClaimable::new("1", "12", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.create(&PClaimClaimable::new("1", "13", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.create(&PClaimClaimable::new("2", "14", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();
        repo.create(&PClaimClaimable::new("2", "15", "Principal", "scope", "", Utc::now().naive_utc(), NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))).unwrap();

        let results = repo.get_by_claims(vec!["1".to_string()]);
        assert_eq!(2, results.len());
        let results = repo.get_by_claimables(vec!["12".to_string()], "Principal".to_string());
        assert_eq!(1, results.len());
    }
}
