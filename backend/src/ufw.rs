// 1. Add new rule
// 2. Remove rule
// 3. Enable/Disable UFW
// 4. List rules
use crate::sudo::SwitchedUserCommand;
use regex::Regex;

#[derive(serde::Serialize)]
pub struct UfwRule {
    pub index: u32,
    pub to: String,
    pub from: String,
    pub action: String,
}

/// Fetches the current UFW status
///
/// The status is fetched using the command `/usr/sbin/ufw status` which is ran using `sudo`.
/// This invokes the ufw executable and requests the status and a list of rules in form of a table.
/// The first line of the ouput contains the status. i.e.: "Status: active" or "Status: inactive".
/// Bellow that is a table of rules.
///
/// These values are parsed into a tuple containg a bool that shows if the ufw is enable or not.
/// True: Enabled; False: Disabled
/// And a vector containing UfwRules with index, to, from and action.
///
/// * `password` - The password used to authenticate `sudo`
pub fn ufw_status(password: String) -> Result<(bool, Vec<UfwRule>), String> {
    let output = match SwitchedUserCommand::new(password, "/usr/sbin/ufw status".to_string())
        .output() {
            Ok(v) => v.stdout,
            Err(_) => {
                return Err("Wrong sudo password".to_string())
            }
        };
    let mut output_lines = output
        .lines()
        .map(|x| x.to_string())
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>();

    if output_lines.len() == 0 {
        return Err("Invalid Sudo Password or failed to read UFW status".to_string());
    }

    let enabled = output_lines[0] == "Status: active";

    let mut rules_vec: Vec<UfwRule> = Vec::new();
    if enabled && output_lines.len() > 3 {
        output_lines.drain(0..3);

        let mut index: i32 = 1;

        let re = Regex::new(r"\s{2,}").unwrap();

        for line in output_lines {
            let l_s: Vec<&str> = re.split(&line).collect();
            let to = l_s[0];
            let action = l_s[1];
            let from = l_s[2];
            rules_vec.push(UfwRule {
                to: String::from(to),
                from: String::from(from),
                action: String::from(action),
                index: index as u32,
            });

            index += 1;
        }
    }

    Ok((enabled, rules_vec))
}

/// Create new UFW rule by spawning a command containing the from, to and action value.
///
/// The command is spawned with `sudo` to allow for adding rules.
/// * `password` - The password used to authenticate sudo.
/// * `from` - From IP/Port used to create new rule. This may not be empty.
/// * `to` - To IP/Port used to create new rule. This may not be empty.
/// * `action` - Action used to create new rule. This may be "allow" or "deny"
pub fn new_rule(password: String, from: String, to: String, action: String) -> Result<(), String> {
    let command = format!(
        "/usr/sbin/ufw {} from {} to any port {}",
        action.to_lowercase(),
        from.to_lowercase(),
        to.to_lowercase()
    );

    match SwitchedUserCommand::new(password, command).spawn() {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to spawn command".to_string()),
    }
}

/// Deletes UFW rule by spawning a command contaiting the rules index
///
/// The command is spawned with `sudo` to allow for deletign rules.
/// `password` - The password to authenticate sudo.
/// `index` - The index of the rule to delete.
pub fn delete_rule(password: String, index: u32) -> Result<(), String> {
    let command = format!("/usr/sbin/ufw --force delete {} ", index);

    match SwitchedUserCommand::new(password, command).spawn() {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to spawn command".to_string()),
    }
}
