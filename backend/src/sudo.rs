use crate::config_file;
use std::io::{Read, Write};
use std::process::Command;
use std::process::Stdio;
use std::str;

fn decrypt_aes_256_cbc(password: &String, ciphertext: String) -> String {
    let command = format!(
        "echo {} | openssl aes-256-cbc -d -a -pbkdf2 -pass pass:{}",
        ciphertext, password
    );

    // Execute the command and capture the output
    let output = Command::new("sh").arg("-c").arg(&command).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Convert the output to a string and replace newline characters
                let result = str::from_utf8(&output.stdout)
                    .unwrap_or("")
                    .replace('\n', "");
                result
            } else {
                // Handle non-zero exit status
                String::new()
            }
        }
        Err(_err) => {
            // Handle the error when executing the command
            String::new()
        }
    }
}

pub fn admin_password(decryption_key: &String) -> String {
    let decrypted =
        decrypt_aes_256_cbc(decryption_key, config_file::read("zentrox_admin_password"));
    println!("{}", decrypted);
    decrypted
}

pub fn spawn(username: String, password: String, command: String) -> () {
    std::thread::spawn(move || {
        let mut child = Command::new("su")
            .arg(username)
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
            stdin
                .write_all(format!("{}\n", password).as_bytes())
                .expect("Failed to write to stdin");
            std::thread::sleep(std::time::Duration::from_secs(1));

            stdin
                .write_all(format!("{}\n", command).as_bytes())
                .expect("Failed to write to stdin");
        });
        let output = child.wait_with_output().expect("Failed to read stdout");
        String::from_utf8_lossy(&output.stdout).to_string()
    });
}

pub struct SwitchedUserCommand {
    username: String,
    password: String,
    command: String,
    args: Vec<String>,
}

impl SwitchedUserCommand {
    pub fn new(username: String, password: String, command: String) -> SwitchedUserCommand {
        return SwitchedUserCommand {
            username,
            password,
            command,
            args: vec![],
        };
    }

    pub fn arg(&mut self, argument: String) -> &mut Self {
        self.args = {
            let mut v = self.args.clone();
            v.push(argument);
            v
        };

        self
    }

    pub fn spawn(&self) -> () {
        // Spawn the child process
        let mut full_command = format!("{} ", String::from(&self.command));
        for argument in &self.args {
            full_command = format!("{}{} ", full_command, argument)
        }

        let mut handle = Command::new("su")
            .arg(&self.username)
            .arg(format!("--command=\"{}\"", full_command))
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let password = self.password.to_string();
        // Take stdin from the child process
        let _ = std::thread::Builder::new().spawn(move || {
            let mut stdinput = handle.stdin.take().unwrap();
            stdinput
                .write_all(format!("{}\n", password).as_bytes())
                .expect("Failed to write to stdin (password)");
            let mut stderror = handle.stderr.take().unwrap();
            let mut stderrbuff = String::new();
            let _ = stderror.read_to_string(&mut stderrbuff);
        });
    }
}
