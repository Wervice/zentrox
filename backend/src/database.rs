use diesel::{Connection as _, prelude::*};
use rusqlite::Connection;
use std::path::PathBuf;

use crate::models::AdminAccount;

/// Get the absolute path for the database by joining ~/.local/share with zentrox/database.db.
pub fn get_database_location() -> PathBuf {
    dirs::data_local_dir()
        .unwrap()
        .join("zentrox")
        .join("database.db")
}

pub fn base_database_setup() -> Result<(), String> {
    let connection = Connection::open(get_database_location().to_str().unwrap()).unwrap();
    let s = connection.execute_batch(include_str!("../setup.sql"));

    match s {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn establish_connection() -> SqliteConnection {
    let db_url = get_database_location();
    SqliteConnection::establish(
        db_url
            .as_os_str()
            .to_str()
            .expect("Failed to get database URL"),
    )
    .expect("Failed to establish database connection.")
}

pub fn get_administrator_account() -> AdminAccount {
    use crate::{models::AdminAccount, schema::Admin::dsl::*};

    Admin
        .select(AdminAccount::as_select())
        .first(&mut establish_connection())
        .unwrap()
}
