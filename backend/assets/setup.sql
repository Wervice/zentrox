CREATE TABLE Admin (
	username TEXT NOT NULL,
	use_otp INTEGER NOT NULL,
	otp_secret TEXT,
	password_hash TEXT NOT NULL,
	created_at INTEGER NOT NULL,
	updated_at INTEGER NOT NULL,
	id INTEGER PRIMARY KEY CHECK (id = 0) NOT NULL -- Only one Admin account will be supported in the code
);
CREATE TABLE Configuration (
	server_name TEXT NOT NULL,
	media_enabled INTEGER NOT NULL,
	vault_enabled INTEGER NOT NULL,
	tls_cert TEXT NOT NULL,
	id INTEGER PRIMARY KEY CHECK (id = 0) NOT NULL -- This table provides general configuration details and there may only be one such config entry
);
CREATE TABLE Encryption (
	argon2_salt TEXT NOT NULL,
	id INTEGER PRIMARY KEY CHECK (id = 0) NOT NULL
);
CREATE TABLE FileSharing (
	code TEXT NOT NULL CHECK (length(code) = 64), -- A code should always be 64 characters in length
	file_path TEXT NOT NULL,
	use_password INTEGER NOT NULL,
	password TEXT,
	shared_since INTEGER NOT NULL,
	PRIMARY KEY (code)
);
CREATE TABLE Media (
	file_path TEXT NOT NULL, 
	genre TEXT, 
	name TEXT, 
	artist TEXT,
	cover TEXT,
	PRIMARY KEY (file_path)
);
CREATE TABLE RecommendedMedia (
	file_path TEXT NOT NULL, 
	last_view INTEGER NOT NULL,
	PRIMARY KEY (file_path)
);
CREATE TABLE MediaSources (
	directory_path TEXT NOT NULL,
	alias TEXT NOT NULL,
	enabled INTEGER NOT NULL,
	PRIMARY KEY (directory_path)
);
CREATE TABLE PackageActions (
	key INTEGER NOT NULL,
	last_database_update INTEGER,
	PRIMARY KEY (key)
);
