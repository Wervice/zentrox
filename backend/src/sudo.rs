use std::io::BufReader;
use std::io::{Read, Write};
use std::process::Command;
use std::process::Stdio;
use std::thread;

pub struct SwitchedUserCommand {
    password: String,
    command: String,
    args: Vec<String>,
}

/// SudoOutput used with .output()
#[allow(dead_code)]
pub struct SudoOuput {
    pub stdout: String,
    pub stderr: String,
}

impl SwitchedUserCommand {
    /// Create new SwitchedUserCommand.
    /// * `password` - The password used for `sudo`
    /// * `command` - The command without arguments that will be launched
    pub fn new(password: String, command: String) -> SwitchedUserCommand {
        SwitchedUserCommand {
            password,
            command,
            args: vec![],
        }
    }

    /// Adds an argument to an exisiting SwitchedUserCommand
    /// * `argument` - The argument that will be added
    pub fn arg(&mut self, argument: String) -> &mut Self {
        self.args = {
            let mut v = self.args.clone();
            v.push(argument);
            v
        };

        self
    }

    /// Spawns the SwitchedUserCommand with every added argument and the command.
    /// The final `sudo` command looks like this:
    /// `sudo -S -k COMMAND ARGS`
    ///
    /// The password is written to stdin without ANY checks if a password was required.
    /// Normaly, sudo will ask for a password. If sudo does not ask for a password, the password
    /// will still be written to stdin. Normaly, the input will just be ignored. If th command that
    /// was passed does read from stdin though, the input will be read.
    ///
    /// As a Result the exist status (i32) will bre returned.
    pub fn spawn(&self) -> Result<i32, String> {
        // Prepare vaiables
        let args = self.args.clone();
        let password = self.password.clone();
        let command = self.command.clone();

        // Command Thread, handles the actual command
        let thread_handle = thread::spawn(move || {
            let mut command_handle = Command::new("sudo")
                .arg("-S")
                .arg("-k")
                .args(command.clone().split(" "))
                .args(args)
                .stdin(Stdio::piped())
                // .stderr(Stdio::piped())
                // .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn process");

            let password = password;
            let mut stdinput = command_handle.stdin.take().unwrap();
            stdinput
                .write_all(format!("{}\n", password).as_bytes())
                .expect("Failed to write to stdin (password)");
            std::thread::sleep(std::time::Duration::from_millis(500));
            match command_handle.wait().unwrap().code() {
                Some(code) => Ok(code),
                None => Ok(0),
            }
        });

        thread_handle
            .join()
            .expect("Failed to join command thread with main thread")
    }

    #[allow(dead_code)]
    /// Spawns the SwitchedUserCommand with every added argument and the command.
    /// The final `sudo` command looks like this:
    /// `sudo -S -k COMMAND ARGS`
    ///
    /// The password is written to stdin without ANY checks if a password was required.
    /// Normaly, sudo will ask for a password. If sudo does not ask for a password, the password
    /// will still be written to stdin. Normaly, the input will just be ignored. If th command that
    /// was passed does read from stdin though, the input will be read.
    ///
    pub fn output(&self) -> Result<SudoOuput, String> {
        // Prepare variables
        let args = self.args.clone();
        let password = self.password.clone();
        let command = self.command.clone();

        // Command Thread, handles the actual command
        let thread_handle = thread::spawn(move || {
            let mut command_handle = Command::new("sudo")
                .arg("-S") // Read password from stdin
                .arg("-k") // Force sudo to prompt for the password
                .args(command.clone().split(" "))
                .args(args)
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn process");

            // Capture output and errors

            let mut stdin = command_handle.stdin.take().expect("Failed to open stdin");
            let stdout = command_handle
                .stdout
                .take()
                .expect("Failed to capture stdout");
            let stderr = command_handle
                .stderr
                .take()
                .expect("Failed to capture stderr");

            let _ = thread::spawn(move || {
                writeln!(stdin, "{}", password).expect("Failed to write password to stdin");
                stdin.flush().expect("Failed to flush stdin");
            });

            let mut o_reader = BufReader::new(stdout);
            let mut stdout_content = String::new();
            o_reader
                .read_to_string(&mut stdout_content)
                .expect("Failed to read stdout");

            let mut e_reader = BufReader::new(stderr);
            let mut stderr_content = String::new();
            e_reader
                .read_to_string(&mut stderr_content)
                .expect("Failed to read stderr");

            // Check if sudo reported an incorrect password

            if stderr_content.contains("sudo")
                && stderr_content.contains("incorrect password attempt")
            {
                return Err("Incorrect password provided".to_string());
            }

            // Return the output if everything succeeded
            Ok(SudoOuput {
                stdout: stdout_content,
                stderr: stderr_content,
            })
        });

        // Join the thread and return the result
        thread_handle
            .join()
            .unwrap_or_else(|_| Err("Failed to join thread".to_string()))
    }
}
