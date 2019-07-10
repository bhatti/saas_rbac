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
    pub data_source: &'a dyn super::data_source::DataSource
}

impl<'a> AuditRecordRepository<'a> {
    /// Returns latest audit records
    pub fn latest(&self, max: i64) -> Vec<PAuditRecord> {
        if let Ok(connection) = self.data_source.new_connection() {
            match rbac_audit_records::table
                .limit(max)
                .order(rbac_audit_records::created_at.desc())
                .load::<PAuditRecord>(&*connection) {
                Ok(v) => v,
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Creates new instance of the audit-record
    pub fn create_with(&self, message: &str, action: &str, context: &str, created_by: String) -> Result<usize, diesel::result::Error> {
        self.create(
            &PAuditRecord {
                 id: Uuid::new_v4().to_hyphenated().to_string(),
                 message: message.to_string(),
                 action: Some(action.to_string()),
                 context: Some(context.to_string()),
                 created_at: Some(Utc::now().naive_utc()),
                 created_by: Some(created_by.clone()),
            })
    }

    /// Creates new instance of the audit-record
    pub fn create(&self, rec: &PAuditRecord) -> Result<usize, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        diesel::insert_into(rbac_audit_records::table).values(rec).execute(&*connection)
    }
    /// Finds an audit-record by id
    pub fn get(&self, id: &str) -> Result<PAuditRecord, diesel::result::Error> {
        let connection = self.data_source.new_connection()?;
        rbac_audit_records::table.find(id.to_string()).get_result::<PAuditRecord>(&*connection)
    }

    /// Removes all audit-records from the database -- for testing
    pub fn clear(&self) {
        let connection = self.data_source.new_connection().unwrap();
        diesel::delete(rbac_audit_records::table).execute(&*connection).unwrap();
    }

}


#[cfg(test)]
mod tests {
    use plexrbac::persistence::locator::RepositoryLocator;
    use plexrbac::persistence::data_source::DefaultDataSource;
    use plexrbac::persistence::models::{PAuditRecord};

    #[test]
    fn test_save() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_audit_record_repository();
        repo.clear();

        let record = PAuditRecord::new("mymessage", Some("maction".to_string()), Some("mcontext".to_string()));
        let record_str = format!("{:?}", record);
        repo.create(&record).unwrap();

        let loaded = repo.get(record.id.as_str()).unwrap();
        assert_eq!(record_str, format!("{:?}", loaded));
    }

    #[test]
    fn test_get_latest() {
        let cf = DefaultDataSource::new();
        let locator = RepositoryLocator::new(&cf);
        let repo = locator.new_audit_record_repository();
        repo.clear();

        repo.create(&PAuditRecord::new("mymessage1", None, None)).unwrap();
        repo.create(&PAuditRecord::new("mymessage2", None, None)).unwrap();
        let results = repo.latest(10);
        assert_eq!(2, results.len());
    }
}
