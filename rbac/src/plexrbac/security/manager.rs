//#![crate_name = "doc"]
    
use evalexpr::EvalexprError;
use plexrbac::security::request::PermissionRequest;
use plexrbac::persistence::manager::PersistenceManager;
use plexrbac::domain::models::PermissionResult;
use plexrbac::utils::text;
use plexrbac::utils::evaluator::*;

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

    pub fn check(&self, request: &PermissionRequest) -> Result<PermissionResult, evalexpr::EvalexprError> {
        if let Some(principal) = self.persistence_manager.get_principal(&request.context, request.context.realm_id.as_str(), request.context.principal_id.as_str()) {
            let claim_resources = self.persistence_manager.get_resources_by_claims(&request.context, request.context.realm_id.as_str(), &principal.claims, request.resource_name.clone(), request.resource_scope.clone());
            let mut claim_resources_str  = String::from("");
            for cr in claim_resources {
                claim_resources_str.push_str(format!("\t{}     {}     {}\n", cr.claim.action, cr.constraints, cr.resource.resource_name).as_str());
                if  text::regex_find(cr.claim.action.as_str(), request.action.as_str()) {
                    if cr.constraints.len() > 0 {
                        match evaluate(cr.constraints.as_str(), &request.context.properties) {
                            Ok(ok) => {
                                if ok {
                                    return Ok(PermissionResult::from(cr.claim.effect));
                                } else {
                                    //println!(">>>>>>>>> EVALUATED FALSE for {} -- {:?}\n{:?}", cr.constraints.as_str(), cr, request);
                                }
                            },
                            Err(err) => return Err(err),
                        }
                    } else {
                        return Ok(PermissionResult::from(cr.claim.effect));
                    }
                }
            }
            Err(EvalexprError::CustomMessage(format!("No matching claim found for {:?} -- available claims\n{}", request, claim_resources_str)))
        } else {
            Err(EvalexprError::CustomMessage(format!("Could not find principal data for {:?}", request)))
        }
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::domain::models::*;
    use plexrbac::security::context::SecurityContext;
    use plexrbac::security::manager::SecurityManager;
    use plexrbac::security::request::PermissionRequest;

    #[test]
    fn test_evaluate() {
        // Initialize context and repository
        let ctx = SecurityContext::new("0".into(), "0".into());
        let factory = RepositoryFactory::new();
        let mgr = factory.new_persistence_manager();
        mgr.clear();
        // Bootstrapping dependent data

        // Creating security realm
        let realm = mgr.new_realm_with(&ctx, "banking").unwrap();

        // Creating organization
        let org = mgr.new_org_with(&ctx, "bank-of-flakes").unwrap();

        // Creating Users
        let tom = mgr.new_principal_with(&ctx, &org, "tom").unwrap();

        // Creating Roles
        let employee = mgr.new_role_with(&ctx, &realm, &org, "Employee").unwrap();
        let teller = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Teller").unwrap();

        // Creating Resources
        let deposit_account = mgr.new_resource_with(&ctx, &realm, "DepositAccount").unwrap();

        // Creating claims for resources
        let cd_deposit = mgr.new_claim_with(&ctx, &realm, &deposit_account, "(CREATE|DELETE)").unwrap();
        let ru_deposit = mgr.new_claim_with(&ctx, &realm, &deposit_account, "(READ|UPDATE)").unwrap();

        // Mapping Principals and Claims to Roles
        mgr.map_principal_to_role(&ctx, &tom, &teller);

        // Map claims to roles as follows:
        mgr.map_role_to_claim(&ctx, &teller, &ru_deposit, "U.S.", r#"employeeRegion == "Midwest""#);

        let security_mgr = SecurityManager::new(mgr);
        let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), "READ", "DepositAccount", "U.S.");
        req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
        assert_eq!(PermissionResult::Allow, security_mgr.check(&req).unwrap());
    }
}
