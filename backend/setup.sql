CREATE TABLE Secrets (
	name varchar(255) NOT NULL, 
	value varchar(1023), 
	PRIMARY KEY (name)
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
	lastview date NOT NULL,
	PRIMARY KEY	(filepath)
);
CREATE TABLE Ftp (
	key int,
	running boolean NOT NULL,
	pid int,
	username varchar(2047),
	password varchar(2047),
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
	PRIMARY KEY (key)
);
