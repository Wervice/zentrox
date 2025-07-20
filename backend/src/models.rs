use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::Admin)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AdminAccount {
    pub key: i32,
    pub username: String,
    pub use_otp: bool,
    pub knows_otp: bool,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::Secrets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Secret {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::Settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Setting {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::PackageActions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PackageAction {
    pub last_database_update: Option<i32>,
    pub key: i32,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::MediaSources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MediaSource {
    pub folderpath: String,
    pub alias: String,
    pub enabled: bool,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::Media)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MediaEntry {
    pub filepath: String,
    pub genre: Option<String>,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub cover: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::RecommendedMedia)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RecommendedMediaEntry {
    pub filepath: String,
    pub lastview: i32,
    pub category: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::FileSharing)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SharedFile {
    pub code: String,
    pub file_path: String,
    pub use_password: bool,
    pub password: Option<String>,
}
