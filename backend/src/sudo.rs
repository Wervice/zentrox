use crate::config_file;
use std::io::{Error, Read, Write};
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
    decrypted
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

    pub fn spawn(&self) -> Result<i32, String> {

        let mut handle = Command::new("sudo")
            .arg("-S")
            .arg("-k")
            .arg(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            // .stderr(Stdio::piped())
            // .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let password = self.password.to_string();
        let mut stdinput = handle.stdin.take().unwrap();
        stdinput
            .write_all(format!("{}\n", password).as_bytes())
            .expect("Failed to write to stdin (password)");
        std::thread::sleep(std::time::Duration::from_millis(500));
        match handle.wait().unwrap().code() {
            Some(code) => Ok(code as i32),
            None => Ok(0)
        }
    }
}
