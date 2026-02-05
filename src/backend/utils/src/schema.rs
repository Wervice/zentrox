diesel::table! {
    #[allow(non_snake_case)]
    Users (id) {
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
    #[allow(non_snake_case)]
    BlockedIPs (ip) {
        since -> BigInt,
        ip -> Text
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    Configuration (id) {
        server_name -> Text,
        media_enabled -> Bool,
        vault_enabled -> Bool,
        tls_cert -> Text,
        id -> Integer,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    Encryption (id) {
        argon2_salt -> Text,
        id -> Integer,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    FileSharing (code) {
        code -> Text,
        file_path -> Text,
        use_password -> Bool,
        password -> Nullable<Text>,
        shared_since -> BigInt,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    LoginRequestHistory (id) {
        time -> BigInt,
        username -> Text,
        ip -> Text,
        action -> Text,
        id -> Text
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    Media (file_path) {
        file_path -> Text,
        genre -> Nullable<Text>,
        name -> Nullable<Text>,
        artist -> Nullable<Text>,
        cover -> Nullable<Text>,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    RecommendedMedia (file_path) {
        file_path -> Text,
        last_view -> BigInt,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    MediaSources (directory_path) {
        directory_path -> Text,
        alias -> Text,
        enabled -> Bool,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    PackageActions (key) {
        key -> Integer,
        last_database_update -> Nullable<BigInt>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    Users,
    Configuration,
    Encryption,
    FileSharing,
    Media,
    MediaSources,
    PackageActions,
);
