//#![crate_name = "doc"]

//////////////////////////////////////////////////////////////////////////////////////////////
///
/// This module defines common domain classes
///
//////////////////////////////////////////////////////////////////////////////////////////////


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

#[cfg(test)]
mod tests {
    use plexrbac::common::*;

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
}
