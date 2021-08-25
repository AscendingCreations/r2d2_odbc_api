//! ODBC support for the `r2d2` connection pool Via odbc-api.
extern crate odbc_api;
use odbc_api::{
    sys::{AttrConnectionPooling, AttrCpMatch},
    Connection, Environment,
};
extern crate r2d2;

#[macro_use]
extern crate lazy_static;

pub use odbc_api::*;
use std::error::Error;
use std::fmt;

#[cfg(feature = "rocket_pooling")]
extern crate rocket_sync_db_pools;
use rocket_sync_db_pools::{Config, PoolResult, Poolable};
extern crate rocket;
use rocket::{Build, Rocket};

#[derive(Debug)]
pub struct ODBCConnectionManager {
    connection_string: String,
}

pub struct ODBCConnection(Connection<'static>);

unsafe impl Send for ODBCConnection {}

impl ODBCConnection {
    pub fn raw(&self) -> &Connection<'static> {
        &self.0
    }
}

pub struct ODBCEnv(Environment);

unsafe impl Sync for ODBCEnv {}

unsafe impl Send for ODBCEnv {}

#[derive(Debug)]
pub struct ODBCError(Box<dyn Error>);

lazy_static! {
    static ref ENV: ODBCEnv = unsafe {
        Environment::set_connection_pooling(AttrConnectionPooling::DriverAware).unwrap();
        let mut env = Environment::new().unwrap();
        env.set_connection_pooling_matching(AttrCpMatch::Strict)
            .unwrap();
        ODBCEnv(env)
    };
}

impl Error for ODBCError {
    fn description(&self) -> &str {
        "Error connecting to Database"
    }
}

impl fmt::Display for ODBCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<E: 'static> From<std::sync::PoisonError<E>> for ODBCError {
    fn from(err: std::sync::PoisonError<E>) -> Self {
        ODBCError(Box::new(err))
    }
}

impl From<odbc_api::Error> for ODBCError {
    fn from(err: odbc_api::Error) -> Self {
        ODBCError(Box::new(err))
    }
}

impl ODBCConnectionManager {
    /// Creates a new `ODBCConnectionManager`.
    pub fn new<S: Into<String>>(connection_string: S) -> ODBCConnectionManager {
        ODBCConnectionManager {
            connection_string: connection_string.into(),
        }
    }
}

/// An `r2d2::ManageConnection` for ODBC connections.
///
/// ## Example
///
/// ```rust,no_run
///extern crate anyhow;
///extern crate odbc_api;
///extern crate r2d2;
///extern crate r2d2_odbc_api;
///
///use anyhow::Error;
///use odbc_api::*;
///use r2d2_odbc_api::ODBCConnectionManager;
///use std::str;
///use std::thread;
///
///fn main() -> Result<(), Error> {
///    let manager = ODBCConnectionManager::new("DRIVER={SQL Server};SERVER=usmikzo-db01");
///    let pool = r2d2::Pool::new(manager).unwrap();
///
///    let mut children = vec![];
///    for i in 0..10i32 {
///        let pool = pool.clone();
///        children.push(thread::spawn(move || {
///            let pool_conn = pool.get().unwrap();
///            let conn = pool_conn.raw();
///
///            if let Some(cursor) = conn.execute("SELECT @@version", ()).unwrap() {
///                let mut buffers =
///                    buffers::TextRowSet::for_cursor(5000, &cursor, Some(4096)).unwrap();
///                let mut row_set_cursor = cursor.bind_buffer(&mut buffers).unwrap();
///
///                while let Some(batch) = row_set_cursor.fetch().unwrap() {
///                    if let Some(val) = batch.at(0, 0) {
///                        println!("THREAD {} {}", i, str::from_utf8(val).unwrap());
///                    }
///                }
///            };
///        }));
///    }
///
///    for child in children {
///       let _ = child.join();
///    }
///
///    Ok(())
///}
/// ```
impl r2d2::ManageConnection for ODBCConnectionManager {
    type Connection = ODBCConnection;
    type Error = ODBCError;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let env = &ENV.0;
        Ok(ODBCConnection(
            env.connect_with_connection_string(&self.connection_string)?,
        ))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        //Will work for most Databases If we encounter others we could try a different approach.
        conn.0.execute("SELECT 1;", ())?;
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[cfg(feature = "rocket_pooling")]
impl Poolable for ODBCConnection {
    type Manager = ODBCConnectionManager;
    type Error = std::convert::Infallible;

    fn pool(db_name: &str, rocket: &Rocket<Build>) -> PoolResult<Self> {
        let config = Config::from(db_name, rocket)?;
        let manager = ODBCConnectionManager::new(&config.url);
        Ok(r2d2::Pool::builder()
            .max_size(config.pool_size)
            .build(manager)?)
    }
}
