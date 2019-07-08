//#![feature(plugin)]
//#![plugin(rocket_codegen)]


#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate log;
#[macro_use] extern crate env_logger;

extern crate chrono;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate regex;
extern crate evalexpr;

use std::sync::Mutex;
use std::collections::HashMap;

mod plexrbac;

use rocket::Rocket;
use rocket_contrib::json::{JsonValue};
use rocket::fairing::AdHoc;

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`.
embed_migrations!();

use plexrbac::service::realm;
use plexrbac::service::organization;
use plexrbac::persistence::data_source::DbConn;
use plexrbac::persistence::data_source::new_pool;

fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = DbConn::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
    Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
 }


fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        //.attach(AdHoc::on_attach("DB Connection", |rocket| {
        //    Ok(rocket.manage(rocket))
        //}))
        .mount("/api/realms", routes![
               realm::all_realms,
               realm::create_realm,
               realm::update_realm,
               realm::get_realm,
               realm::delete_realm,
               realm::get_resources_by_realm,
               realm::create_resource,
               realm::update_resource,
               realm::get_resource,
               realm::delete_resource,
               realm::get_instances,
               realm::create_instance,
               realm::update_instance,
               realm::get_instance,
               realm::delete_instance,
               realm::get_quotas,
               realm::create_quota,
               realm::update_quota,
               realm::get_quota,
               realm::delete_quota,
               realm::get_realm_claims,
               realm::get_claims,
               realm::create_claim,
               realm::update_claim,
               realm::get_claim,
               realm::delete_claim,
               realm::add_principal_to_claim,
               realm::delete_principal_from_claim,
               realm::add_role_to_claim,
               realm::delete_role_from_claim,
               realm::add_license_to_claim,
               realm::delete_license_from_claim
        ])
        .mount("/api/orgs", routes![
               organization::all_orgs,
               organization::create_org,
               organization::update_org,
               organization::get_org,
               organization::delete_org,
               organization::get_groups_by_org,
               organization::create_group,
               organization::update_group,
               organization::get_group,
               organization::delete_group,
               organization::add_principal_to_group,
               organization::delete_principal_from_group,
               organization::get_roles_by_org,
               organization::create_role,
               organization::update_role,
               organization::get_role,
               organization::delete_role,
               organization::add_principal_to_role,
               organization::delete_principal_from_role,
               organization::add_group_to_role,
               organization::delete_group_from_role,
               organization::get_principals_by_org,
               organization::create_principal,
               organization::update_principal,
               organization::get_principal,
               organization::delete_principal,
               organization::get_licenses_by_org,
               organization::create_license,
               organization::update_license,
               organization::get_license,
               organization::delete_license
                   ])
        .register(catchers![not_found])
        .manage(new_pool())
        .manage(Mutex::new(HashMap::<String, String>::new()))
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

fn main() {
    let rocket = rocket();
    rocket.launch();
}
