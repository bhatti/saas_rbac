//#![crate_name = "doc"]
    
use plexrbac::security::request::PermissionRequest;
use plexrbac::security::response::PermissionResponse;
use plexrbac::persistence::manager::PersistenceManager;
use plexrbac::utils::text;
use plexrbac::utils::evaluator::*;
use plexrbac::common::RbacError;
use log::{info, warn};

////////////////////////////////////////////////////////////////////////////////
/// SecurityManager checks access
///
pub struct SecurityManager<'a> {
    pub persistence_manager: PersistenceManager<'a>,
}

impl <'a> SecurityManager<'a> {
    pub fn new(persistence_manager: PersistenceManager) -> SecurityManager {
        SecurityManager {
            persistence_manager: persistence_manager,
        }
    }

    pub fn check(&self, request: &PermissionRequest) -> Result<PermissionResponse, RbacError> {
        if let Some(principal) = self.persistence_manager.get_principal(&request.context, request.context.realm_id.as_str(), request.context.principal_id.as_str()) {
            let claim_resources = self.persistence_manager.get_resources_by_claims(&request.context, request.context.realm_id.as_str(), &principal, request.resource_name.clone(), request.resource_scope.clone());
            let mut claim_resources_str  = String::from("");
            for cr in claim_resources {
                claim_resources_str.push_str(format!("\t{}     {}     {}\n", cr.claim.action, cr.constraints, cr.resource.resource_name).as_str());
                if  text::regex_find(cr.claim.action.as_str(), request.action.as_str()) {
                    if cr.constraints.len() > 0 {
                        match evaluate(cr.constraints.as_str(), &request.context.properties) {
                            Ok(ok) => {
                                if ok {
                                    info!("GRANTED PERMISSION {:?} -- {:?}", request, cr.claim);
                                    return Ok(PermissionResponse::from(cr.claim.effect()));
                                } else {
                                    //info!(">>>>>>>>> EVALUATED FALSE for {} -- {:?}\n{:?}", cr.constraints.as_str(), cr, request);
                                }
                            },
                            Err(err) => return Err(RbacError::Evaluation(err.to_string())),
                        }
                    } else {
                        return Ok(PermissionResponse::from(cr.claim.effect()));
                    }
                }
            }

            warn!("DENIED PERMISSION {:?} because no matching claim found -- available claims: {}!!!", request, claim_resources_str);
            Err(RbacError::Evaluation(format!("No matching claim found for {:?} -- available claims: {}!!!", request, claim_resources_str)))
        } else {
            Err(RbacError::Evaluation(format!("Could not find principal data for {:?}", request)))
        }
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::common::SecurityContext;
    use plexrbac::security::manager::SecurityManager;
    use plexrbac::security::request::PermissionRequest;
    use plexrbac::security::response::PermissionResponse;
    use plexrbac::common::*;

    #[test]
    fn test_evaluate() {
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let pm = locator.new_persistence_manager();
        pm.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = pm.new_realm_with(&ctx, "banking").unwrap();

        // Creating organization
        let org = pm.new_org_with(&ctx, "bank-of-flakes").unwrap();

        // Creating Users
        let tom = pm.new_principal_with(&ctx, &org, "tom").unwrap();

        // Creating Roles
        let employee = pm.new_role_with(&ctx, &realm, &org, "Employee").unwrap();
        let teller = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Teller").unwrap();

        // Creating Resources
        let deposit_account = pm.new_resource_with(&ctx, &realm, "DepositAccount").unwrap();

        // Creating claims for resources
        let _cd_deposit = pm.new_claim_with(&ctx, &realm, &deposit_account, "(CREATE|DELETE)").unwrap();
        let ru_deposit = pm.new_claim_with(&ctx, &realm, &deposit_account, "(READ|UPDATE)").unwrap();

        // Mapping Principals and Claims to Roles
        pm.map_principal_to_role(&ctx, &tom, &teller).unwrap();

        // Map claims to roles as follows:
        pm.map_role_to_claim(&ctx, &teller, &ru_deposit, "U.S.", r#"employeeRegion == "Midwest""#).unwrap();

        let sm = SecurityManager::new(pm);
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert_eq!(PermissionResponse::Allow, sm.check(&req).unwrap());
    }
}
