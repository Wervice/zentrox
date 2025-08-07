use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::Admin)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AdminAccount {
    pub id: i32,
    pub username: String,
    pub use_otp: bool,
    pub knows_otp: bool,
    pub otp_secret: Option<String>,
    pub password_hash: String,
    pub created_at: i64,
    pub updated_at: i64
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::Configuration)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Configurations {
    pub server_name: String,
    pub media_enabled: bool,
    pub vault_enabled: bool,
    pub tls_cert: String,
    pub id: i32,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::PackageActions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PackageAction {
    pub last_database_update: Option<i64>,
    pub key: i32,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::MediaSources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MediaSource {
    pub directory_path: String,
    pub alias: String,
    pub enabled: bool,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = crate::schema::Media)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MediaEntry {
    pub file_path: String,
    pub genre: Option<String>,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub cover: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::RecommendedMedia)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RecommendedMediaEntry {
    pub file_path: String,
    pub category: String,
    pub last_view: i64,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, serde::Serialize)]
#[diesel(table_name = crate::schema::FileSharing)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SharedFile {
    pub code: String,
    pub file_path: String,
    pub use_password: bool,
    pub password: Option<String>,
    pub shared_since: i64,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, serde::Serialize, Debug)]
#[diesel(table_name = crate::schema::Encryption)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Secrets {
    pub argon2_salt: String,
    pub id: i32
}
