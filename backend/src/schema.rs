diesel::table! {
    Admin (key) {
        key -> Integer,
        username -> Text,
        use_otp -> Bool,
        knows_otp -> Bool,
    }
}

diesel::table! {
    FileSharing (code) {
        code -> Text,
        file_path -> Text,
        use_password -> Bool,
        password -> Nullable<Text>,
    }
}

diesel::table! {
    Media (filepath) {
        filepath -> Text,
        genre -> Nullable<Text>,
        name -> Nullable<Text>,
        artist -> Nullable<Text>,
        cover -> Nullable<Text>,
    }
}

diesel::table! {
    MediaSources (folderpath) {
        folderpath -> Text,
        alias -> Text,
        enabled -> Bool,
    }
}

diesel::table! {
    PackageActions (key) {
        key -> Integer,
        last_database_update -> Nullable<Integer>,
    }
}

diesel::table! {
    RecommendedMedia (filepath) {
        filepath -> Text,
        lastview -> Integer,
        category -> Text,
    }
}

diesel::table! {
    Secrets (name) {
        name -> Text,
        value -> Nullable<Text>,
    }
}

diesel::table! {
    Settings (name) {
        name -> Text,
        value -> Nullable<Text>,
    }
}

diesel::table! {
    VaultNames (rowid) {
        rowid -> Integer,
        uuid -> Nullable<Text>,
        name -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    Admin,
    FileSharing,
    Media,
    MediaSources,
    PackageActions,
    RecommendedMedia,
    Secrets,
    Settings,
    VaultNames,
);
