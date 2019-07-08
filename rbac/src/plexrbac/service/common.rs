//#![crate_name = "doc"]

use rocket::response::status::Custom;
use rocket::http::Status;
use plexrbac::common::RbacError;

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// Common Helper Methods
///
//////////////////////////////////////////////////////////////////////////////////////////////
pub fn error_status(err: RbacError) -> Custom<String> {
    match err {
        RbacError::NotFound(_) => Custom(Status::NotFound, err.to_string()),
        RbacError::Duplicate(_) => Custom(Status::Conflict, err.to_string()),
        _ => {
            let emsg = err.to_string();
            if emsg.contains("UNIQUE constraint") {
                Custom(Status::Conflict, emsg)
            } else {
                Custom(Status::InternalServerError, emsg)
            } 
        } 
    }
}
