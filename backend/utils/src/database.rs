use diesel::{
    Connection,
    connection::SimpleConnection,
    prelude::*,
    r2d2::{self, ConnectionManager, CustomizeConnection, Pool},
};
use std::{path::PathBuf, time::Duration};

/// Get the absolute path for the database by joining ~/.local/share with zentrox/database.db.
pub fn get_database_location() -> PathBuf {
    dirs::data_local_dir()
        .unwrap()
        .join("zentrox")
        .join("database.db")
}

pub fn base_database_setup() -> Result<(), Box<dyn std::error::Error>> {
    let connection = rusqlite::Connection::open(get_database_location().to_str().unwrap()).unwrap();
    let s = connection.execute_batch(include_str!("../../assets/setup.sql"));
    s?;
    Ok(())
}

#[derive(Debug)]
pub struct ConnectionOptions {
    pub busy_timeout: Option<Duration>,
}

impl CustomizeConnection<SqliteConnection, r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), r2d2::Error> {
        if let Some(timeout) = self.busy_timeout {
            conn.batch_execute(&format!("PRAGMA busy_timeout={};", timeout.as_millis()))?;
        }
        Ok(())
    }
}

pub fn establish_direct_connection() -> SqliteConnection {
    let db_url = get_database_location();
    SqliteConnection::establish(
        db_url
            .as_os_str()
            .to_str()
            .expect("Failed to get database URL"),
    )
    .expect("Failed to establish database connection.")
}

pub fn create_connection_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    let db_url = get_database_location();
    let mgr = ConnectionManager::<SqliteConnection>::new(
        db_url
            .as_os_str()
            .to_str()
            .expect("Failed to retrieve database URL."),
    );
    let options = ConnectionOptions {
        busy_timeout: Some(Duration::from_millis(1000)),
    };
    r2d2::Pool::builder()
        .connection_customizer(Box::new(options))
        .max_size(10)
        .connection_timeout(Duration::from_millis(500))
        .build(mgr)
        .expect("Failed to create pool.")
}
