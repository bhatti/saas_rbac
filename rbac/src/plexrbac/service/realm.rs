//#![crate_name = "doc"]
    
extern crate chrono;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate regex;
extern crate evalexpr;

use rocket::{State};
use rocket_contrib::json::{Json};
use rocket::http::Status;
use rocket::response::status::Custom;

use diesel::prelude::*;
use plexrbac::domain::models::{SecurityRealm, Resource, ResourceInstance, ResourceQuota, Claim};
use plexrbac::persistence::locator::RepositoryLocator;
use plexrbac::persistence::data_source::PooledDataSource;
use plexrbac::common::{SecurityContext};
use plexrbac::service::common::{AssociationForm};
use r2d2::{Pool};
use diesel::r2d2::ConnectionManager;

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// REST APIs for managing Security Realms
///
//////////////////////////////////////////////////////////////////////////////////////////////


#[get("/")]
pub fn all_realms(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>) -> Json<Vec<SecurityRealm>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_realm_repository(&ds);
    Json(repo.all(&ctx))
}

#[post("/", format = "json", data = "<realm>")]
pub fn create_realm(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm: Json<SecurityRealm>) -> Result<Json<SecurityRealm>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_realm_repository(&ds);
    match repo.create(&ctx, &realm) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<id>", format = "json", data = "<realm>")]
pub fn update_realm(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, id: String, mut realm: Json<SecurityRealm>) -> Result<Json<SecurityRealm>, Custom<String>> {
    realm.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_realm_repository(&ds);
    match repo.update(&ctx, &realm) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<id>", format = "json")]
pub fn get_realm(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, id: String) -> Result<Json<SecurityRealm>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_realm_repository(&ds);
    match repo.get(&ctx, &id.as_str()) {
        Some(realm) => Ok(Json(realm)),
        None => Err(Custom(Status::NotFound, format!("realm with id {} not found", id))),
    }
}

