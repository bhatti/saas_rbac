//#![crate_name = "doc"]

use plexrbac::security::request::{PermissionRequest};
use plexrbac::security::response::{PermissionResponse};
use plexrbac::security::manager::{SecurityManager};
use plexrbac::persistence::locator::RepositoryLocator;
use plexrbac::persistence::data_source::PooledDataSource;

use rocket::{State};
use rocket_contrib::json::{Json};
use rocket::http::Status;
use rocket::response::status::Custom;

use diesel::prelude::*;
use plexrbac::common::{SecurityContext};
use r2d2::{Pool};
use diesel::r2d2::ConnectionManager;

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// REST APIs for checking security
///
//////////////////////////////////////////////////////////////////////////////////////////////


///////////////////////////////// PERMISSION CHECK //////////////////////////////
///
#[get("/")]
pub fn check(req: PermissionRequest, pool: State<Pool<ConnectionManager<SqliteConnection>>>) -> Result<Json<PermissionResponse>, Custom<String>> {
    let ds = PooledDataSource {pool: &*pool};
    let locator = RepositoryLocator::new(&ds);
    let pm = locator.new_persistence_manager();
    let sm = SecurityManager::new(pm);
    match sm.check(&req) {
        Ok(resp) => Ok(Json(resp)),
        Err(err) => Err(Custom(Status::Unauthorized, format!("Failed to authorize {:?} - {:?}", req, err)))
    }
}

