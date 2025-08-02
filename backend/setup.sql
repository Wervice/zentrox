CREATE TABLE Secrets (
	name varchar(255) NOT NULL, 
	value varchar(5000), 
	PRIMARY KEY (name)
);
CREATE TABLE MediaSources (
	folderpath varchar(2047) NOT NULL,
	alias varchar(255) NOT NULL,
	enabled boolean NOT NULL,
	PRIMARY KEY (folderpath)
);
CREATE TABLE Media (
	filepath varchar(2047) NOT NULL, 
	genre varchar(255), 
	name varchar(511), 
	artist varchar(255), 
	cover varchar(2047), 
	PRIMARY KEY (filepath)
);
CREATE TABLE RecommendedMedia (
	filepath varchar(2047) NOT NULL,
	lastview integer NOT NULL,
	category varchar(20) NOT NULL,
	PRIMARY KEY	(filepath)
);
CREATE TABLE Settings (
	name varchar(255) NOT NULL,
	value varchar(2047),
	PRIMARY KEY (name)
);
CREATE TABLE Admin (
	key int NOT NULL,
	username varchar(1047) NOT NULL,
	use_otp boolean NOT NULL,
	knows_otp boolean NOT NULL,
	PRIMARY KEY (key)
);
CREATE TABLE VaultNames (
	uuid varchar(127),
	name varchar(16383)
);
CREATE TABLE PackageActions (
	key integer NOT NULL,
	last_database_update integer,
	PRIMARY KEY (key)
);
CREATE TABLE FileSharing (
	code varchar(65) NOT NULL,
	file_path varchar(2048) NOT NULL,
	use_password int NOT NULL,
	password varchar(129),
	shared_since int NOT NULL,
	PRIMARY KEY (code)
);
