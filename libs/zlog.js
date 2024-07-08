module.exports = function zlog(input, type = "info") {
	var line;
	input = String(input);
	if (type == "info") {
		const lines = input.split("\n");
		const dateTime = new Date().toLocaleString();
		for (line of lines) {
			console.log("[ \x1B[32m✅ Info\x1B[0m  " + dateTime + "] " + line);
		}
	} else if (type == "error") {
		const lines = input.split("\n");
		const dateTime = new Date().toLocaleString();
		for (line of lines) {
			console.error("[ \x1B[31m❎ Error \x1B[0m" + dateTime + "] " + line);
		}
	}
}
