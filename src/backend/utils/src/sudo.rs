use std::fmt::Display;
use std::io::BufReader;
use std::io::{Read, Write};
use std::process::Command;
use std::process::Stdio;
use std::thread;

#[derive(Debug)]
pub struct SudoCommand {
    password: String,
    program: String,
    args: Vec<String>,
}

#[derive(Debug)]
pub struct SudoOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: Option<i32>,
}

/// Errors while running sudo
#[derive(Debug)]
pub enum SudoError {
    /// The supplied sudo password is invalid.
    WrongPassword,
    /// Joining the process thread with the main thread failed.
    JoiningFailed,
    /// The user running Zentrox is not in the `sudoers` file.
    NotInSudoers,
    /// The supplied information may not be passed to sudo
    BadParameters,
}

impl SudoCommand {
    /// Create new SudoCommand
    /// * `password` - The password used for `sudo`
    /// * `program` - The command without arguments that will be launched
    pub fn new<A: Display, B: Display>(password: A, program: B) -> SudoCommand {
        SudoCommand {
            password: password.to_string(),
            program: program.to_string(),
            args: vec![],
        }
    }

    /// Adds an argument
    /// * `argument` - The argument that will be added
    pub fn arg<T: Display>(&mut self, argument: T) -> &mut Self {
        self.args = {
            let mut v = self.args.clone();
            v.push(argument.to_string());
            v
        };

        self
    }

    /// Adds multiple arguments
    pub fn args<T: Display>(&mut self, arguments: Vec<T>) -> &mut Self {
        for arg in arguments {
            self.arg(arg);
        }

        self
    }

    pub fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }

    /// Spawns a SudoCommand and captures the contents of standard input & standard error.
    pub fn output(&self) -> Result<SudoOutput, SudoError> {
        let args = self.args.clone();
        let password = self.password.clone();
        let program = self.program.clone();

        let prohibited_program = &[' ', '\n', '\r', '\t'];
        let prohibited_password = &['\n', '\r'];
        if program.chars().any(|x| prohibited_program.contains(&x))
            || password.chars().any(|x| prohibited_password.contains(&x))
        {
            return Err(SudoError::BadParameters);
        }

        let thread_handle = thread::spawn(move || {
            let mut command_handle = Command::new("sudo")
                .arg("-S")
                .arg("-k")
                .arg(program)
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
                writeln!(stdin, "{password}").expect("Failed to write password to stdin");
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

            if stderr_content.contains("Sorry, try again.") {
                return Err(SudoError::WrongPassword);
            }
            if stderr_content.contains("is not in the sudoers file.") {
                return Err(SudoError::NotInSudoers);
            }

            // Return the output if everything succeeded
            Ok(SudoOutput {
                stdout: stdout_content,
                stderr: stderr_content,
                status: command_handle.wait().unwrap().code(),
            })
        });

        thread_handle
            .join()
            .unwrap_or(Err(SudoError::JoiningFailed))
    }
}

pub fn verify_password(password: String) -> bool {
    let mut c = Command::new("sudo");
    c.args(["-S", "-k", "-v"]);
    c.stdin(Stdio::piped());
    c.stdout(Stdio::null());
    c.stderr(Stdio::piped());
    let mut h = c.spawn().unwrap();
    let mut stdin = h.stdin.take().unwrap();
    let stderr = h.stderr.take().unwrap();
    let _ = thread::spawn(move || {
        writeln!(stdin, "{password}").expect("Failed to write password to stdin");
        stdin.flush().expect("Failed to flush stdin");
    });
    let mut e_reader = BufReader::new(stderr);
    let mut stderr_content = String::new();
    e_reader
        .read_to_string(&mut stderr_content)
        .expect("Failed to read stdout");
    !stderr_content.contains("Sorry, try again.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticate_with_result() {
        let password =
            std::env::var("TEST_PASSWORD").expect("Requires TEST_PASSWORD environment variable");
        let x = SudoCommand::new(password, "echo")
            .args(vec!["tested"])
            .output();
        if let Err(e) = x {
            panic!("Failed with: {:?}", e);
        }
    }

    #[test]
    fn authenticate_with_output() {
        let password =
            std::env::var("TEST_PASSWORD").expect("Requires TEST_PASSWORD environment variable");
        let x = SudoCommand::new(password, "echo")
            .args(vec!["tested"])
            .output();
        if let Err(e) = x {
            panic!("Failed with: {:?}", e);
        }
    }
}
