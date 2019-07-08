//#![crate_name = "doc"]
//![feature(proc_macro_hygiene, decl_macro, never_type)]

use std::collections::HashMap;
use plexrbac::utils::evaluator;

use rocket::request::{self, Request, FromRequest};
use rocket::outcome::Outcome::*;


//////////////////////////////////////////////////////////////////////////////////////////////
///
/// This module defines common domain classes
///
//////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
/// SecurityContext defines context of security invocation
///
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityContext {
    pub realm_id: String,
    pub principal_id: String,
    pub properties: HashMap<String, ValueWrapper>,
}

impl<'a, 'r> FromRequest<'a, 'r> for SecurityContext {
    type Error = (); // experimental exclamation;
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let realm = req.headers().get_one("X-Realm").unwrap_or_else(||"");
        let principal = req.headers().get_one("X-Principal").unwrap_or_else(||"");
        Success(SecurityContext::new(realm, principal))
    }
}

impl SecurityContext {
    /// Creates new instance of security context
    pub fn new(realm_id: &str, principal_id: &str) -> SecurityContext {
        SecurityContext {
            realm_id: realm_id.to_string(),
            principal_id: principal_id.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, val: ValueWrapper) {
        self.properties.insert(name.to_string(), val);
    }

    pub fn evaluate(&self, expr: &str) -> Result<bool, evalexpr::EvalexprError> {
        evaluator::evaluate(expr, &self.properties)
    }
}


/// Constants
#[derive(Debug, Clone, PartialEq)]
pub enum Constants {
    Principal,
    Role,
    Group,
    LicensePolicy,
    Allow,
    Deny
}

impl std::fmt::Display for Constants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Status
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    INFLIGHT,
    PENDING,
    FAILED,
    COMPLETED,
    UNKNOWN
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


/// Sample ResourceType - feel free to update
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Api,
    Method,
    Data,
    Feature,
    Report,
    Config,
    Job,
    App,
    Network,
    Device,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


/// Sample ActionType - feel free to update
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    READ,
    VIEW,
    UPDATE,
    DELETE,
    CREATE,
    APPROVE,
    SUBMIT,
    UPLOAD,
    DOWNLOAD,
    BUILD,
    EXECUTE,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


/// ValueWrapper is used to wrap values inside hashmap
#[derive(Debug, Clone, PartialEq)]
pub enum ValueWrapper {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}


use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RbacError {
    Duplicate(String),
    NotFound(String),
    Persistence(String),
    Security(String),
    Evaluation(String),
    QuotaExceeded(String),
    Custom(String),
}

impl fmt::Display for RbacError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RbacError::Duplicate(ref e) => e.fmt(f),
            RbacError::NotFound(ref e) => e.fmt(f),
            RbacError::Persistence(ref e) => e.fmt(f),
            RbacError::Security(ref e) => e.fmt(f),
            RbacError::Evaluation(ref e) => e.fmt(f),
            RbacError::QuotaExceeded(ref e) => e.fmt(f),
            RbacError::Custom(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for RbacError {
    fn description(&self) -> &str {
        match *self {
            RbacError::Duplicate(ref e) => e.as_str(),
            RbacError::NotFound(ref e) => e.as_str(),
            RbacError::Persistence(ref e) => e.as_str(),
            RbacError::Security(ref e) => e.as_str(),
            RbacError::Evaluation(ref e) => e.as_str(),
            RbacError::QuotaExceeded(ref e) => e.as_str(),
            RbacError::Custom(ref e) => e.as_str(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            RbacError::Duplicate(_) => None,
            RbacError::NotFound(_) => None,
            RbacError::Persistence(_) => None,
            RbacError::Security(_) => None,
            RbacError::Evaluation(_) => None,
            RbacError::QuotaExceeded(_) => None,
            RbacError::Custom(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::common::*;
    use chrono::{Utc, Datelike};

    #[test]
    fn test_resource_type() {
        assert_eq!("Api".to_string(), ResourceType::Api.to_string());
        assert_eq!("Method".to_string(), ResourceType::Method.to_string());
        assert_eq!("Data".to_string(), ResourceType::Data.to_string());
        assert_eq!("Feature".to_string(), ResourceType::Feature.to_string());
        assert_eq!("Report".to_string(), ResourceType::Report.to_string());
        assert_eq!("Config".to_string(), ResourceType::Config.to_string());
        assert_eq!("Job".to_string(), ResourceType::Job.to_string());
        assert_eq!("App".to_string(), ResourceType::App.to_string());
        assert_eq!("Network".to_string(), ResourceType::Network.to_string());
        assert_eq!("Device".to_string(), ResourceType::Device.to_string());
    }

    #[test]
    fn test_action() {
        assert_eq!("READ".to_string(), ActionType::READ.to_string());
        assert_eq!("VIEW".to_string(), ActionType::VIEW.to_string());
        assert_eq!("UPDATE".to_string(), ActionType::UPDATE.to_string());
        assert_eq!("DELETE".to_string(), ActionType::DELETE.to_string());
        assert_eq!("CREATE".to_string(), ActionType::CREATE.to_string());
        assert_eq!("APPROVE".to_string(), ActionType::APPROVE.to_string());
        assert_eq!("SUBMIT".to_string(), ActionType::SUBMIT.to_string());
        assert_eq!("UPLOAD".to_string(), ActionType::UPLOAD.to_string());
        assert_eq!("DOWNLOAD".to_string(), ActionType::DOWNLOAD.to_string());
        assert_eq!("BUILD".to_string(), ActionType::BUILD.to_string());
        assert_eq!("EXECUTE".to_string(), ActionType::EXECUTE.to_string());
    }

    #[test]
    fn test_status() {
        assert_eq!("INFLIGHT".to_string(), Status::INFLIGHT.to_string());
        assert_eq!("PENDING".to_string(), Status::PENDING.to_string());
        assert_eq!("FAILED".to_string(), Status::FAILED.to_string());
        assert_eq!("COMPLETED".to_string(), Status::COMPLETED.to_string());
        assert_eq!("UNKNOWN".to_string(), Status::UNKNOWN.to_string());
    }

    #[test]
    fn test_error() {
        assert_eq!("test".to_string(), RbacError::Persistence("test".to_string()).to_string());
        assert_eq!("test".to_string(), RbacError::Security("test".to_string()).to_string());
        assert_eq!("test".to_string(), RbacError::Evaluation("test".to_string()).to_string());
        assert_eq!("test".to_string(), RbacError::QuotaExceeded("test".to_string()).to_string());
        assert_eq!("test".to_string(), RbacError::Custom("test".to_string()).to_string());
    }

    #[test]
    fn test_bool_evaluate() {
        let mut ctx =  SecurityContext::new("org", "user");
        ctx.add("tr".into(), ValueWrapper::Bool(true));
        ctx.add("fa".into(), ValueWrapper::Bool(false));
        ctx.add("five".into(), ValueWrapper::Int(5));
        ctx.add("six".into(), ValueWrapper::Int(6));
        ctx.add("half".into(), ValueWrapper::Float(0.5));
        ctx.add("zero".into(), ValueWrapper::Int(0));

        assert_eq!(ctx.evaluate("tr"), Ok(true));
        assert_eq!(ctx.evaluate("fa"), Ok(false));
        assert_eq!(ctx.evaluate("tr && false"), Ok(false));
        assert!(ctx.evaluate("five + six").is_err());
        assert_eq!(ctx.evaluate("five < six && true"), Ok(true));
        assert!(ctx.evaluate("11").is_err());
    }

    #[test]
    fn test_regex_match() {
        let mut ctx =  SecurityContext::new("org", "user");
        ctx.add("rx".into(), ValueWrapper::String(r"^\d{4}-\d{2}-\d{2}$".to_string()));
        ctx.add("s".into(), ValueWrapper::String("2014-01-01".to_string()));
        assert_eq!(ctx.evaluate("regex_match(rx, s)"), Ok(true));
    }

    #[test]
    fn test_geo() {
        let mut ctx =  SecurityContext::new("org", "user");
        ctx.add("lat1".into(), ValueWrapper::Float(47.620422));
        ctx.add("lon1".into(), ValueWrapper::Float(-122.349358));
       
        ctx.add("lat2".into(), ValueWrapper::Float(46.879967));
        ctx.add("lon2".into(), ValueWrapper::Float(-121.726906));

        assert_eq!(ctx.evaluate("geo_distance_km(lat1, lon1, lat2, lon2) < 100"), Ok(true));
    }

    #[test]
    fn test_regex() {
        let mut ctx =  SecurityContext::new("org", "user");
        ctx.add("rx".into(), ValueWrapper::String("works".to_string()));
        ctx.add("s".into(), ValueWrapper::String("works on my machine".to_string()));
        assert_eq!(ctx.evaluate("regex_find(rx, s)"), Ok(true));
    }

    #[test]
    fn test_date() {
        let mut ctx =  SecurityContext::new("org", "user");
        ctx.add("dow".into(), ValueWrapper::String(format!("{:?}", Utc::now().naive_utc().weekday())));
        assert_eq!(ctx.evaluate(format!("current_ordinal() == {}", Utc::now().naive_utc().ordinal()).as_str()), Ok(true));
        assert_eq!(ctx.evaluate("current_weekday() == dow"), Ok(true));
    }
}