#[delete("/<id>", format = "json")]
pub fn delete_realm(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_realm_repository(&ds);
    match repo.delete(&ctx, id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

///////////////////////////////// RESOURCES APIS //////////////////////////////

#[get("/<realm_id>/resources")]
pub fn get_resources_by_realm(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String) -> Json<Vec<Resource>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_resource_repository(&ds);
    let mut resources = vec![];
    for (_, r) in repo.get_by_realm(&ctx, realm_id.as_str()) {
        resources.push(r);
    }
    Json(resources)
}

#[post("/<realm_id>/resources", format = "json", data = "<resource>")]
pub fn create_resource(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, mut resource: Json<Resource>) -> Result<Json<Resource>, Custom<String>> {
    resource.realm_id = realm_id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_resource_repository(&ds);
    match repo.create(&ctx, &resource) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<id>", format = "json", data = "<resource>")]
pub fn update_resource(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, id: String, mut resource: Json<Resource>) -> Result<Json<Resource>, Custom<String>> {
    resource.realm_id = realm_id;
    resource.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_resource_repository(&ds);
    match repo.update(&ctx, &resource) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<realm_id>/resources/<id>", format = "json")]
pub fn get_resource(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, id: String) -> Result<Json<Resource>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_resource_repository(&ds);
    match repo.get(&ctx, &realm_id.as_str(), &id.as_str()) {
        Some(resource) => Ok(Json(resource)),
        None => Err(Custom(Status::NotFound, format!("resource with id {} not found", id))),
    }
}

#[delete("/<realm_id>/resources/<id>", format = "json")]
pub fn delete_resource(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_resource_repository(&ds);
    match repo.delete(&ctx, realm_id.as_str(), id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

///////////////////////////////// RESOURCE INSTANCE APIS //////////////////////////////
#[get("/<realm_id>/resources/<resource_id>/instances")]
pub fn get_instances(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String) -> Json<Vec<ResourceInstance>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Json(vec![]);
    }
    let repo = RepositoryLocator::build_resource_instance_repository(&ds);
    Json(repo.get_by_resource(&ctx, resource_id.as_str()))
}

#[post("/<realm_id>/resources/<resource_id>/instances", format = "json", data = "<instance>")]
pub fn create_instance(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, mut instance: Json<ResourceInstance>) -> Result<Json<ResourceInstance>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    //
    instance.resource_id = resource_id;
    let repo = RepositoryLocator::build_persistence_manager(&ds);
    match repo.new_resource_instance(&ctx, ctx.principal_id.as_str(), &mut instance) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<resource_id>/instances/<id>", format = "json", data = "<instance>")]
pub fn update_instance(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String, mut instance: Json<ResourceInstance>) -> Result<Json<ResourceInstance>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    instance.id = id;
    instance.resource_id = resource_id;
    let repo = RepositoryLocator::build_resource_instance_repository(&ds);
    match repo.update(&ctx, &instance) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<realm_id>/resources/<resource_id>/instances/<id>", format = "json")]
pub fn get_instance(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String) -> Result<Json<ResourceInstance>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_resource_instance_repository(&ds);
    match repo.get(&ctx, &id.as_str()) {
        Some(instance) => Ok(Json(instance)),
        None => Err(Custom(Status::NotFound, format!("instance with id {} not found", id))),
    }
}

#[delete("/<realm_id>/resources/<resource_id>/instances/<id>", format = "json")]
pub fn delete_instance(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_resource_instance_repository(&ds);
    match repo.delete(&ctx, id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

///////////////////////////////// RESOURCE QUOTA APIS //////////////////////////////

#[get("/<realm_id>/resources/<resource_id>/quota")]
pub fn get_quotas(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String) -> Json<Vec<ResourceQuota>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Json(vec![]);
    }
    let repo = RepositoryLocator::build_resource_quota_repository(&ds);
    Json(repo.get_by_resource(&ctx, resource_id.as_str()))
}

#[post("/<realm_id>/resources/<resource_id>/quota", format = "json", data = "<quota>")]
pub fn create_quota(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, mut quota: Json<ResourceQuota>) -> Result<Json<ResourceQuota>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    quota.resource_id = resource_id;
    let repo = RepositoryLocator::build_resource_quota_repository(&ds);
    match repo.create(&ctx, &quota) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<resource_id>/quota/<id>", format = "json", data = "<quota>")]
pub fn update_quota(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String, mut quota: Json<ResourceQuota>) -> Result<Json<ResourceQuota>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    quota.id = id;
    quota.resource_id = resource_id;
    let repo = RepositoryLocator::build_resource_quota_repository(&ds);
    match repo.update(&ctx, &quota) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<realm_id>/resources/<resource_id>/quota/<id>", format = "json")]
pub fn get_quota(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String) -> Result<Json<ResourceQuota>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_resource_quota_repository(&ds);
    match repo.get(&ctx, id.as_str()) {
        Some(quota) => Ok(Json(quota)),
        None => Err(Custom(Status::NotFound, format!("quota with id {} not found", id))),
    }
}

#[delete("/<realm_id>/resources/<resource_id>/quota/<id>", format = "json")]
pub fn delete_quota(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_resource_quota_repository(&ds);
    match repo.delete(&ctx, id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

///////////////////////////////// RESOURCE CLAIM APIS //////////////////////////////

#[get("/<realm_id>/claims")]
pub fn get_realm_claims(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String) -> Json<Vec<Claim>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_claim_repository(&ds);
    Json(repo.get_by_realm(&ctx, realm_id.as_str()))
}

#[get("/<realm_id>/resources/<resource_id>/claims")]
pub fn get_claims(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String) -> Json<Vec<Claim>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Json(vec![]);
    }
    let repo = RepositoryLocator::build_claim_repository(&ds);
    Json(repo.get_by_realm_resource(&ctx, realm_id.as_str(), resource_id.as_str()))
}

