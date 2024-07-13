const otplib = require("otplib");

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
	topt.keyuri(username, "Zentrox", secret);
};

module.exports = { otpAuth, currentOtp, otpUri, generateSecret };
