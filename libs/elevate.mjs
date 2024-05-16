const chpr = require("child_process")

class Shell {
	constructor(username, shell, password) {
		this.username = username
		this.shell = shell
		this.password = password
		this.s_process = chpr.exec(`su ${username}\n`)

		this.s_process.stdin.write(this.password+"\n")
		this.s_process.stderr.on("data", (data) => {console.log("Shell Err: " + data)})
		this.s_process.stdout.on("data", (data) => {console.log("Shell Out: " + data)})
	}
	write(command) {
		this.s_process.stdin.write(command)
		console.log("Shell In: " + command)
	}
	kill() {
		this.s_process.kill()
	}
}
