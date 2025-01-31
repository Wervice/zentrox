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
CREATE TABLE Ftp (
	key int,
	running boolean NOT NULL,
	pid int,
	username varchar(2047),
	local_root varchar(2047),
	PRIMARY KEY	(key)
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
)
