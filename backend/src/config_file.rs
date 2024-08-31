use std::fs;
use std::path;

/// Read from a key-value config file.
///
/// * `key` - Key that leads to the required value
/// The file is stored in the TOML file format.
/// The toml file is located under $HOME/zentrox_data/config.toml
/// If the key is not found an empty string "" is returned.
pub fn read(key: &str) -> String {
    let config_file = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data")
        .join("config.toml");

    match fs::read_to_string(config_file)
        .expect("Failed to read config file")
        .parse::<toml::Table>()
        .unwrap()
        .get(key)
    {
        Some(value) => value.to_string().replace("\"", ""),
        None => String::from(""),
    }
}

/// Read write to key-value config file.
///
/// * `key` - Key where value is stored
/// * `value` - Value that should be stored
/// The file is stored in the TOML file format.
/// The toml file is located under $HOME/zentrox_data/config.toml
/// If the key already exists the current value is overwritten.
/// The function will return (), if everything worked and an error in case of a fail during the
/// write to the config file.
pub fn write(key: &str, value: &str) -> Result<(), std::io::Error> {
    let config_file = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data")
        .join("config.toml");

    let mut config_file_parsed = fs::read_to_string(&config_file)
        .expect("Failed to read config file")
        .parse::<toml_edit::DocumentMut>()
        .expect("Failed to parse config file");
    config_file_parsed[key] = toml_edit::value(value);

    match fs::write(config_file, config_file_parsed.to_string()) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    Ok(())
}
