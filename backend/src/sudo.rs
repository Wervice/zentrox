use rand::distributions::{Alphanumeric, DistString};
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

pub enum SudoExecutionResult {
    Success(i32),
    WrongPassword,
    Unauthorized,
    ExecutionError(String),
}

pub enum SudoExecutionOutput {
    Success(SudoOuput),
    WrongPassword,
    Unauthorized,
    ExecutionError(String),
}

impl SwitchedUserCommand {
    /// Create new SwitchedUserCommand.
    /// * `password` - The password used for `sudo`
    /// * `command` - The command without arguments that will be launched
    pub fn new<T: ToString>(password: T, command: T) -> SwitchedUserCommand {
        SwitchedUserCommand {
            password: password.to_string(),
            command: command.to_string(),
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

    pub fn args(&mut self, arguments: Vec<&str>) -> &mut Self {
        for arg in arguments {
            self.arg(arg.to_string());
        }

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
    pub fn spawn(&self) -> SudoExecutionResult {
        // Prepare vaiables
        let args = self.args.clone();
        let password = self.password.clone();
        let command = self.command.clone();
        let random_prompt = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 16)
            .to_ascii_lowercase();

        // Command Thread, handles the actual command
        let thread_handle = thread::spawn(move || {
            let mut command_handle = Command::new("sudo")
                .arg("-S")
                .arg("-k")
                .arg("--prompt")
                .arg(&random_prompt)
                .args(command.clone().split(" "))
                .args(args)
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                // .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn process");

            let password = password;
            let mut stdinput = command_handle.stdin.take().unwrap();
            let mut stderr = command_handle.stderr.take().unwrap();
            let sent_prompt = &mut [0; 16];
            let sent_prompt_read = stderr.read(sent_prompt);
            if let Err(e) = sent_prompt_read {
                println!("{e}");
                return SudoExecutionResult::ExecutionError(
                    "Failed to read prompt from sudo".to_string(),
                );
            }
            dbg!(&sent_prompt);
            dbg!(&random_prompt);
            let sent_prompt_string: String = sent_prompt.map(|x| (x as char).to_string()).join("");
            if sent_prompt_string != random_prompt {
                return SudoExecutionResult::ExecutionError("Unequal prompts".to_string());
            }
            stdinput
                .write_all(format!("{}\n", password).as_bytes())
                .expect("Failed to write to stdin (password)");
            std::thread::sleep(std::time::Duration::from_millis(500));
            let after_password = &mut [0; 64];
            let _ = stderr.read(after_password);
            let after_password_string: String =
                after_password.map(|x| (x as char).to_string()).join("");
            if after_password_string.contains("Sorry, try again.") {
                return SudoExecutionResult::WrongPassword;
            }
            if after_password_string.contains("is not in the sudoers file.") {
                return SudoExecutionResult::Unauthorized;
            }
            match command_handle.wait().unwrap().code() {
                Some(code) => SudoExecutionResult::Success(code),
                None => SudoExecutionResult::Success(0),
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
    pub fn output(&self) -> SudoExecutionOutput {
        // Prepare variables
        let args = self.args.clone();
        let password = self.password.clone();
        let command = self.command.clone();
        let random_prompt = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 16)
            .to_ascii_lowercase();

        // Command Thread, handles the actual command
        let thread_handle = thread::spawn(move || {
            let mut command_handle = Command::new("sudo")
                .arg("-S") // Read password from stdin
                .arg("-k") // Force sudo to prompt for the password
                .arg("--prompt")
                .arg(&random_prompt)
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
            let mut stderr = command_handle
                .stderr
                .take()
                .expect("Failed to capture stderr");
            let sent_prompt = &mut [0; 16];
            let sent_prompt_read = stderr.read(sent_prompt);
            if let Err(e) = sent_prompt_read {
                println!("{e}");
                return SudoExecutionOutput::ExecutionError(
                    "Failed to read prompt from sudo".to_string(),
                );
            }
            dbg!(&sent_prompt);
            dbg!(&random_prompt);
            if sent_prompt.iter().map(|x| *x).collect::<Vec<u8>>()
                != random_prompt
                    .chars()
                    .map(|x| x.to_ascii_lowercase() as u8)
                    .collect::<Vec<u8>>()
            {
                return SudoExecutionOutput::ExecutionError("Unequal prompts".to_string());
            }
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

            if stderr_content.contains("incorrect password attempt")
                || stderr_content.contains("Sorry, try again.")
            {
                dbg!(stderr_content);
                return SudoExecutionOutput::WrongPassword;
            }
            if stderr_content.contains("is not in the sudoers file.") {
                return SudoExecutionOutput::Unauthorized;
            }

            // Return the output if everything succeeded
            SudoExecutionOutput::Success(SudoOuput {
                stdout: stdout_content,
                stderr: stderr_content,
            })
        });

        // Join the thread and return the result
        thread_handle.join().unwrap_or_else(|_| {
            SudoExecutionOutput::ExecutionError("Failed to join thread".to_string())
        })
    }
}
