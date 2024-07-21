const chpr = require("child_process");

const zlog = require("./zlog");

const logShellOut = true;

module.exports = class Shell {
	// Class that generates a shell to launch a new virtual shell with command "su ..."
	constructor(
		username,
		shell = "sh",
		password,
		exitcall = () => {},
		useTimeout = true,
		timeout = 100000,
		errorOnStderr = false,
	) {
		this.username = username;
		this.shell = shell;
		this.password = password;
		this.s_process = chpr.exec(`su -c ${this.shell} ${username}\n`);
		this.pid = this.s_process.pid;
		this.authed = false;
		this.outCall = () => {};
		zlog("Shell summoned", "info");
		this.authed = false;
		this.s_process.stderr.on("data", (data) => {
			if (data == "Password: " && !this.authed) {
				this.s_process.stdin.write(this.password + "\n");
				this.authed = true;
			}
			zlog(`Shell Err: ${data}`);
		});

		this.s_process.on("exit", (data) => {
			exitcall(data);
		});
		this.errorOnStderr = errorOnStderr;
		if (useTimeout) {
			setTimeout(() => {
				this.kill();
			}, timeout);
		}
	}

	write(command) {
		zlog(`Shell In: ${command}`);
		return new Promise((resolve, reject) => {
			if (!this.authed) {
				setTimeout(() => {
					this.write(command).then(resolve).catch(reject);
				}, 500);
				return;
			}

			const handleStdout = (data) => {
				resolve(data);
				this.s_process.stdout.removeListener("data", handleStdout);
				if (logShellOut) {
					zlog(`Shell Out: ${data}`);
				}
			};

			const handleStderr = (data) => {
				if (this.errorOnStderr) {
					reject(new Error(`Stderr of command ${command} is: ${data}`));
					this.s_process.stdout.removeListener("data", handleStderr);
					zlog(`Shell Err: ${data}`, "error");
				} else {
					zlog(`Supressed Shell Error: ${data}`, "error");
				}
				if (this.killOnStderr) {
					this.kill();
				}
			};

			this.s_process.stdout.on("data", handleStdout);
			this.s_process.stderr.on("data", handleStderr);

			try {
				this.s_process.stdin.write(command);
			} catch (err) {
				zlog(err, "error");
			}
		});
	}

	kill() {
		this.s_process.kill();
	}
};
