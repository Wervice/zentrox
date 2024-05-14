const chpr = require("child_process");

function execSyncAsSudo(command, sudo) {
	var execOut = chpr.execSync(
		`echo "${sudo.replaceAll('"', '\\"').replaceAll("'", "\\'").replaceAll("$", "\\$")}" | -S ${command}`,
		{ stdio: pipe },
	);
	return execOut;
}
