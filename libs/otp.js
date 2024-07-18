const otplib = require("otplib");
const { readDatabase, writeDatabase } = require("./mapbase");
const path = require("path");
const { zentroxInstallationPath } = require("./commonVariables");

const topt = otplib.authenticator;

const generateSecret = () => {
	return otplib.authenticator.generateSecret();
};

const otpAuth = (code, secret) => {
	return topt.check(code, secret);
};

const currentOtp = (secret) => {
	return topt.generate(secret);
};

const otpUri = (username, secret) => {
	return topt.keyuri(username, "Zentrox", secret);
};

function firstOtp() {
	return (
		readDatabase(path.join(zentroxInstallationPath, "config.db"), "useOtp") ===
			"1" &&
		String(
			readDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"otpSecret",
			),
		).length === 0
	);
}

function firstOtpView() {
	return !(
		readDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"knowsOtpSecret",
		) === "1"
	);
}

function knowsOtpSecret() {
	writeDatabase(
		path.join(zentroxInstallationPath, "config.db"),
		"knowsOtpSecret",
		"1",
	);
}

module.exports = {
	otpAuth,
	currentOtp,
	otpUri,
	generateSecret,
	firstOtp,
	firstOtpView,
	knowsOtpSecret,
};
