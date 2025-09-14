diesel::table! {
    Admin (id) {
        id -> Integer,
        username -> Text,
        use_otp -> Bool,
        otp_secret -> Nullable<Text>,
        password_hash -> Text,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    Configuration (id) {
        server_name -> Text,
        media_enabled -> Bool,
        vault_enabled -> Bool,
        tls_cert -> Text,
        id -> Integer,
    }
}

diesel::table! {
    Encryption (id) {
        argon2_salt -> Text,
        id -> Integer,
    }
}

diesel::table! {
    FileSharing (code) {
        code -> Text,
        file_path -> Text,
        use_password -> Bool,
        password -> Nullable<Text>,
        shared_since -> BigInt,
    }
}

diesel::table! {
    Media (file_path) {
        file_path -> Text,
        genre -> Nullable<Text>,
        name -> Nullable<Text>,
        artist -> Nullable<Text>,
        cover -> Nullable<Text>,
    }
}

diesel::table! {
    RecommendedMedia (file_path) {
        file_path -> Text,
        last_view -> BigInt,
    }
}

diesel::table! {
    MediaSources (directory_path) {
        directory_path -> Text,
        alias -> Text,
        enabled -> Bool,
    }
}

diesel::table! {
    PackageActions (key) {
        key -> Integer,
        last_database_update -> Nullable<BigInt>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    Admin,
    Configuration,
    Encryption,
    FileSharing,
    Media,
    MediaSources,
    PackageActions,
);
