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
use plexrbac::domain::models::{Organization, Group, Role, Principal, LicensePolicy};
use plexrbac::persistence::locator::RepositoryLocator;
use plexrbac::persistence::data_source::PooledDataSource;
use plexrbac::common::{SecurityContext};
use plexrbac::service::common::{AssociationForm};
use r2d2::{Pool};
use diesel::r2d2::ConnectionManager;

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// REST APIs for managing organization
///
//////////////////////////////////////////////////////////////////////////////////////////////


///////////////////////////////// ORGANIZATION APIS //////////////////////////////
///
#[get("/")]
pub fn all_orgs(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>) -> Json<Vec<Organization>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_org_repository(&ds);
    Json(repo.all(&ctx))
}

#[post("/", format = "json", data = "<org>")]
pub fn create_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org: Json<Organization>) -> Result<Json<Organization>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_org_repository(&ds);
    match repo.create(&ctx, &org) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<id>", format = "json", data = "<org>")]
pub fn update_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, id: String, mut org: Json<Organization>) -> Result<Json<Organization>, Custom<String>> {
    org.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_org_repository(&ds);
    match repo.update(&ctx, &org) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<id>", format = "json")]
pub fn get_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, id: String) -> Result<Json<Organization>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_org_repository(&ds);
    match repo.get(&ctx, id.as_str()) {
        Some(org) => Ok(Json(org)),
        None => Err(Custom(Status::NotFound, format!("org with id {} not found", id))),
    }
}

#[delete("/<id>", format = "json")]
pub fn delete_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_org_repository(&ds);
    match repo.delete(&ctx, id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

///////////////////////////////// GROUPS APIS //////////////////////////////

#[get("/<org_id>/groups")]
pub fn get_groups_by_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String) -> Json<Vec<Group>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_group_repository(&ds);
    let mut groups = vec![];
    for (_, g) in repo.get_by_org(&ctx, org_id.as_str()) {
        groups.push(g);
    }
    Json(groups)
}

#[post("/<org_id>/groups", format = "json", data = "<group>")]
pub fn create_group(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, mut group: Json<Group>) -> Result<Json<Group>, Custom<String>> {
    group.organization_id = org_id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_group_repository(&ds);
    match repo.create(&ctx, &group) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<org_id>/groups/<id>", format = "json", data = "<group>")]
pub fn update_group(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String, mut group: Json<Group>) -> Result<Json<Group>, Custom<String>> {
    group.organization_id = org_id;
    group.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_group_repository(&ds);
    match repo.update(&ctx, &group) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<org_id>/groups/<id>", format = "json")]
pub fn get_group(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<Group>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_group_repository(&ds);
    match repo.get(&ctx, org_id.as_str(), id.as_str()) {
        Some(group) => Ok(Json(group)),
        None => Err(Custom(Status::NotFound, format!("group with id {} not found", id))),
    }
}

#[delete("/<org_id>/groups/<id>", format = "json")]
pub fn delete_group(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_group_repository(&ds);
    match repo.delete(&ctx, org_id.as_str(), id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}


#[put("/<org_id>/groups/<group_id>/principals/<principal_id>")]
pub fn add_principal_to_group(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, group_id: String, principal_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // group-id must exist within the organization
    if RepositoryLocator::build_group_repository(&ds).get(&ctx, &org_id.as_str(), &group_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("group with id {} not found within organization {}", group_id, org_id)));
    }
    let repo = RepositoryLocator::build_group_principal_repository(&ds);
    match repo.add_principal_to_group(&ctx, group_id.as_str(), principal_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[delete("/<org_id>/groups/<group_id>/principals/<principal_id>")]
pub fn delete_principal_from_group(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, group_id: String, principal_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // group-id must exist within the organization
    if RepositoryLocator::build_group_repository(&ds).get(&ctx, &org_id.as_str(), &group_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("group with id {} not found within organization {}", group_id, org_id)));
    }
    let repo = RepositoryLocator::build_group_principal_repository(&ds);
    match repo.delete_principal_from_group(&ctx, group_id.as_str(), principal_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

///////////////////////////////// ROLE APIS //////////////////////////////

#[get("/<org_id>/roles")]
pub fn get_roles_by_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String) -> Json<Vec<Role>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_role_repository(&ds);
    let mut roles = vec![];
    for (_, g) in repo.get_by_org(&ctx, org_id.as_str()) {
        roles.push(g);
    }
    Json(roles)
}

#[post("/<org_id>/roles", format = "json", data = "<role>")]
pub fn create_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, mut role: Json<Role>) -> Result<Json<Role>, Custom<String>> {
    role.organization_id = org_id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_role_repository(&ds);
    match repo.create(&ctx, &role) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<org_id>/roles/<id>", format = "json", data = "<role>")]
pub fn update_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String, mut role: Json<Role>) -> Result<Json<Role>, Custom<String>> {
    role.organization_id = org_id;
    role.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_role_repository(&ds);
    match repo.update(&ctx, &role) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<org_id>/roles/<id>", format = "json")]
pub fn get_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<Role>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_role_repository(&ds);
    match repo.get(&ctx, org_id.as_str(), id.as_str()) {
        Some(role) => Ok(Json(role)),
        None => Err(Custom(Status::NotFound, format!("role with id {} not found", id))),
    }
}

#[delete("/<org_id>/roles/<id>", format = "json")]
pub fn delete_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_role_repository(&ds);
    match repo.delete(&ctx, org_id.as_str(), id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}


