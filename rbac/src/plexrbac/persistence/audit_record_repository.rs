//#![crate_name = "doc"]

extern crate uuid as uuu;
use chrono::{Utc};
use diesel::prelude::*;
use super::schema::rbac_audit_records;
use super::models::{PAuditRecord};
use self::uuu::Uuid;

//////////////////////////////////////////////////////////////////////////////////////////////
/// AuditRecordRepository defines methods for accessing and persisting audit records
///
pub struct AuditRecordRepository<'a> {
    pub factory: &'a super::factory::RepositoryFactory
}

impl<'a> AuditRecordRepository<'a> {
    /// Returns latest audit records
    pub fn latest(&self, max: i64) -> Vec<PAuditRecord> {
        let connection = self.factory.new_connection();
        match rbac_audit_records::table
            .limit(max)
            .order(rbac_audit_records::created_at.desc())
            .load::<PAuditRecord>(&*connection) {
            Ok(v) => v,
            _ => vec![],
        }
    }

    /// Creates new instance of the audit-record
    pub fn create_with(&self, message: &str, action: &str, context: &str, created_by: String) -> Option<diesel::result::Error> {
        self.create(
            &PAuditRecord {
                 id: Uuid::new_v4().to_hyphenated().to_string(),
                 message: message.to_string(),
                 action: Some(action.to_string()),
                 context: Some(context.to_string()),
                 created_at: Utc::now().naive_utc(),
                 created_by: Some(created_by.clone()),
            })
    }

    /// Creates new instance of the audit-record
    pub fn create(&self, rec: &PAuditRecord) -> Option<diesel::result::Error> {
        let connection = self.factory.new_connection();
        match diesel::insert_into(rbac_audit_records::table).values(rec).execute(&*connection) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    /// Removes all audit-records from the database
    pub fn clear(&self) -> usize {
        let connection = self.factory.new_connection();
        match diesel::delete(rbac_audit_records::table).execute(&*connection) {
            Ok(n) => n,
            _ => 0,
        }
    }

    /// Finds an audit-record by id
    pub fn get(&self, id: &str) -> Option<PAuditRecord> {
        let connection = self.factory.new_connection();
        match rbac_audit_records::table.find(id.to_string()).get_result::<PAuditRecord>(&*connection) {
            Ok(rec) => {
                Some(rec)
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::factory::RepositoryFactory;
    use plexrbac::persistence::models::{PAuditRecord};

    #[test]
    fn test_save() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_audit_record_repository();
        repo.clear();

        let record = PAuditRecord::new("mymessage", Some("maction".to_string()), Some("mcontext".to_string()));
        let record_str = format!("{:?}", record);
        repo.create(&record);

        let loaded = repo.get(record.id.as_str()).unwrap();
        assert_eq!(record_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_get_latest() {
        let factory = RepositoryFactory::new();
        let repo = factory.new_audit_record_repository();
        repo.clear();

        repo.create(&PAuditRecord::new("mymessage1", None, None));
        repo.create(&PAuditRecord::new("mymessage2", None, None));
        let results = repo.latest(10);
        assert_eq!(2, results.len());
    }
}
