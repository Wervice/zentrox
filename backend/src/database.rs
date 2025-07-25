use diesel::{prelude::*, Connection as _};
use rusqlite::Connection;
use std::{fmt::Display, path::PathBuf};

use crate::models::AdminAccount;

pub const ST_BOOL_TRUE: &str = "TRUE";
pub const ST_BOOL_FALSE: &str = "FALSE";

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

pub fn get_secret_by_name<T: Display>(secret_name: T) -> Option<String> {
    use crate::models::Secret;
    use crate::schema::Secrets::dsl::*;

    match Secrets
        .filter(name.eq(secret_name.to_string()))
        .select(Secret::as_select())
        .first(&mut establish_connection())
    {
        Ok(v) => Some(v.value?),
        Err(_) => None,
    }
}

pub fn get_setting_by_name<T: Display>(setting_name: T) -> Option<String> {
    use crate::models::Setting;
    use crate::schema::Settings::dsl::*;

    match Settings
        .filter(name.eq(setting_name.to_string()))
        .select(Setting::as_select())
        .first(&mut establish_connection())
    {
        Ok(v) => Some(v.value?),
        Err(_) => None,
    }
}
