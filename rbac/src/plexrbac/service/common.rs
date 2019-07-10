//#![crate_name = "doc"]

use rocket::response::status::Custom;
use rocket::http::Status;
use plexrbac::common::RbacError;
use rocket::request::{self, Request, FromRequest};
use rocket::outcome::Outcome::*;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use chrono::format::ParseResult;

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


#[derive(Debug, Clone)]
pub struct AssociationForm {
    pub scope: String,
    pub constraints: String,
    pub effective_at: String,
    pub expired_at: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for AssociationForm {
    type Error = std::convert::Infallible;

    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let scope = req.get_query_value("scope").and_then(|r| r.ok()).unwrap_or("".into());
        let constraints = req.get_query_value("constraints").and_then(|r| r.ok()).unwrap_or("".into());
        let effective_at = req.get_query_value("effective_at").and_then(|r| r.ok()).unwrap_or("".into());
        let expired_at = req.get_query_value("expired_at").and_then(|r| r.ok()).unwrap_or("".into());
        let form = AssociationForm {scope: scope, constraints: constraints, effective_at: effective_at, expired_at: expired_at};
        if !form.effective_at.is_empty() && form.parse(form.effective_at.as_str()).is_err() {
            //return Failure((Status::BadRequest, "effective_at date must be of in format YYY-MM-DD".to_string()));
        }
        if !form.expired_at.is_empty() && form.parse(form.expired_at.as_str()).is_err() {
            //return Failure((Status::BadRequest, "expired_at date must be of in format YYY-MM-DD".to_string()));
        }
        Success(form)
    }
}

impl AssociationForm {
    pub fn parse(&self, val: &str) -> ParseResult<NaiveDateTime> {
       NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S%z")
    }

    pub fn effective_at(&self) -> NaiveDateTime {
        if !self.effective_at.is_empty() {
            self.parse(self.effective_at.as_str()).unwrap()
        } else {
            Utc::now().naive_utc()
        }
    }

    pub fn expired_at(&self) -> NaiveDateTime {
        if !self.expired_at.is_empty() {
            self.parse(self.expired_at.as_str()).unwrap()
        } else {
            NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0)
        }
    }

}