#[put("/<org_id>/roles/<role_id>/principals/<principal_id>")]
pub fn add_principal_to_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, role_id: String, principal_id: String, cc: AssociationForm) -> Result<Json<usize>, Custom<String>> { // Form<>
    let ds = PooledDataSource {pool: &*pool};
    // role-id must exist within the organization
    if RepositoryLocator::build_role_repository(&ds).get(&ctx, &org_id.as_str(), &role_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("role with id {} not found within organization {}", role_id, org_id)));
    }
    let repo = RepositoryLocator::build_role_roleable_repository(&ds);
    //let cc = form.into_inner();
    match repo.add_principal_to_role(&ctx, role_id.as_str(), principal_id.as_str(), cc.constraints.as_str(), cc.effective_at(), cc.expired_at()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[delete("/<org_id>/roles/<role_id>/principals/<principal_id>")]
pub fn delete_principal_from_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, role_id: String, principal_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // role-id must exist within the organization
    if RepositoryLocator::build_role_repository(&ds).get(&ctx, &org_id.as_str(), &role_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("role with id {} not found within organization {}", role_id, org_id)));
    }
    let repo = RepositoryLocator::build_role_roleable_repository(&ds);
    match repo.delete_principal_from_role(&ctx, role_id.as_str(), principal_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<org_id>/roles/<role_id>/groups/<group_id>")]
pub fn add_group_to_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, role_id: String, group_id: String, cc: AssociationForm) -> Result<Json<usize>, Custom<String>> { // Form<>
    let ds = PooledDataSource {pool: &*pool};
    // role-id must exist within the organization
    if RepositoryLocator::build_role_repository(&ds).get(&ctx, &org_id.as_str(), &role_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("role with id {} not found within organization {}", role_id, org_id)));
    }
    let repo = RepositoryLocator::build_role_roleable_repository(&ds);
    //let cc = form.into_inner();
    match repo.add_group_to_role(&ctx, role_id.as_str(), group_id.as_str(), cc.constraints.as_str(), cc.effective_at(), cc.expired_at()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[delete("/<org_id>/roles/<role_id>/groups/<group_id>")]
pub fn delete_group_from_role(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, role_id: String, group_id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    // role-id must exist within the organization
    if RepositoryLocator::build_role_repository(&ds).get(&ctx, &org_id.as_str(), &role_id.as_str()) == None {
        return Err(Custom(Status::NotFound, format!("role with id {} not found within organization {}", role_id, org_id)));
    }
    let repo = RepositoryLocator::build_role_roleable_repository(&ds);
    match repo.delete_group_from_role(&ctx, role_id.as_str(), group_id.as_str()) {
        Ok(size) => Ok(Json(size)),
        Err(err) => Err(super::common::error_status(err)),
    }
}
///////////////////////////////// PRINCIPAL APIS //////////////////////////////

#[get("/<org_id>/principals")]
pub fn get_principals_by_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String) -> Json<Vec<Principal>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_principal_repository(&ds);
    Json(repo.get_by_org(&ctx, org_id.as_str()))
}

#[post("/<org_id>/principals", format = "json", data = "<principal>")]
pub fn create_principal(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, mut principal: Json<Principal>) -> Result<Json<Principal>, Custom<String>> {
    principal.organization_id = org_id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_principal_repository(&ds);
    match repo.create(&ctx, &principal) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<org_id>/principals/<id>", format = "json", data = "<principal>")]
pub fn update_principal(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String, mut principal: Json<Principal>) -> Result<Json<Principal>, Custom<String>> {
    principal.organization_id = org_id;
    principal.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_principal_repository(&ds);
    match repo.update(&ctx, &principal) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<org_id>/principals/<id>", format = "json")]
pub fn get_principal(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<Principal>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_principal_repository(&ds);
    match repo.get(&ctx, id.as_str()) {
        Some(principal) => Ok(Json(principal)),
        None => Err(Custom(Status::NotFound, format!("principal with id {} not found", id))),
    }
}

#[delete("/<org_id>/principals/<id>", format = "json")]
pub fn delete_principal(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_principal_repository(&ds);
    match repo.delete(&ctx, id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}


///////////////////////////////// LICENSE POLICY APIS //////////////////////////////

#[get("/<org_id>/licenses")]
pub fn get_licenses_by_org(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String) -> Json<Vec<LicensePolicy>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_license_policy_repository(&ds);
    Json(repo.get_by_org(&ctx, org_id.as_str()))
}

#[post("/<org_id>/licenses", format = "json", data = "<license>")]
pub fn create_license(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, mut license: Json<LicensePolicy>) -> Result<Json<LicensePolicy>, Custom<String>> {
    license.organization_id = org_id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_license_policy_repository(&ds);
    match repo.create(&ctx, &license) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[put("/<org_id>/licenses/<id>", format = "json", data = "<license>")]
pub fn update_license(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String, mut license: Json<LicensePolicy>) -> Result<Json<LicensePolicy>, Custom<String>> {
    license.organization_id = org_id;
    license.id = id;
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_license_policy_repository(&ds);
    match repo.update(&ctx, &license) {
        Ok(saved) => Ok(Json(saved)),
        Err(err) => Err(super::common::error_status(err)),
    }
}

#[get("/<org_id>/licenses/<id>", format = "json")]
pub fn get_license(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<LicensePolicy>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_license_policy_repository(&ds);
    match repo.get(&ctx, org_id.as_str(), id.as_str()) {
        Some(license) => Ok(Json(license)),
        None => Err(Custom(Status::NotFound, format!("license with id {} not found", id))),
    }
}

#[delete("/<org_id>/licenses/<id>", format = "json")]
pub fn delete_license(ctx: SecurityContext, pool: State<Pool<ConnectionManager<SqliteConnection>>>, org_id: String, id: String) -> Result<Json<usize>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let repo = RepositoryLocator::build_license_policy_repository(&ds);
    match repo.delete(&ctx, org_id.as_str(), id.as_str()) {
        Ok(count) => Ok(Json(count)),
        Err(err) => Err(super::common::error_status(err)),
    }
}
