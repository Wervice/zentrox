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
	name varchar(255),
	value varchar(2047),
	PRIMARY KEY (name)
);
CREATE TABLE Admin (
	key int,
	username varchar(1047),
	use_otp boolean,
	knows_otp boolean,
	PRIMARY KEY (key)
);
CREATE TABLE VaultNames (
	uuid varchar(127),
	name varchar(16383)
);
CREATE TABLE PackageActions (
	key integer,
	last_database_update integer,
	PRIMARY KEY (key)
);
CREATE TABLE FileSharingCollection (
	id varchar(40) NOT NULL,
	real_name varchar(1047) NOT NULL,
	time_limit integer NOT NULL,
	password_protection boolean NOT NULL,
	download_counter integer NOT NULL,
	added_timestamp integer NOT NULL,
	PRIMARY KEY (id)
);
CREATE TABLE FileSharingPasswordAttempts (
	ip varchar(128) NOT NULL,
	id varchar(40) NOT NULL,
	attempts int NOT NULL,
	PRIMARY KEY (ip)
);
CREATE TABLE FileSharingLocationAttempts (
	ip varchar(128) NOT NULL,
	attempts int NOT NULL
);
