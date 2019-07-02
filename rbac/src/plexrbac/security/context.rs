//#![crate_name = "doc"]

use std::collections::HashMap;
use plexrbac::utils::evaluator;
use plexrbac::domain::models::ValueWrapper;


////////////////////////////////////////////////////////////////////////////////
/// SecurityContext defines context of security invocation
///
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityContext {
    pub realm_id: String,
    pub principal_id: String,
    pub properties: HashMap<String, ValueWrapper>,
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



#[cfg(test)]
mod tests {
    use plexrbac::security::context::{SecurityContext};
    use plexrbac::domain::models::{ValueWrapper};
    use chrono::{Utc, Datelike};

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
