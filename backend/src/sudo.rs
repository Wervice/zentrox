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
pub struct SudoOuput {
    stdout: String,
    stderr: String,
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
        
        let out = thread_handle.join().expect("Failed to join command thread with main thread");
        out    
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
    /// As a Result a SudoOutput will be returned. This struct contains the stderr and stdout
    /// contents of the spawned command in form of a UTF-8 lossy string.
    pub fn output(&self) -> SudoOuput {
        let mut handle = Command::new("sudo")
            .arg("-S")
            .arg("-k")
            .arg(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let password = self.password.to_string();
        let mut stdinput = handle.stdin.take().unwrap();
        stdinput
            .write_all(format!("{}\n", password).as_bytes())
            .expect("Failed to write to stdin (password)");
        std::thread::sleep(std::time::Duration::from_millis(500));

        let out_bytes = handle.stdout.take().unwrap().bytes();
        let err_bytes = handle.stderr.take().unwrap().bytes();

        let mut out_bytes_unwraped = Vec::<u8>::new();
        let mut err_bytes_unwraped = Vec::<u8>::new();

        for byte in out_bytes {
            out_bytes_unwraped.push(byte.unwrap_or(0_u8))
        }

        for byte in err_bytes {
            err_bytes_unwraped.push(match byte {
                Ok(b) => b,
                Err(_) => 0_u8,
            })
        }

        SudoOuput {
            stdout: String::from_utf8_lossy(&out_bytes_unwraped).to_string(),
            stderr: String::from_utf8_lossy(&err_bytes_unwraped).to_string(),
        }
    }
}
