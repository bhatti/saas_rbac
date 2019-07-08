//#![crate_name = "doc"]
//#[macro_use]
//

use diesel::prelude::*;
use r2d2::{Pool};
use diesel::r2d2::ConnectionManager;
use std::sync::Arc;
use dotenv::dotenv;
use std::env;

//////////////////////////////////////////////////////////////////////////////////////////////
/// DataSource defines trait for generating connection
///
pub trait DataSource {
    fn new_connection(&self) -> Result<DbConn, diesel::result::Error>;
}

//use std::mem; -- move
//impl DataSource for DbConn {
//    /// Creates a new database connection using database pool
//    fn new_connection(&self) -> Result<DbConn, diesel::result::Error> {
//        Ok(self)
//    }
//}

#[database("sqlite_database")]
pub struct DbConn(SqliteConnection);

//pub struct DbConn(pub PooledConnection<ConnectionManager<SqliteConnection>>);
//impl Deref for DbConn {
//    type Target = SqliteConnection;
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//use rocket::Rocket;
//impl DbConn {
//    fn get_one(rocket: &Rocket) -> DbConn {
//        if let Ok(conn) = rocket.state::PooledConnection<ConnectionManager<SqliteConnection>>().get() {
//            DbConn(conn)
                     //.and_then(|pool| pool.0.get().ok())
                     //.map(DbConn)
//        }
//    }
//}

//////////////////////////////////////////////////////////////////////////////////////////////
/// DefaultDataSource is used to create default connection locator
///

//#[derive(Default)]
pub struct DefaultDataSource {
    pub pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,
}

//impl<'a, 'r> FromRequest<'a, 'r> for DefaultDataSource {
//    type Error = ();
//    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, ()> {
//        let ds = *req.guard::<State<DefaultDataSource>>()?;
//        Success(ds)
//    }
//}

// creates new pool
pub fn new_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok(); // Grabbing ENV vars

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // r2d2::Config Docs: https://docs.rs/r2d2/0.7.4/r2d2/struct.Config.html

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    Pool::new(manager).expect("Failed to create pool.")
}

impl DefaultDataSource {
    /// Creates instance of DefaultDataSource
    pub fn new() -> DefaultDataSource {
        let pool = new_pool();
        DefaultDataSource {pool: Arc::new(pool)}
    }
}

impl DataSource for DefaultDataSource {
    /// Creates a new database connection using database pool
    fn new_connection(&self) -> Result<DbConn, diesel::result::Error> {
        match self.pool.get() {
            Ok(conn) => Ok(DbConn(conn)),
            Err(err) => Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand, Box::new(format!("Failed to get connection {}", err)))),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////
/// PooledDataSource is used to create connection from rocket's pool
///
pub struct PooledDataSource<'a> {
    //pub rocket: &'a Rocket,
    pub pool: &'a Pool<ConnectionManager<SqliteConnection>>,
}

impl<'a> PooledDataSource<'a> {
    /// Creates instance of PooledDataSource
    pub fn new(pool: &Pool<ConnectionManager<SqliteConnection>>) -> PooledDataSource {
        PooledDataSource {pool: pool}
    }
}

impl<'a> DataSource for PooledDataSource<'a> {
    /// Creates a new database connection using database pool
    fn new_connection(&self) -> Result<DbConn, diesel::result::Error> {
        match self.pool.get() {
            Ok(conn) => Ok(DbConn(conn)),
            Err(err) => Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand, Box::new(format!("Failed to get connection {}", err)))),
        }
        //match DbConn::get_one(&self.rocket) {
        //    Some(conn) => Ok(conn),
        //    _ => Err(diesel::result::Error::DatabaseError(
        //            diesel::result::DatabaseErrorKind::UnableToSendCommand, Box::new(format!("Failed to get connection")))),
        //}
    }
}


/*
pub struct BasicDataSource {
}

impl DataSource for BasicDataSource {
    /// Creates a new database connection using database pool
    fn new_connection(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, diesel::result::Error> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        match SqliteConnection::establish(&database_url) {
            Ok(conn) => Ok(conn),
            Err(err) => Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand, Box::new(format!("Failed to get connection {}", err)))),
        }
    }
}
*/
