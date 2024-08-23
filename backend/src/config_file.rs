use dirs;
use std::fs;
use std::path;
use toml;
use toml_edit;

pub fn read(key: &str) -> String {
    let zentrox_installation_path = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data");
    let config_file = zentrox_installation_path.join("config.toml");

    let _ = match fs::read_to_string(config_file)
        .expect("Failed to read config file")
        .parse::<toml::Table>()
        .unwrap()
        .get(key)
    {
        Some(value) => return value.to_string().replace("\"", ""),
        None => return "".to_string(),
    };
}

pub fn write(key: &str, value: &str) -> bool {
    let zentrox_installation_path = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data");
    let config_file = zentrox_installation_path.join("config.toml");
    let mut config_file_parsed = fs::read_to_string(&config_file)
        .expect("Failed to read config file")
        .to_string()
        .parse::<toml_edit::DocumentMut>()
        .expect("Failed to parse config file");
    config_file_parsed[key] = toml_edit::value(value);
    let _ = fs::write(config_file, config_file_parsed.to_string());
    return true;
}