#[post("/<realm_id>/resources/<resource_id>/claims", format = "json", data = "<claim>")]
pub fn create_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, mut claim: Json<Claim>) -> Result<Json<Claim>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    claim.realm_id = realm_id;
    claim.resource_id = resource_id;
    let repo = RepositoryLocator::build_claim_repository(&ds);
    match repo.create(&ctx, &claim) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<resource_id>/claims/<id>", format = "json", data = "<claim>")]
pub fn update_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String, mut claim: Json<Claim>) -> Result<Json<Claim>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    claim.id = id;
    claim.resource_id = resource_id;
    claim.realm_id = realm_id;
    let repo = RepositoryLocator::build_claim_repository(&ds);
    match repo.update(&ctx, &claim) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<realm_id>/resources/<resource_id>/claims/<id>", format = "json")]
pub fn get_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String) -> Result<Json<Claim>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_repository(&ds);
    match repo.get(&ctx, realm_id.as_str(), resource_id.as_str(), id.as_str()) {
        Some(claim) => Ok(Json(claim)),
        None => Err(Custom(Status::NotFound, format!("claim with id {} not found", id))),
    }
}

#[delete("/<realm_id>/resources/<resource_id>/claims/<id>", format = "json")]
pub fn delete_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_repository(&ds);
    match repo.delete(&ctx, realm_id.as_str(), resource_id.as_str(), id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<resource_id>/claims/<claim_id>/principals/<principal_id>", format = "json")]
pub fn add_principal_to_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, claim_id: String, principal_id: String, cc: AssociationForm) -> Result<Json<usize>, Custom<String>> { // Form<>
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_claimable_repository(&ds);
    //let cc = form.into_inner();
    match repo.add_principal_to_claim(&ctx, principal_id.as_str(), claim_id.as_str(), cc.scope.as_str(), cc.constraints.as_str(), cc.effective_at(), cc.expired_at()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[delete("/<realm_id>/resources/<resource_id>/claims/<claim_id>/principals/<principal_id>", format = "json")]
pub fn delete_principal_from_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, claim_id: String, principal_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_claimable_repository(&ds);
    match repo.delete_principal_from_claim(&ctx, principal_id.as_str(), claim_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<resource_id>/claims/<claim_id>/roles/<role_id>", format = "json")]
pub fn add_role_to_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, claim_id: String, role_id: String, cc: AssociationForm) -> Result<Json<usize>, Custom<String>> { // Form<>
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_claimable_repository(&ds);
    //let cc = form.into_inner();
    match repo.add_role_to_claim(&ctx, role_id.as_str(), claim_id.as_str(), cc.scope.as_str(), cc.constraints.as_str(), cc.effective_at(), cc.expired_at()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[delete("/<realm_id>/resources/<resource_id>/claims/<claim_id>/roles/<role_id>", format = "json")]
pub fn delete_role_from_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, claim_id: String, role_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_claimable_repository(&ds);
    match repo.delete_role_from_claim(&ctx, role_id.as_str(), claim_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<realm_id>/resources/<resource_id>/claims/<claim_id>/licenses/<license_policy_id>", format = "json")]
pub fn add_license_to_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, claim_id: String, license_policy_id: String, cc: AssociationForm) -> Result<Json<usize>, Custom<String>> { // Form<>
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_claimable_repository(&ds);
    //let cc = form.into_inner();
    match repo.add_license_policy_to_claim(&ctx, license_policy_id.as_str(), claim_id.as_str(), cc.scope.as_str(), cc.constraints.as_str(), cc.effective_at(), cc.expired_at()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[delete("/<realm_id>/resources/<resource_id>/claims/<claim_id>/licenses/<license_policy_id>", format = "json")]
pub fn delete_license_from_claim(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, realm_id: String, resource_id: String, claim_id: String, license_policy_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // resource-id must exist within the realm
    if RepositoryLocator::build_resource_repository(&ds).get(&ctx, &realm_id.as_str(), &resource_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("resource with id {} not found within relam {}", resource_id, realm_id)));
    }
    let repo = RepositoryLocator::build_claim_claimable_repository(&ds);
    match repo.delete_license_policy_from_claim(&ctx, license_policy_id.as_str(), claim_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}
